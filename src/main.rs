// RTGC-0.8 Main Entry Point - Simple engine runner
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rtgc::engine::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut engine = Engine::new(&event_loop)?;
    engine.run()?;
    Ok(())
}
