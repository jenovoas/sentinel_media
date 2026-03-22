use log::{info, warn};
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Constantes para configuraciones de GPU
const NVIDIA_ENCODER_H264: &str = "h264_nvenc";
const CPU_ENCODER: &str = "libx264";

/// Estructura para manejar operaciones de video
pub struct VideoEngine {
    gpu_enabled: bool,
    encoder: &'static str,
}

impl VideoEngine {
    /// Inicializa el motor de video detectando capacidades de hardware
    pub async fn new() -> Self {
        let gpu_available = Self::check_nvenc_availability().await;

        // Selección de estrategia de encoding basada en hardware
        let encoder = if gpu_available {
            info!("🚀 GPU Acceleration DETECTED: Activando protocolo NVENC");
            NVIDIA_ENCODER_H264
        } else {
            warn!("⚠️ GPU Acceleration NOT AVAILABLE: Fallback a CPU (libx264)");
            CPU_ENCODER
        };

        Self {
            gpu_enabled: gpu_available,
            encoder,
        }
    }

    /// Verifica si el encoder NVENC está disponible en el sistema
    async fn check_nvenc_availability() -> bool {
        // Ejecutamos 'ffmpeg -encoders' y buscamos 'nvenc'
        match Command::new("ffmpeg").arg("-encoders").output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout.contains("nvenc")
            },
            Err(_) => false,
        }
    }

    /// Realiza el stitching (unión) de múltiples clips de video usando GPU si es posible.
    ///
    /// # Arguments
    /// * `inputs` - Lista de rutas absolutas a los videos de entrada
    /// * `output` - Ruta donde se guardará el video final
    pub async fn stitch_clips(&self, inputs: &[PathBuf], output: &Path) -> Result<(), String> {
        if inputs.is_empty() {
            return Err("No input clips provided".to_string());
        }

        // Crear archivo de lista temporal para ffmpeg concat demuxer
        // Formato: file '/path/to/file'
        let mut list_content = String::new();
        for path in inputs {
            // Escapar comillas simples en la ruta
            let path_str = path.to_string_lossy().replace("'", "'\\''");
            list_content.push_str(&format!("file '{}'\n", path_str));
        }

        // Usamos /dev/shm para el archivo temporal (Zero-Copy / RAM Disk)
        let list_path = PathBuf::from("/dev/shm").join(format!(
            "sentinel_stitch_{}.txt",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        if let Err(e) = tokio::fs::write(&list_path, list_content).await {
            return Err(format!("Failed to write concat list to RAM: {}", e));
        }

        info!(
            "Iniciando Video Stitching: {} clips -> {}",
            inputs.len(),
            output.display()
        );

        // Componer comando FFmpeg
        let mut cmd = Command::new("ffmpeg");

        // Habilitar aceleración de hardware para decodificación si es posible (cuvid/cuda)
        if self.gpu_enabled {
            cmd.arg("-hwaccel").arg("cuda");
            cmd.arg("-hwaccel_output_format").arg("cuda");
        }

        cmd.arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-i")
            .arg(&list_path);

        if self.gpu_enabled {
            // Configuración NVENC para alta velocidad y calidad balanceada
            cmd.arg("-c:v").arg(self.encoder)
               .arg("-preset").arg("p4") // Preset medio-rápido (p1-p7)
               .arg("-tune").arg("hq")
               .arg("-rc").arg("vbr")
               .arg("-cq").arg("23") // Calidad constante aceptable
               .arg("-b:v").arg("5M") // Bitrate objetivo 5Mbps
               .arg("-maxrate").arg("10M")
               .arg("-bufsize").arg("10M");
        } else {
            // Fallback CPU
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg("fast")
                .arg("-crf")
                .arg("23");
        }

        // Audio copy (sin transcodificar para velocidad máxima) o aac si es necesario
        cmd.arg("-c:a").arg("aac")
           .arg("-b:a").arg("192k")
           // Sobrescribir salida
           .arg("-y")
           .arg(output);

        // Ejecutar
        let status = cmd
            .status()
            .await
            .map_err(|e| format!("FFmpeg execution failed: {}", e))?;

        // Limpieza (Fire & Forget)
        let _ = tokio::fs::remove_file(&list_path).await;

        if status.success() {
            info!("Stitching completado exitosamente: {}", output.display());
            Ok(())
        } else {
            Err(format!(
                "FFmpeg exited with error code: {:?}",
                status.code()
            ))
        }
    }
}
