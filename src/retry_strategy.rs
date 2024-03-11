use rand::{thread_rng, Rng};
use std::ops::Mul;
use std::time::Duration;

pub trait RetryStrategy: Send {
    fn delay(&self, attempt: usize) -> Duration;
    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool;
}

pub struct FixedDelayStrategy {
    pub delay_duration: Duration,
}

impl RetryStrategy for FixedDelayStrategy {
    fn delay(&self, _attempt: usize) -> Duration {
        self.delay_duration
    }

    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool {
        attempt < max_attempts
    }
}

pub struct ExponentialBackoffStrategy {
    pub initial_delay: Duration,
    pub max_delay: Duration,
}

impl RetryStrategy for ExponentialBackoffStrategy {
    fn delay(&self, attempt: usize) -> Duration {
        let delay = self.initial_delay.mul(2_u32.pow(attempt as u32));
        std::cmp::min(delay, self.max_delay)
    }

    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool {
        attempt < max_attempts
    }
}

pub struct LinearBackoffStrategy {
    pub initial_delay: Duration,
    pub increment: Duration,
    pub max_delay: Duration,
}

impl RetryStrategy for LinearBackoffStrategy {
    fn delay(&self, attempt: usize) -> Duration {
        let delay = if attempt == 0 {
            self.initial_delay
        } else {
            self.initial_delay + self.increment.mul(attempt as u32)
        };
        std::cmp::min(delay, self.max_delay)
    }

    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool {
        attempt < max_attempts
    }
}

pub struct RandomDelayStrategy {
    pub min_delay: Duration,
    pub max_delay: Duration,
}

impl RetryStrategy for RandomDelayStrategy {
    fn delay(&self, _attempt: usize) -> Duration {
        let mut rng = rand::thread_rng();
        let min = self.min_delay.as_millis() as u64;
        let max = self.max_delay.as_millis() as u64;
        Duration::from_millis(rng.gen_range(min..=max))
    }

    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool {
        attempt < max_attempts
    }
}

pub struct IncrementalBackoffStrategy {
    pub initial_delay: Duration,
    pub increment_step: Duration,
    pub step_increment: Duration,
    pub max_delay: Duration,
}

impl RetryStrategy for IncrementalBackoffStrategy {
    fn delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }

        let increment = self.increment_step + self.step_increment.mul(attempt as u32 - 1);
        let delay = self.initial_delay + increment.mul(attempt as u32);
        std::cmp::min(delay, self.max_delay)
    }

    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool {
        attempt < max_attempts
    }
}

pub struct FibonacciBackoffStrategy {
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl RetryStrategy for FibonacciBackoffStrategy {
    fn delay(&self, attempt: usize) -> Duration {
        let fib_number = fibonacci(attempt);
        let delay = self.base_delay.mul(fib_number as u32);
        std::cmp::min(delay, self.max_delay)
    }

    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool {
        attempt < max_attempts
    }
}

fn fibonacci(n: usize) -> usize {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub struct GeometricBackoffStrategy {
    pub initial_delay: Duration,
    pub multiplier: f64,
    pub max_delay: Duration,
}

impl RetryStrategy for GeometricBackoffStrategy {
    fn delay(&self, attempt: usize) -> Duration {
        let delay = if attempt == 0 {
            self.initial_delay
        } else {
            let calculated_delay_secs =
                self.initial_delay.as_secs_f64() * self.multiplier.powi(attempt as i32);
            Duration::from_secs_f64(calculated_delay_secs)
        };
        std::cmp::min(delay, self.max_delay)
    }

    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool {
        attempt < max_attempts
    }
}

pub struct HarmonicBackoffStrategy {
    pub initial_delay: Duration,
    pub max_delay: Duration,
}

impl RetryStrategy for HarmonicBackoffStrategy {
    fn delay(&self, attempt: usize) -> Duration {
        let harmonic_number = (1..=attempt).map(|x| 1.0 / x as f64).sum::<f64>();
        let delay = self.initial_delay.mul_f64(harmonic_number);
        std::cmp::min(delay, self.max_delay)
    }

    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool {
        attempt < max_attempts
    }
}

pub struct JitterBackoffStrategy {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

impl RetryStrategy for JitterBackoffStrategy {
    fn delay(&self, attempt: usize) -> Duration {
        let mut rng = thread_rng();
        let exp_delay = self
            .initial_delay
            .mul_f64(self.multiplier.powi(attempt as i32));
        let jitter = rng.gen_range(Duration::ZERO..exp_delay);
        let delay_with_jitter = exp_delay + jitter;
        std::cmp::min(delay_with_jitter, self.max_delay)
    }

    fn should_retry(&self, attempt: usize, max_attempts: usize) -> bool {
        attempt < max_attempts
    }
}
