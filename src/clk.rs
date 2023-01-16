use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

/// Clock signal generator.
///
/// An [`Iterator`] that ensures values are yielded on average[^1] according to
/// the [elapsed real time]. `Clock` internally handles the ilogic of keeping
/// track of the "ticks" (rising edges) of the clock signal.
///
/// [^1]: As `Clock` internally uses the host machine's [`sleep`](thread::sleep)
///       functionality, the host OS may elect to sleep for longer than the
///       specified duration. To combat this, upon waking from sleep the `Clock`
///       will check how long it has been sleeping, and tick accordingly to make
///       up missed cycles.
///
/// [elapsed real time]: https://en.wikipedia.org/wiki/Elapsed_real_time
#[derive(Debug)]
pub struct Clock(Receiver<()>);

impl Clock {
    /// Constructs a `Clock` that ticks at the provided frequency.
    #[must_use]
    pub fn with_freq(freq: u32) -> Self {
        let dur = Duration::from_secs_f64(f64::from(freq).recip());
        Self::with_period(dur)
    }

    /// Constructs a `Clock` whose ticks last the provided duration.
    #[must_use]
    pub fn with_period(dur: Duration) -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            Self::run(dur, &tx);
        });

        Clock(rx)
    }

    fn run(dur: Duration, tx: &Sender<()>) {
        // Keep track of how many cycles we missed while sleeping
        let mut missed = 0;

        // Each iteration, clock in every missed cycle. Run until failure
        // (usually caused by the receiver hanging up).
        while (0..missed).all(|_| tx.send(()).is_ok()) {
            // Check the time before going to sleep
            // NOTE: Due to OS scheduling, the call to `thread::sleep()` may
            //       last longer than the specified duration. Because of this,
            //       we must record how many cycles were missed.
            let now = Instant::now();
            // Sleep for the specified duration
            thread::sleep(dur);
            // Clock in this cycle
            missed = 1;
            // Calculate how many cycles were missed since we went to sleep
            missed += now
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
