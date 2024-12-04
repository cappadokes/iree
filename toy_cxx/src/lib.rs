use std::time::{Instant, Duration};

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type Clock;
        fn timer_start() -> Box<Clock>;
        unsafe fn timer_end(clk: *mut Clock);
    }
}

/// Wraps [Instant] so as to be usable by [cxx].
struct Clock {
    heart: Instant,
}

impl Clock {
    fn new() -> Self {
        Clock {
            heart: Instant::now()
        }
    }

    /// Returns the elapsed time since the [Clock]
    /// was created in the form of a [Duration].
    fn tick(&self) -> Duration {
        self.heart
            .elapsed()
    }
}

/// Creates a new [Clock] and wraps it around a [Box],
/// so as to be passable across [cxx]'s FFI bridge.
fn timer_start() -> Box<Clock> {
    Box::new(Clock::new())
}

/// Consumes a boxed [Clock] and prints the time elapsed
/// since its creation to stdout.
unsafe fn timer_end(clk: *mut Clock) {
    let dur = clk.as_ref().unwrap().tick();
    println!("Allocation time: {} μs", dur.as_micros());
}