extern crate time;

pub use self::time::Duration;

pub struct FPSCounter {
    updated_at: time::PreciseTime,
    refresh_rate: time::Duration,
    render_time: i64,
    frames_rendered: i64,
    last_fps: i64,
}

impl FPSCounter {
    pub fn new(refresh_rate: time::Duration) -> FPSCounter {
        FPSCounter {
            updated_at: time::PreciseTime::now(),
            refresh_rate: refresh_rate,
            render_time: 0,
            frames_rendered: 0,
            last_fps: 0,
        }
    }
    pub fn end_frame(&mut self) {
        self.frames_rendered += 1;
        let elapsed = self.updated_at.to(time::PreciseTime::now());
        if elapsed > self.refresh_rate {
            self.updated_at = time::PreciseTime::now();
            self.render_time = elapsed.num_milliseconds() / self.frames_rendered;
            self.last_fps = 1000 / self.render_time;
            self.frames_rendered = 0;
        }
    }
    pub fn current_fps(&self) -> i64 {
        self.last_fps
    }
    pub fn render_time(&self) -> i64 {
        self.render_time
    }
}
