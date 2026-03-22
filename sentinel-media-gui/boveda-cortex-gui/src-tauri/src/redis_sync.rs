use tauri::Emitter;
use redis::ControlFlow;
use redis::PubSubCommands;
use std::thread;

pub fn start_redis_listener(handle: tauri::AppHandle) {
    thread::spawn(move || {
        let client = match redis::Client::open("redis://127.0.0.1/") {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ Redis Sync: Error conectando: {}", e);
                return;
            }
        };

        let mut con = match client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ Redis Sync: Error obteniendo conexión: {}", e);
                return;
            }
        };

        println!("📡 Redis Sync: Escuchando canal 'qntp_channel' (Ring 0 sync)...");

        let _: () = con.subscribe(&["qntp_channel"], |msg| {
            let payload: String = msg.get_payload().unwrap_or_default();
            // Emitir tick a la GUI para refrescar cristales
            let _ = handle.emit("ring0-tick", payload);
            ControlFlow::Continue
        }).unwrap_or_else(|e| {
            eprintln!("❌ Redis Sync: Error en suscripción: {}", e);
        });
    });
}
