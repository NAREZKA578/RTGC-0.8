// RTGC-0.8 Main Entry Point - Simple engine runner
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rtgc::core;
use tracing::{error, info};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Запуск движка через центральный модуль core
    match core::run() {
        Ok(()) => {
            info!("Engine shutdown successfully");
            Ok(())
        }
        Err(e) => {
            error!("Engine failed with error: {}", e);
            eprintln!("Fatal error: {}", e);
            Err(e)
        }
    }
}
