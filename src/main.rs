mod opts;

use clap::Parser;
use oh_my_jitter::{
    DecorrelatedJitter, EqualJitter, ExponentialBackoff, FullJitter, percentile, run_simulation,
};
use opts::{Algorithm, Args};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    // create a background thread to handle ctrl--c
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        let args = Args::parse();

        println!("Arguments: ");
        println!(" {:?}", args);

        let results = match args.algorithm {
            Algorithm::ExponentialBackoff => run_simulation(
                args.clients,
                args.base,
                args.cap,
                args.tries,
                args.slot_size,
                &mut ExponentialBackoff,
                &running
            ),
            Algorithm::FullJitter => run_simulation(
                args.clients,
                args.base,
                args.cap,
                args.tries,
                args.slot_size,
                &mut FullJitter,
                &running
            ),
            Algorithm::EqualJitter => run_simulation(
                args.clients,
                args.base,
                args.cap,
                args.tries,
                args.slot_size,
                &mut EqualJitter,
                &running
            ),
            Algorithm::DecorrelatedJitter => run_simulation(
                args.clients,
                args.base,
                args.cap,
                args.tries,
                args.slot_size,
                &mut DecorrelatedJitter,
                &running
            ),
        };

        let n: f64 = results.len() as f64;
        let mean_completion_time = results.iter().map(|r| r.completion_time).sum::<f64>() / n;
        let max_completion_time = results
            .iter()
            .map(|r| r.completion_time)
            .fold(f64::MIN, f64::max);
        let mean_attempts = results.iter().map(|r| r.max_attempts).sum::<u64>() as f64 / n;
        let max_attempts = results.iter().map(|r| r.max_attempts).max().unwrap_or(0);

        let mut completion_sorted: Vec<f64> = results.iter().map(|r| r.completion_time).collect();
        completion_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p90_completion_time = percentile(&completion_sorted, 90.0);
        let p95_completion_time = percentile(&completion_sorted, 95.0);
        let p99_completion_time = percentile(&completion_sorted, 99.0);

        let mut attempts_sorted: Vec<u64> = results.iter().map(|r| r.max_attempts).collect();
        attempts_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let attempts_f64: Vec<f64> = attempts_sorted.iter().map(|&x| x as f64).collect();
        let p90_attempts = percentile(&attempts_f64, 90.0);
        let p95_attempts = percentile(&attempts_f64, 95.0);
        let p99_attempts = percentile(&attempts_f64, 99.0);

        println!(
            "Results ({} tries, and {} served client):",
            results.len(),
            results[0].total_served_clients
        );
        println!("  mean_completion_time: {:.2}s", mean_completion_time);
        println!("  max_completion_time: {:.2}s", max_completion_time);
        println!("  p90_completion_time: {:.2}s", p90_completion_time);
        println!("  p95_completion_time: {:.2}s", p95_completion_time);
        println!("  p99_completion_time: {:.2}s", p99_completion_time);
        println!("Attempts Analysis: ");
        println!("  mean_attempts: {:.2}", mean_attempts);
        println!("  max_attempts: {:.2}", max_attempts);
        println!("  p90_attempts: {:.2}", p90_attempts);
        println!("  p95_attempts: {:.2}", p95_attempts);
        println!("  p99_attempts: {:.2}", p99_attempts);
        break;
    }
}
