extern crate time;

pub use self::time::Duration;

pub struct FPSCounter {
    updated_at: time::PreciseTime,
    refresh_rate: time::Duration,
    frames_rendered: i64,
    last_fps: i64,
}

impl FPSCounter {
    pub fn new(refresh_rate: time::Duration) -> FPSCounter {
        FPSCounter {
            updated_at: time::PreciseTime::now(),
            refresh_rate: refresh_rate,
            frames_rendered: 0,
            last_fps: 0,
        }
    }
    pub fn end_frame(&mut self) {
        self.frames_rendered += 1;
    }
    pub fn current_fps(&mut self) -> i64 {
        let elapsed = self.updated_at.to(time::PreciseTime::now());
        if elapsed > self.refresh_rate {
            self.updated_at = time::PreciseTime::now();
            self.last_fps = 1000 * self.frames_rendered / elapsed.num_milliseconds();
            self.frames_rendered = 0;
        }
        self.last_fps
    }
}
