//! # 🛡️ SHARED MEMORY BRIDGE (ANCHOR) - RUST PURE 🛡️
//! Provides `SharedBuffer` to anchor Liquid Lattice to Host RAM.
//! Uses POSIX shared memory (shm_open/mmap).

use libc::{close, ftruncate, mmap, munmap, shm_open, shm_unlink};
use libc::{MAP_FAILED, MAP_SHARED, O_CREAT, O_EXCL, O_RDWR, PROT_READ, PROT_WRITE};
use std::ffi::CString;
use std::ptr;
use std::sync::atomic::{fence, Ordering};
use std::io::{Error, ErrorKind, Result};

pub struct SharedBuffer {
    pub name: String,
    pub size: usize,
    // FIX #4: ptr y fd son privados — el caller no debe manipularlos directamente
    ptr: *mut u8,
    fd: i32,
    pub is_owner: bool,
}

// SAFETY: Synchronization must be handled by caller.
unsafe impl Send for SharedBuffer {}
unsafe impl Sync for SharedBuffer {}

impl SharedBuffer {
    pub fn new(name: String, size: usize, create: bool) -> Result<Self> {
        let c_name = CString::new(name.clone()).map_err(|e| {
            Error::new(ErrorKind::InvalidInput, format!("Invalid name: {}", e))
        })?;

        unsafe {
            let fd = if create {
                // FIX #3: O_EXCL evita que dos procesos creen el mismo buffer simultáneamente
                let fd = shm_open(c_name.as_ptr(), O_CREAT | O_RDWR | O_EXCL, 0o666);
                if fd == -1 {
                    let err = Error::last_os_error();
                    // EEXIST: el buffer quedó de un crash anterior — limpiar y reintentar
                    if err.raw_os_error() == Some(libc::EEXIST) {
                        shm_unlink(c_name.as_ptr());
                        let fd2 = shm_open(c_name.as_ptr(), O_CREAT | O_RDWR | O_EXCL, 0o666);
                        if fd2 == -1 {
                            return Err(Error::last_os_error());
                        }
                        fd2
                    } else {
                        return Err(err);
                    }
                } else {
                    fd
                }
            } else {
                let fd = shm_open(c_name.as_ptr(), O_RDWR, 0o666);
                if fd == -1 {
                    return Err(Error::last_os_error());
                }
                fd
            };

            if create && ftruncate(fd, size as i64) == -1 {
                close(fd);
                return Err(Error::last_os_error());
            }

            let ptr = mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                0,
            );

            if ptr == MAP_FAILED {
                close(fd);
                return Err(Error::last_os_error());
            }

            Ok(SharedBuffer {
                name,
                size,
                ptr: ptr as *mut u8,
                fd,
                is_owner: create,
            })
        }
    }

    pub fn write(&self, offset: usize, data: &[u8]) -> Result<usize> {
        if offset + data.len() > self.size {
            return Err(Error::new(ErrorKind::Other, "Write out of bounds"));
        }
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.add(offset), data.len());
        }
        // FIX #2: fence Release — garantiza que la escritura es visible antes de que
        // cualquier otro proceso/hilo lea. Sin esto el compilador puede reordenar.
        fence(Ordering::Release);
        Ok(data.len())
    }

    pub fn read(&self, offset: usize, length: usize) -> Result<Vec<u8>> {
        if offset + length > self.size {
            return Err(Error::new(ErrorKind::Other, "Read out of bounds"));
        }
        // FIX #2: fence Acquire — espera a que todas las escrituras previas sean visibles
        fence(Ordering::Acquire);
        let mut buffer = vec![0u8; length];
        unsafe {
            ptr::copy_nonoverlapping(self.ptr.add(offset), buffer.as_mut_ptr(), length);
        }
        Ok(buffer)
    }

    pub fn close(&mut self) {
        unsafe {
            if !self.ptr.is_null() && self.ptr != MAP_FAILED as *mut u8 {
                munmap(self.ptr as *mut libc::c_void, self.size);
                self.ptr = ptr::null_mut();
            }
            if self.fd != -1 {
                close(self.fd);
                self.fd = -1;
            }
        }
    }

    /// Devuelve el puntero crudo — solo para logging/debugging, nunca para escribir directamente.
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn unlink(&self) {
        if self.is_owner {
            if let Ok(c_name) = CString::new(self.name.clone()) {
                unsafe {
                    shm_unlink(c_name.as_ptr());
                }
            }
        }
    }
}

impl Drop for SharedBuffer {
    fn drop(&mut self) {
        // FIX #1: unlink antes de close — el owner destruye el segmento al morir
        // Orden crítico: primero unlink (borra la entrada en /dev/shm/),
        // luego close (libera el fd y el mapping de este proceso)
        self.unlink();
        self.close();
    }
}
