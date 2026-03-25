//! Weather System Module
//!
//! Provides dynamic weather simulation including:
//! - Multiple weather types (clear, rain, snow, storms, fog)
//! - Cloud layers and sky coloring
//! - Wind simulation with gusts and altitude variation
//! - Precipitation particles
//! - Lightning effects
//! - Atmospheric conditions (temperature, humidity, pressure)

pub mod dynamic_weather;

pub use dynamic_weather::*;
