// RTGC-0.8 Main Entry Point - Simple engine runner
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rtgc::engine::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализация логгера для отладки
    tracing_subscriber::fmt::init();
    
    let mut engine = Engine::new()?;
    engine.run()?;
    Ok(())
}
