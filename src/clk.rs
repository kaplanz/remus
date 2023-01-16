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
        // Keep track of fractional missed cycles
        let mut rem = 0;

        loop {
            // Check the time before going to sleep
            // NOTE: Due to OS scheduling, the call to `thread::sleep()` may
            //       last longer than the specified duration. Because of this,
            //       we must record how many cycles were missed.
            let now = Instant::now();
            // Sleep for the specified duration
            thread::sleep(dur);
            // Calculate how many cycles were slept through
            let cycles = {
                // Get elapsed (with remainder), duration in nanoseconds
                let now = now.elapsed().as_nanos() + rem;
                let dur = dur.as_nanos();
                // Calculate elapsed cycle remainder
                rem = now % dur;
                // Calculate elapsed complete cycles
                now / dur
            };
            // Clock in elapsed cycles. Run until failure (usually caused by the
            // receiver hanging up).
            if (0..cycles).any(|_| tx.send(()).is_err()) {
                break;
            }
        }
    }
}

impl Iterator for Clock {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.0.recv().ok()
    }
}
