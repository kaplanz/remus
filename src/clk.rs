use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
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
pub struct Clock {
    dx: Duration,
    go: Arc<AtomicBool>,
    rx: Receiver<()>,
}

impl Clock {
    /// Constructs a `Clock` that ticks at the provided frequency.
    #[must_use]
    pub fn with_freq(freq: u32) -> Self {
        // Calculate this frequency's corresponding duration.
        let dx = Self::to_period(freq);
        // Start the run-thread
        Self::start(dx)
    }

    /// Constructs a `Clock` whose ticks last the provided duration.
    #[must_use]
    pub fn with_period(period: Duration) -> Self {
        // Start the run-thread
        Self::start(period)
    }

    /// Spins up a run-thread for execution.
    fn start(dx: Duration) -> Self {
        // Create a receiver/sender pair for transmitting clock ticks
        let (tx, rx) = mpsc::channel();
        // Create an atomic bool as the enable signal
        let go = Arc::new(AtomicBool::new(true));

        // Spin up the run-thread
        {
            let go = go.clone();
            thread::spawn(move || {
                Self::run(dx, &go, &tx);
            });
        }

        // Return the constructed clock
        Clock { dx, go, rx }
    }

    /// Gets this [`Clock`]'s period.
    #[must_use]
    pub fn period(&self) -> Duration {
        self.dx
    }

    /// Gets this [`Clock`]'s frequency.
    #[must_use]
    pub fn freq(&self) -> u32 {
        Self::to_freq(self.dx)
    }

    /// Converts a frequency into a period.
    fn to_period(freq: u32) -> Duration {
        Duration::from_secs_f64(f64::from(freq).recip())
    }

    /// Converts a period into a frequency.
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn to_freq(period: Duration) -> u32 {
        period
            .as_secs_f64()
            .recip()
            .round()
            .rem_euclid(f64::from(u32::MAX)) as u32
    }

    /// Pauses the clock, preventing iterations from progressing.
    ///
    /// # Note
    ///
    /// Does nothing if the clock is already paused. Upon being paused, cycles
    /// already clocked-in by the run-thread will still run.
    pub fn pause(&mut self) {
        self.go.store(false, Ordering::Release);
    }

    /// Resumes the clock, iterating at the previous frequency.
    ///
    /// # Note
    ///
    /// Does nothing if the clock is already running.
    pub fn resume(&mut self) {
        self.go.store(true, Ordering::Release);
    }

    /// Main function of a run-thread.
    ///
    /// Continually sends clock ticks at the provided frequency.
    fn run(dx: Duration, go: &Arc<AtomicBool>, tx: &Sender<()>) {
        // Keep track of fractional missed cycles
        let mut rem = 0;

        loop {
            // Loop until paused externally
            while go.load(Ordering::Acquire) {
                // Check the time before going to sleep
                // NOTE: Due to OS scheduling, the call to `thread::sleep()` may
                //       last longer than the specified duration. Because of this,
                //       we must record how many cycles were missed.
                let now = Instant::now();
                // Sleep for the specified duration
                thread::sleep(dx);
                // Calculate how many cycles were slept through
                let cycles = {
                    // Get elapsed (with remainder), duration in nanoseconds
                    let now = now.elapsed().as_nanos() + rem;
                    let per = dx.as_nanos();
                    // Calculate elapsed cycle remainder
                    rem = now % per;
                    // Calculate elapsed complete cycles
                    now / per
                };
                // Clock in elapsed cycles. Run until failure (usually caused by the
                // receiver hanging up).
                if (0..cycles).any(|_| tx.send(()).is_err()) {
                    // error encountered, pause the clock
                    go.store(false, Ordering::Release);
                    break;
                }
            }

            // Yield, since this thread has nothing to do
            thread::yield_now();
        }
    }
}

impl Iterator for Clock {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        self.rx.recv().ok()
    }
}
