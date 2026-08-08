use clap::{Parser, ValueEnum};
use oh_my_jitter::{
    run_simulation,
    FullJitter,
    EqualJitter,
    ExponentialBackoff,
    DecorrelatedJitter};

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Algorithm {
    FullJitter,
    EqualJitter,
    ExponentialBackoff,
    DecorrelatedJitter,
}

#[derive(Parser, Debug)]
#[command(name = "oh-my-jitter")]
struct Args {
    #[arg(short = 'n', long, default_value_t = 100)]
    clients: usize,

    #[arg(short, long, default_value_t = 0.1)]
    base: f64,

    #[arg(short, long, default_value_t = 1.0)]
    cap: f64,

    #[arg(short, long, default_value_t = 10)]
    tries: u64,

    #[arg(short, long, default_value_t = 1.0)]
    max_delay: f64,

    #[arg(short, long, default_value_t = 1.0)]
    slot_size: f64,

    #[arg(short = 'a', long, default_value = "full-jitter")]
    algorithm: Algorithm,
}

fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (percentile / 100.0 * sorted.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted.len() -1);
    sorted[index]
}
fn main() {
    let args = Args::parse();

    println!("Arguments: ");
    println!(" {:?}", args);

    let results = match args.algorithm {
        Algorithm::ExponentialBackoff => run_simulation(
            args.clients,
            args.base,
            args.cap,
            args.tries,
            args.max_delay,
            args.slot_size,
            &mut ExponentialBackoff,
        ),
        Algorithm::FullJitter => run_simulation(
            args.clients,
            args.base,
            args.cap,
            args.tries,
            args.max_delay,
            args.slot_size,
            &mut FullJitter,
        ),
        Algorithm::EqualJitter => run_simulation(
            args.clients,
            args.base,
            args.cap,
            args.tries,
            args.max_delay,
            args.slot_size,
            &mut EqualJitter,
        ),
        Algorithm::DecorrelatedJitter => run_simulation(
            args.clients,
            args.base,
            args.cap,
            args.tries,
            args.max_delay,
            args.slot_size,
            &mut DecorrelatedJitter,
        ),
    };

let n:f64 = results.len() as f64;
let mean_completion_time = results.iter().map(|r| r.completion_time).sum::<f64>() / n;
let max_completion_time = results.iter().map(|r| r.completion_time).fold(f64::MIN, f64::max);
let mean_attempts = results.iter().map(|r| r.max_attempts).sum::<u64>() as f64 / n;
let max_attempts = results.iter().map(|r| r.max_attempts).max().unwrap_or(0);

let mut completion_sorted: Vec<f64> = results.iter().map(|r| r.completion_time).collect();
completion_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
let p90_completion_time =  percentile(&completion_sorted, 90.0);
    let p95_completion_time = percentile(&completion_sorted, 95.0);
    let p99_completion_time = percentile(&completion_sorted, 99.0);

let mut attempts_sorted: Vec<u64> = results.iter().map(|r| r.max_attempts).collect();
    attempts_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let attempts_f64: Vec<f64> = attempts_sorted.iter().map(|&x| x as f64).collect();
    let p90_attempts = percentile(&attempts_f64, 90.0);
    let p95_attempts = percentile(&attempts_f64, 95.0);
    let p99_attempts = percentile(&attempts_f64, 99.0);

println!("Results ({} tries):", args.tries);
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
}
