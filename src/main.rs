// RTGC-0.8 Main Entry Point - Simple engine runner
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rtgc::core;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Запуск движка через центральный модуль core
    core::run()?;
    Ok(())
}
