use std::time::{Duration, Instant};

/// The clock/timer for the engine, used to handle information like delta time
pub struct EngineTimer {
    pub last_frame_time: Instant,
    pub accumulator: Duration,
    pub fixed_time_step: Duration,
}

impl EngineTimer {
    pub fn new() -> Self {
        Self {
            last_frame_time: Instant::now(),
            accumulator: Duration::ZERO,
            fixed_time_step: Duration::from_secs_f64(1.0 / 60.0),
        }
    }

    /// Returns the current delta time information
    pub fn tick(&mut self) -> DeltaTimeInfo {
        let current_time = Instant::now();
        let delta = current_time - self.last_frame_time;
        self.last_frame_time = current_time;

        self.accumulator += delta;

        // Calculate how many fixed updates to run
        let mut updates = 0;
        while self.accumulator >= self.fixed_time_step {
            self.accumulator -= self.fixed_time_step;
            updates += 1;
        }

        DeltaTimeInfo {
            fixed_updates: updates,
            fixed_dt: self.fixed_time_step.as_secs_f32(),
            alpha: self.accumulator.as_secs_f32() / self.fixed_time_step.as_secs_f32(),
        }
    }
}

/// The delta time information, used to store information about the delta time
pub struct DeltaTimeInfo {
    pub fixed_updates: u32,
    pub fixed_dt: f32,
    pub alpha: f32,
}
