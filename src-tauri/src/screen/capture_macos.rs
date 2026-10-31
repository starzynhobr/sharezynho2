use std::sync::Arc;
use std::time::Duration;
use xcap::Monitor;

use super::capture::CaptureFlags;

pub fn monitor_names() -> Vec<String> {
    Monitor::all()
        .map(|list| {
            list.into_iter()
                .map(|m| m.name().unwrap_or_else(|_| "Monitor".to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub fn start(
    monitor_index: usize,
    flags: Arc<CaptureFlags>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (error_tx, error_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let monitor = match Monitor::all().map(|list| list.into_iter().nth(monitor_index)) {
            Ok(Some(monitor)) => monitor,
            Ok(None) => {
                let _ = error_tx.send("Monitor não encontrado".to_string());
                return;
            }
            Err(e) => {
                let _ = error_tx.send(e.to_string());
                return;
            }
        };

        while !flags.should_stop() {
            if !flags.ready_for_frame() {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
            match monitor.capture_image() {
                Ok(image) => flags.send_rgba(
                    image.as_raw(),
                    image.width() as usize,
                    image.height() as usize,
                ),
                Err(e) => {
                    let _ = error_tx.send(format!(
                        "Não foi possível capturar a tela. Verifique a permissão de Gravação de Tela: {}",
                        e
                    ));
                    return;
                }
            }
        }
    });

    if let Ok(e) = error_rx.recv_timeout(Duration::from_millis(1000)) {
        return Err(e.into());
    }
    Ok(())
}
