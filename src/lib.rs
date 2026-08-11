use rand::{random_range};

pub trait Backoff {
    fn next_delay(
        &mut self,
        attempt: u64,
        previous_delay: f64,
        base: f64,
        max_delay: f64,
        cap: f64,
    ) -> f64;
}

pub struct ExponentialBackoff;

impl Backoff for ExponentialBackoff {
    fn next_delay(
        &mut self,
        attempt: u64,
        _previous_delay: f64,
        base: f64,
        _max_delay: f64,
        cap: f64,
    ) -> f64 {
        cap.min(base * 2_f64.powf(attempt as f64))
    }
}

pub struct FullJitter;

impl Backoff for FullJitter {
    fn next_delay(
        &mut self,
        attempt: u64,
        _previous_delay: f64,
        base: f64,
        _max_delay: f64,
        cap: f64,
    ) -> f64 {
        let max_delay = cap.min(base * 2_f64.powf(attempt as f64));
        random_range(0.0..=max_delay)
    }
}

pub struct EqualJitter;

impl Backoff for EqualJitter {
    fn next_delay(
        &mut self,
        attempt: u64,
        _previous_delay: f64,
        base: f64,
        _max_delay: f64,
        cap: f64,
    ) -> f64 {
        let temp = cap.min(base * 2_f64.powf(attempt as f64));
        temp / 2_f64 + random_range(0.0..=temp / 2_f64)
    }
}

pub struct DecorrelatedJitter;

impl Backoff for DecorrelatedJitter {
    fn next_delay(
        &mut self,
        _attempt: u64,
        previous_delay: f64,
        base: f64,
        _max_delay: f64,
        cap: f64,
    ) -> f64 {
        let upper = (previous_delay * 3.0).max(base);
        cap.min(random_range(base..=upper))
    }
}

pub struct BackoffReuslt {
    pub completion_time: f64,
    pub max_attempts: u64,
}

pub fn backoff_simulation<T: Backoff>(
    clients: usize,
    base: f64,
    max_delay: f64,
    cap: f64,
    strategy: &mut T,
    slot_size: f64,
) -> BackoffReuslt {
    let mut next_attempt: Vec<u64> = vec![0; clients];
    let mut attempts: Vec<u64> = vec![0; clients];
    let mut previous_delay: Vec<f64> = vec![0.0; clients];
    let mut served: Vec<bool> = vec![false; clients];

    let mut served_count = 0;
    let mut t = 0;

    while served_count < clients {
        let ready: Vec<usize> = (0..clients)
            .filter(|&i| !served[i] && next_attempt[i] <= t)
            .collect();
        if !ready.is_empty() {
            let chosen = ready[0];
            served[chosen] = true;
            served_count += 1;

            for &i in &ready {
                if i == chosen {
                    continue;
                }

                attempts[i] += 1;
                let delay =
                    strategy.next_delay(attempts[i], previous_delay[i], base, max_delay, cap);
                previous_delay[i] = delay;

                let delay_seconds = (delay / slot_size).ceil().max(1.0);
                next_attempt[i] = t + delay_seconds as u64;
            }
        }
        t += 1;
    }
    BackoffReuslt {
        completion_time: t.saturating_sub(1) as f64,
        max_attempts: *attempts.iter().max().unwrap_or(&0),
    }
}

pub fn run_simulation<T: Backoff>(
    clients: usize,
    base: f64,
    cap: f64,
    tries: u64,
    max_delay: f64,
    slot_size: f64,
    strategy: &mut T,
) -> Vec<BackoffReuslt> {
    (0..tries)
        .map(|_|{ backoff_simulation(clients, base, max_delay, cap, strategy, slot_size)})
        .collect()
}
