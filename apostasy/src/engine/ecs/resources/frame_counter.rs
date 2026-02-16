use std::time::{Duration, Instant};

use crate::{self as apostasy, engine::ecs::World};
use apostasy_macros::{Resource, update};

#[derive(Resource)]
pub struct FPSCounter {
    pub last_frame_time: Instant,
    pub frame_times: Vec<Duration>,
    pub max_samples: usize,
    pub current_fps: f32,
}
impl Default for FPSCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FPSCounter {
    pub fn new() -> Self {
        Self {
            last_frame_time: Instant::now(),
            frame_times: Vec::new(),
            max_samples: 60,
            current_fps: 0.0,
        }
    }

    pub fn fps(&self) -> f32 {
        self.current_fps
    }
}

#[update]
pub fn update_fps_counter(world: &mut World) {
    world.with_resource_mut::<FPSCounter, _, _>(|fps_counter| {
        let now = Instant::now();
        let delta = now - fps_counter.last_frame_time;
        fps_counter.last_frame_time = now;

        fps_counter.frame_times.push(delta);
        if fps_counter.frame_times.len() > fps_counter.max_samples {
            fps_counter.frame_times.remove(0);
        }

        // Calculate average FPS
        let average_frame_time: Duration =
            fps_counter.frame_times.iter().sum::<Duration>() / fps_counter.frame_times.len() as u32;
        fps_counter.current_fps = 1.0 / average_frame_time.as_secs_f32();
    });
}
