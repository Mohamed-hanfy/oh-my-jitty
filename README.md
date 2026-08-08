# oh-my-jitter

A discrete-time simulator for comparing client **retry/backoff strategies** under
contention. Multiple clients race to be "served" one at a time; when a client loses,
it computes a delay using the chosen backoff algorithm and tries again later. The
simulator reports how long the whole group takes to finish, and how many retries the
worst-off client needed.

This is useful for building intuition about the classic "thundering herd" problem —
what happens when many clients retry a failed request at the same time — and for
comparing strategies like exponential backoff and jitter before picking one for a real
system.

## How the simulation works

- Time advances in discrete steps (slots).
- At each step, every client whose "next attempt" time has arrived is considered ready.
- Exactly **one** ready client is served per step; it's done.
- Every other ready client "fails," increments its attempt count, and computes a new
  delay via the chosen [`Backoff`](src/lib.rs) algorithm.
- The simulation ends once every client has been served.

This models a resource with capacity 1 per time slot (e.g. a single-threaded server, a
lock, a rate-limited endpoint) and many competing clients retrying against it.

## Algorithms implemented

| Algorithm | Formula (roughly) | Notes |
|---|---|---|
| `exponential-backoff` | `min(cap, base * 2^attempt)` | No randomness — all losers retry in lockstep, which is exactly the thundering-herd problem this tool is meant to illustrate. |
| `full-jitter` | `random(0, min(cap, base * 2^attempt))` | AWS's recommended strategy — delay is uniformly random up to the exponential ceiling. Default algorithm. |
| `equal-jitter` | `temp/2 + random(0, temp/2)` where `temp = min(cap, base * 2^attempt)` | Half the delay grows deterministically, half is randomized — a compromise between predictability and spread. |
| `decorrelated-jitter` | `min(cap, random(base, previous_delay * 3))` | Each delay is randomized based on the *previous* delay rather than the attempt count, so it doesn't need to track attempt number. |

See [`src/lib.rs`](src/lib.rs) for exact implementations.

> **Known issue:** `decorrelated-jitter`'s first delay uses `previous_delay = 0.0`,
> which produces an inverted range (`base..=0.0`) and will panic. Fix pending — see
> [TODO](#todo).

## Usage

```bash
cargo run -- [OPTIONS]
```

### Options

| Flag | Short | Default | Description |
|---|---|---|---|
| `--clients` | `-n` | `100` | Number of competing clients |
| `--base` | `-b` | `0.1` | Base delay, in seconds |
| `--cap` | `-c` | `1.0` | Hard ceiling on any computed delay, in seconds |
| `--tries` | `-t` | `10` | Number of independent trials to run and average over |
| `--max-delay` | `-m` | `1.0` | Starting range bound used by decorrelated jitter |
| `--slot-size` | `-s` | `1.0` | Duration of one discrete time slot, in seconds |
| `--algorithm` | `-a` | `full-jitter` | One of: `exponential-backoff`, `full-jitter`, `equal-jitter`, `decorrelated-jitter` |

Run `cargo run -- --help` for the full generated help text.

### Example

```bash
cargo run -- --clients 50 --algorithm full-jitter --tries 1000
```

```
Arguments:
 Args { clients: 50, base: 0.1, cap: 1.0, tries: 1000, max_delay: 1.0, slot_size: 1.0, algorithm: FullJitter }
Results (1000 tries):
  mean_completion_time: 12.40s
  max_completion_time: 22.00s
  p90_completion_time: 17.00s
  p95_completion_time: 19.00s
  p99_completion_time: 21.00s
Attempts Analysis:
  mean_attempts: 4.10
  max_attempts: 7.00
  p90_attempts: 6.00
  p95_attempts: 6.00
  p99_attempts: 7.00
```

- **completion_time**: how many time slots elapsed before every client was served.
- **attempts**: the *worst-case* client's retry count per trial (i.e. the client that
  had to retry the most times).
- **p90 / p95 / p99**: percentiles across all trials, computed via nearest-rank.
  Higher percentiles need larger `--tries` values to be meaningful — with too few
  trials they collapse toward the max.

## Building

```bash
cargo build --release
```

### Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
rand = "0.9"
```

## Project structure

```
src/
├── lib.rs   # Backoff trait + algorithm implementations + simulation engine
└── main.rs  # CLI argument parsing and reporting
```

## TODO

### Fix known bugs
- [ ] `decorrelated-jitter` panics on a client's first retry (`previous_delay = 0.0`
      makes the random range inverted). Seed `previous_delay` with `base` instead, or
      special-case `attempt == 0`.

### More algorithms (non-jitter)
- [ ] **Constant/fixed delay** — naive baseline, every retry waits the same `base`
      seconds
- [ ] **Linear backoff** — `delay = base * attempt`, capped at `cap`
- [ ] **Fibonacci backoff** — `delay = base * fib(attempt)`
- [ ] **Polynomial/quadratic backoff** — `delay = base * attempt^2`
- [ ] **Step/tiered backoff** — fixed delay tiers by attempt-count bucket (matches many
      real HTTP client libraries)
- [ ] **Token-bucket / rate-limited retry** — clients draw from a shared refilling
      bucket instead of computing delays independently
- [ ] **AIMD (additive-increase/multiplicative-decrease)** — delay adapts to observed
      contention, borrowed from TCP congestion control
- [ ] **Circuit breaker** — client stops retrying entirely after N consecutive
      failures, then resumes after a cooldown
- [ ] **Retry-After / server-directed backoff** — server tells the client exactly when
      to retry (as in HTTP `429`/`503` `Retry-After` headers); a useful contrast to
      client-computed algorithms

### More realistic simulation dynamics
- [ ] Heterogeneous clients — vary `base`/`cap` per client
- [ ] Server capacity > 1 — serve N clients per slot instead of 1
- [ ] Client churn — clients give up after N failed attempts
- [ ] Bursty/staggered arrivals — clients don't all start at t=0
- [ ] Probabilistic server-side failures on already-served clients

### Metrics & reporting
- [ ] Per-client attempt distributions, not just the worst client per trial
- [ ] Throughput-over-time (clients served per slot) to visualize thundering-herd spikes
- [ ] CSV/JSON export for external plotting
- [ ] Fairness metric across clients (e.g. Jain's fairness index)

### Tooling
- [ ] `--algorithm all` to run every algorithm and print a comparison table
- [ ] `--seed` for reproducible runs (currently unseeded — results vary between runs)
- [ ] Parallelize trials with `rayon` for faster large-`--tries` runs

## License

*(add your license here)*