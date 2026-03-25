use std::collections::HashMap;
use std::time::Instant;

pub struct Profiler {
    timers: HashMap<String, Instant>,
    measurements: HashMap<String, Vec<f64>>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            measurements: HashMap::new(),
        }
    }

    pub fn start_timer(&mut self, name: &str) {
        self.timers.insert(name.to_string(), Instant::now());
    }

    pub fn stop_timer(&mut self, name: &str) -> Option<f64> {
        if let Some(start_time) = self.timers.remove(name) {
            let elapsed = start_time.elapsed().as_secs_f64() * 1000.0; // Convert to milliseconds
            self.measurements.entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(elapsed);
            Some(elapsed)
        } else {
            None
        }
    }

    pub fn get_average_time(&self, name: &str) -> Option<f64> {
        if let Some(times) = self.measurements.get(name) {
            if !times.is_empty() {
                Some(times.iter().sum::<f64>() / times.len() as f64)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_last_time(&self, name: &str) -> Option<f64> {
        if let Some(times) = self.measurements.get(name) {
            times.last().copied()
        } else {
            None
        }
    }

    pub fn print_profile_report(&self) {
        println!("=== Performance Profile Report ===");
        for (name, times) in &self.measurements {
            if !times.is_empty() {
                let avg_time = times.iter().sum::<f64>() / times.len() as f64;
                let min_time = times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max_time = times.iter().fold(0.0_f64, |a, &b| a.max(b));

                println!(
                    "{}: avg={:.3}ms, min={:.3}ms, max={:.3}ms, calls={}",
                    name,
                    avg_time,
                    min_time,
                    max_time,
                    times.len()
                );
            }
        }
        println!("===================================");
    }

    pub fn reset(&mut self) {
        self.timers.clear();
        self.measurements.clear();
    }
}

// Global profiler instance using parking_lot::Mutex (does not poison on panic)
use parking_lot::Mutex;
use once_cell::sync::Lazy;

pub static PROFILER: Lazy<Mutex<Profiler>> = Lazy::new(|| Mutex::new(Profiler::new()));

#[macro_export]
macro_rules! profile_scope {
    ($name:expr, $block:block) => {{
        use crate::profiler::PROFILER;
        PROFILER.lock().start_timer($name);
        let result = $block;
        PROFILER.lock().stop_timer($name);
        result
    }};
}

pub fn start_timer(name: &str) {
    PROFILER.lock().start_timer(name);
}

pub fn stop_timer(name: &str) -> Option<f64> {
    PROFILER.lock().stop_timer(name)
}

pub fn get_average_time(name: &str) -> Option<f64> {
    PROFILER.lock().get_average_time(name)
}

pub fn get_last_time(name: &str) -> Option<f64> {
    PROFILER.lock().get_last_time(name)
}

pub fn print_profile_report() {
    PROFILER.lock().print_profile_report();
}

pub fn reset_profiler() {
    PROFILER.lock().reset();
}