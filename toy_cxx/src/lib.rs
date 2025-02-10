use std::time::{Instant, Duration};
use cxx::CxxVector;
use ffi::UnplacedSlice;

#[cxx::bridge]
mod ffi {
    /// A memory buffer in need of
    /// an appropriate offset.
    struct UnplacedSlice {
        pub start:  i64,
        pub end:    i64,
        pub size:   i64,
        pub align:  i64,
    }

    extern "Rust" {
        type Clock;
        fn timer_start() -> Box<Clock>;
        fn timer_end(clk: Box<Clock>);
        fn place_slices(data: &CxxVector<UnplacedSlice>) -> Vec<i64>;
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

use coreba::*;

/// Gatekeeper to `idealloc`.
fn place_slices(data: &CxxVector<UnplacedSlice>) -> Vec<i64> {
    // Offsets will be written here.
    let mut res = vec![0; data.len()];

    let mut dirty_jobs: JobSet = vec![];
    for (id, s) in data
        .iter()
        .enumerate() {
        dirty_jobs.push(
            Arc::new(Job {
                size:               s.size as ByteSteps,
                birth:              s.start as ByteSteps,
                death:              s.end as ByteSteps + 2,
                req_size:           s.size as ByteSteps,
                alignment:          Some(s.align as ByteSteps),
                contents:           None,
                originals_boxed:    0,
                id:                 id as u32,
            }
        ));
    }

    // Code copied from idealloc core.
    let mut evts = get_events(&dirty_jobs);
    // Increased by 1 at every first death after a birth.
    let mut num_generations = 0;
    // Helper var for increasing generations.
    let mut last_evt_was_birth = true;
    // Collects "transformed" jobs.
    let mut idealloc_inp = vec![];
    // Keeps live jobs, indexed by ID.
    let mut live: HashMap<u32, Job> = HashMap::new();
    while let Some(e) = evts.pop() {
        match e.evt_t {
            EventKind::Birth    => {
                if !last_evt_was_birth {
                    last_evt_was_birth = true;
                }
                live.insert(
                    e.job.id,
                    Job {
                        size:               e.job.size,
                        birth:              e.job.birth + num_generations,
                        death:              e.job.death + num_generations,
                        req_size:           e.job.size,
                        alignment:          None,
                        contents:           None,
                        originals_boxed:    0,
                        id:                 e.job.id,
                    }
                );
            },
            EventKind::Death    => {
                if last_evt_was_birth {
                    num_generations += 1;
                    last_evt_was_birth = false;
                };
                idealloc_inp.push(live.remove(&e.job.id).unwrap());
            },
        }
    };

    // In theory, we're ready.
    let (reg, _makespan) = coreba::algo::idealloc(coreba::jobset::init(idealloc_inp).unwrap(), 1.0, 0, 3);

    for (id, pj) in &reg {
        res[*id as usize] = pj.offset.get() as i64;
    }

    res
}