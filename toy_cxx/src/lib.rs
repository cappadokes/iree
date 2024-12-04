use std::time::{Instant, Duration};
use cxx::CxxVector;
use ffi::PlacedSlice;

#[cxx::bridge]
mod ffi {
    /// A memory buffer, annotated with
    /// its lifetime and an offset.
    struct PlacedSlice {
        pub start:  i64,
        pub end:    i64,
        pub size:   i64,
        pub offset: i64,
    }

    extern "Rust" {
        type Clock;
        fn timer_start() -> Box<Clock>;
        fn timer_end(clk: Box<Clock>);
        fn print_slices(data: &CxxVector<PlacedSlice>);
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
fn timer_end(clk: Box<Clock>) {
    println!("Allocation time: {} μs", clk.tick().as_micros());
}

/// Lists the descriptions of a group of buffers.
fn print_slices(data: &CxxVector<PlacedSlice>) {
    for (idx, s) in data.iter().enumerate() {
        println!(
            "#{}:\tstart: {}, end: {}, size: {}, offset: {}",
            idx, s.start, s.end, s.size, s.offset
        );
    }
}