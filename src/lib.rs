use rand::{random_range};
use std::sync::atomic::{AtomicBool, Ordering};

pub trait Backoff {
    fn next_delay(
        &mut self,
        attempt: u64,
        previous_delay: f64,
        base: f64,
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
        cap: f64,
    ) -> f64 {
        let upper = (previous_delay * 3.0).max(base);
        cap.min(random_range(base..=upper))
    }
}

pub struct BackoffResult {
    pub completion_time: f64,
    pub max_attempts: u64,
    pub total_served_clients: usize,
}

pub fn backoff_simulation<T: Backoff>(
    clients: usize,
    base: f64,
    cap: f64,
    strategy: &mut T,
    slot_size: f64,
    running: &AtomicBool,
) -> Option<BackoffResult> {
    let mut next_attempt: Vec<u64> = vec![0; clients];
    let mut attempts: Vec<u64> = vec![0; clients];
    let mut previous_delay: Vec<f64> = vec![0.0; clients];
    let mut served: Vec<bool> = vec![false; clients];

    let mut served_count = 0;
    let mut t = 0;

    while served_count < clients {
        if !running.load(Ordering::SeqCst) {
            return None;
        }
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
                    strategy.next_delay(attempts[i], previous_delay[i], base, cap);
                previous_delay[i] = delay;

                let delay_seconds = (delay / slot_size).ceil().max(1.0);
                next_attempt[i] = t + delay_seconds as u64;
            }
        }
        t += 1;
    }
    Some(BackoffResult {
        completion_time: t.saturating_sub(1) as f64,
        max_attempts: *attempts.iter().max().unwrap_or(&0),
        total_served_clients: served_count,
    })
}

pub fn run_simulation<T: Backoff>(
    clients: usize,
    base: f64,
    cap: f64,
    tries: u64,
    slot_size: f64,
    strategy: &mut T,
    running: &AtomicBool
) -> Vec<BackoffResult> {
 let mut  simulation_results = Vec::with_capacity(tries as usize);

    for _ in 0..tries {
        match backoff_simulation(clients, base, cap, strategy, slot_size, running) {
            Some(result) => simulation_results.push(result),
            None => break,
        }
    }
    simulation_results
}
pub fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (pct / 100.0 * sorted.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: f64 = 0.1;
    const CAP: f64 = 1.0;

    // ExponentialBackoff tests

    #[test]
    fn exponential_backoff_attempt_zero() {
        let mut algo = ExponentialBackoff;
        let delay = algo.next_delay(0, 0.0, BASE, CAP);
        assert_eq!(delay, BASE);
    }

    #[test]
    fn exponential_backoff_attempt_one() {
        let mut algo = ExponentialBackoff;
        let delay = algo.next_delay(1, 0.0, BASE, CAP);
        assert_eq!(delay, 0.2);
    }

    #[test]
    fn exponential_backoff_respects_cap() {
        let mut algo = ExponentialBackoff;
        let delay = algo.next_delay(10, 0.0, BASE, CAP);
        assert_eq!(delay, CAP);
    }

    #[test]
    fn exponential_backoff_zero_cap() {
        let mut algo = ExponentialBackoff;
        let delay = algo.next_delay(1, 0.0, BASE, 0.0);
        assert_eq!(delay, 0.0);
    }

    #[test]
    fn exponential_backoff_grows_deterministic() {
        let mut algo = ExponentialBackoff;
        let d1 = algo.next_delay(1, 0.0, BASE, CAP);
        let d2 = algo.next_delay(2, 0.0, BASE, CAP);
        let d3 = algo.next_delay(3, 0.0, BASE, CAP);
        assert!(d1 < d2);
        assert!(d2 < d3);
    }

    #[test]
    fn exponential_backoff_base_gt_cap() {
        let mut algo = ExponentialBackoff;
        let delay = algo.next_delay(1, 0.0, 5.0, 1.0);
        assert_eq!(delay, 1.0);
    }

    // FullJitter tests

    #[test]
    fn full_jitter_attempt_zero() {
        let mut algo = FullJitter;
        for _ in 0..100 {
            let delay = algo.next_delay(0, 0.0, BASE, CAP);
            assert!(delay >= 0.0 && delay <= BASE);
        }
    }

    #[test]
    fn full_jitter_attempt_one() {
        let mut algo = FullJitter;
        for _ in 0..100 {
            let delay = algo.next_delay(1, 0.0, BASE, CAP);
            assert!(delay >= 0.0 && delay <= 0.2);
        }
    }

    #[test]
    fn full_jitter_zero_cap() {
        let mut algo = FullJitter;
        let delay = algo.next_delay(1, 0.0, BASE, 0.0);
        assert_eq!(delay, 0.0);
    }

    #[test]
    fn full_jitter_respects_cap() {
        let mut algo = FullJitter;
        for _ in 0..100 {
            let delay = algo.next_delay(10, 0.0, BASE, CAP);
            assert!(delay <= CAP);
        }
    }

    #[test]
    fn full_jitter_base_gt_cap() {
        let mut algo = FullJitter;
        for _ in 0..100 {
            let delay = algo.next_delay(1, 0.0, 5.0, 1.0);
            assert!(delay <= 1.0);
        }
    }

    // EqualJitter tests

    #[test]
    fn equal_jitter_attempt_zero() {
        let mut algo = EqualJitter;
        for _ in 0..100 {
            let delay = algo.next_delay(0, 0.0, BASE, CAP);
            assert!(delay >= BASE / 2.0 && delay <= BASE);
        }
    }

    #[test]
    fn equal_jitter_attempt_one() {
        let mut algo = EqualJitter;
        for _ in 0..100 {
            let delay = algo.next_delay(1, 0.0, BASE, CAP);
            assert!(delay >= 0.1 && delay <= 0.2);
        }
    }

    #[test]
    fn equal_jitter_zero_cap() {
        let mut algo = EqualJitter;
        let delay = algo.next_delay(1, 0.0, BASE, 0.0);
        assert_eq!(delay, 0.0);
    }

    #[test]
    fn equal_jitter_at_least_half() {
        let mut algo = EqualJitter;
        for _ in 0..100 {
            let delay = algo.next_delay(1, 0.0, BASE, CAP);
            assert!(delay >= 0.1);
        }
    }

    #[test]
    fn equal_jitter_base_gt_cap() {
        let mut algo = EqualJitter;
        for _ in 0..100 {
            let delay = algo.next_delay(1, 0.0, 5.0, 1.0);
            assert!(delay >= 0.5 && delay <= 1.0);
        }
    }

    // DecorrelatedJitter tests

    #[test]
    fn decorrelated_jitter_first_attempt_collapses() {
        let mut algo = DecorrelatedJitter;
        assert_eq!(algo.next_delay(1, 0.0, BASE, CAP), BASE);
    }

    #[test]
    fn decorrelated_jitter_uses_previous_delay() {
        let mut algo = DecorrelatedJitter;
        let prev = 0.5;
        for _ in 0..100 {
            let delay = algo.next_delay(1, prev, BASE, CAP);
            let upper = (prev * 3.0).max(BASE);
            assert!(delay >= BASE && delay <= upper.min(CAP));
        }
    }

    #[test]
    fn decorrelated_jitter_grows_with_prev_statistically() {
        let mut algo = DecorrelatedJitter;
        let n = 2000;
        let mean_low: f64 = (0..n).map(|_| algo.next_delay(1, 0.1, BASE, CAP)).sum::<f64>() / n as f64;
        let mean_high: f64 = (0..n).map(|_| algo.next_delay(1, 0.5, BASE, CAP)).sum::<f64>() / n as f64;
        assert!(mean_high > mean_low);
    }

    #[test]
    fn decorrelated_jitter_base_gt_cap() {
        let mut algo = DecorrelatedJitter;
        for _ in 0..100 {
            let delay = algo.next_delay(1, 0.5, 5.0, 1.0);
            assert!(delay <= 1.0);
        }
    }

    // percentile tests

    #[test]
    fn percentile_empty() {
        assert_eq!(percentile(&[], 90.0), 0.0);
    }

    #[test]
    fn percentile_single_element() {
        assert_eq!(percentile(&[5.0], 50.0), 5.0);
    }

    #[test]
    fn percentile_p50() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 50.0), 3.0);
    }

    #[test]
    fn percentile_p90() {
        let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        assert_eq!(percentile(&data, 90.0), 90.0);
    }

    #[test]
    fn percentile_p100() {
        let data = vec![1.0, 2.0, 3.0];
        assert_eq!(percentile(&data, 100.0), 3.0);
    }

    #[test]
    fn percentile_p0() {
        let data = vec![1.0, 2.0, 3.0];
        assert_eq!(percentile(&data, 0.0), 1.0);
    }

    // Simulation tests

    #[test]
    fn simulation_single_client() {
        let mut algo = FullJitter;
        let running = AtomicBool::new(true);
        let result = backoff_simulation(1, BASE, CAP, &mut algo, 1.0, &running).unwrap();
        assert_eq!(result.total_served_clients, 1);
        assert_eq!(result.completion_time, 0.0);
        assert_eq!(result.max_attempts, 0);
    }

    #[test]
    fn simulation_multiple_clients_contention() {
        let mut algo = FullJitter;
        let running = AtomicBool::new(true);
        let result = backoff_simulation(10, BASE, CAP, &mut algo, 1.0, &running).unwrap();
        assert_eq!(result.total_served_clients, 10);
        assert!(result.completion_time > 0.0);
    }

    #[test]
    fn run_simulation_produces_correct_count() {
        let mut algo = FullJitter;
        let running = AtomicBool::new(true);
        let results = run_simulation(5, BASE, CAP, 10, 1.0, &mut algo, &running);
        assert_eq!(results.len(), 10);
        for r in &results {
            assert_eq!(r.total_served_clients, 5);
        }
    }

    #[test]
    fn simulation_all_algorithms_complete() {
        let running = AtomicBool::new(true);

        let mut fb = FullJitter;
        let result = backoff_simulation(20, BASE, CAP, &mut fb, 1.0, &running).unwrap();
        assert_eq!(result.total_served_clients, 20);

        let mut ej = EqualJitter;
        let result = backoff_simulation(20, BASE, CAP, &mut ej, 1.0, &running).unwrap();
        assert_eq!(result.total_served_clients, 20);

        let mut eb = ExponentialBackoff;
        let result = backoff_simulation(20, BASE, CAP, &mut eb, 1.0, &running).unwrap();
        assert_eq!(result.total_served_clients, 20);

        let mut dj = DecorrelatedJitter;
        let result = backoff_simulation(20, BASE, CAP, &mut dj, 1.0, &running).unwrap();
        assert_eq!(result.total_served_clients, 20);
    }

    #[test]
    fn simulation_higher_cap_takes_longer_statistically() {
        let n = 30;
        let running = AtomicBool::new(true);
        let r1_avg: f64 = (0..n)
            .map(|_| backoff_simulation(10, BASE, 0.5, &mut FullJitter, 1.0, &running).unwrap().completion_time)
            .sum::<f64>() / n as f64;
        let r2_avg: f64 = (0..n)
            .map(|_| backoff_simulation(10, BASE, 2.0, &mut FullJitter, 1.0, &running).unwrap().completion_time)
            .sum::<f64>() / n as f64;
        assert!(r2_avg >= r1_avg);
    }

    #[test]
    fn simulation_slot_size_affects_timing_statistically() {
        let n = 30;
        let running = AtomicBool::new(true);
        let r1_avg: f64 = (0..n)
            .map(|_| backoff_simulation(10, BASE, CAP, &mut FullJitter, 1.0, &running).unwrap().completion_time)
            .sum::<f64>() / n as f64;
        let r2_avg: f64 = (0..n)
            .map(|_| backoff_simulation(10, BASE, CAP, &mut FullJitter, 0.5, &running).unwrap().completion_time)
            .sum::<f64>() / n as f64;
        assert!(r2_avg >= r1_avg);
    }
}
