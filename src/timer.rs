use esp_idf_svc::timer::{EspTimerService, Task};
use core::time::Duration;

pub struct Timer {
    start: Duration,
    svc: EspTimerService<Task>,
}

impl Timer
    {
    pub fn new() -> Self {
        let svc = EspTimerService::<Task>::new().unwrap();
        let start = svc.now();
        Self { start, svc }
    }
    pub fn elapsed(&mut self) -> Duration {
        let now = self.svc.now();
        let elapsed = now - self.start;
        self.start = now;
        elapsed
    }
}