use clap::{Parser, ValueEnum, ColorChoice};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Algorithm {
    FullJitter,
    EqualJitter,
    ExponentialBackoff,
    DecorrelatedJitter,
}

#[derive(Parser, Debug)]
#[command(name = "oh-my-jitter", color = ColorChoice::Auto)]
pub struct Args {
    /// Number of clients competing for the resource
    #[arg(short = 'n', long, default_value_t = 100)]
    pub clients: usize,

    /// Base delay in seconds
    #[arg(short, long, default_value_t = 0.1)]
    pub base: f64,

    /// Maximum delay cap in seconds
    #[arg(short, long, default_value_t = 1.0)]
    pub cap: f64,

    /// Number of simulation runs
    #[arg(short, long, default_value_t = 10)]
    pub tries: u64,

    /// Time slot size in seconds
    #[arg(short, long, default_value_t = 1.0)]
    pub slot_size: f64,

    /// Backoff algorithm to use
    #[arg(short = 'a', long, default_value = "full-jitter")]
    pub algorithm: Algorithm,
}
