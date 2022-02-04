//! Clock signal generator.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

/// Create a [`Clock`] that "ticks" at the provided frequency.
pub fn with_freq(freq: u32) -> Clock {
    let dur = Duration::from_secs_f64((freq as f64).recip());
    with_period(dur)
}

/// Create a [`Clock`] whose "ticks" last the provided duration.
pub fn with_period(dur: Duration) -> Clock {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        Clock::run(dur, tx);
    });

    Clock(rx)
}

/// [`Clock`] is an [`Iterator`] yielded by the free functions [`with_freq()`]
/// and [`with_period()`].
///
/// It handles the internal logic of keeping track of the "ticks" of the clock
/// signal.
///
/// NOTE: As [`Clock`] internally uses the host machine's
///       [`sleep`](thread::sleep) functionality, the host OS may elect to sleep
///       for longer than the specified duration. To combad this, upon waking
///       from sleep, the [`Clock`] will check how long it has been sleeping,
///       and "tick" accordingly to how many cycles have been missed.
#[derive(Debug)]
pub struct Clock(Receiver<()>);

impl Clock {
    fn run(dur: Duration, tx: Sender<()>) {
        // Keep track of how many cycles we missed while sleeping
        let mut missed = 0;

        // Each iteration, clock in every missed cycle. Run until failure
        // (usually caused by the receiver hanging up).
        while (0..missed).all(|_| tx.send(()).is_ok()) {
            // Check the time before going to sleep
            // NOTE: Due to OS scheduling the call to `thread::sleep()` may last
            //       longer than the specified duration. Because of this, we
            //       must record how many cycles were missed.
            let now = Instant::now();
            // Sleep for the specified duration
            thread::sleep(dur);
            // Calculate how many cycles were missed since we went to sleep
            missed = now
                .elapsed()
                .as_nanos()
                .checked_div(dur.as_nanos())
                .unwrap_or_default();
        }
    }
}

impl Iterator for Clock {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.0.recv().ok()
    }
}
