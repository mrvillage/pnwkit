#[derive(Debug, Clone)]
pub struct RateLimiter {
    limit: u32,
    remaining: u32,
    reset: u64,
    interval: u32,
    init: bool,
    now: fn() -> u64,
}

impl RateLimiter {
    fn now(&self) -> u64 {
        (self.now)()
    }

    pub fn initialized(&self) -> bool {
        self.init
    }

    pub fn initialize(&mut self, limit: u32, remaining: u32, reset: u64, interval: u32) {
        self.limit = limit;
        self.remaining = remaining;
        self.reset = reset;
        self.interval = interval;
        self.init = true;
    }

    pub fn hit(&mut self) -> u64 {
        if !self.init {
            return 0;
        }
        let current_time = self.now();
        if current_time > self.reset {
            self.remaining = self.limit - 1;
            self.reset = current_time + 1 + self.interval as u64;
            return 0;
        }
        if self.remaining == 0 {
            self.reset - current_time as u64 + 1
        } else {
            self.remaining -= 1;
            0
        }
    }

    pub fn handle_429(&mut self, reset: Option<u64>) -> u64 {
        self.remaining = 0;
        self.reset = reset.unwrap_or(if self.init {
            self.reset
        } else {
            self.now() + (if self.interval > 0 { self.interval } else { 60 }) as u64
        });
        self.reset - self.now()
    }
}
