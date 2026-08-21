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

## Install

**Prerequisites:** You need `cargo` installed. If you don't have it on Linux or macOS, run:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Then install the binary from crates.io:

```bash
cargo install oh-my-jitter
```

Then run it directly:

```bash
oh-my-jitter --help
```

## Usage

```bash
oh-my-jitter -h
```

```text
Usage: oh-my-jitter [OPTIONS]
Options:
  -n, --clients <CLIENTS>      Number of clients competing for the resource [default: 100]
  -b, --base <BASE>            Base delay in seconds for exponential backoff [default: 0.1]
  -c, --cap <CAP>              Maximum delay cap in seconds [default: 1]
  -t, --tries <TRIES>          Number of simulation runs [default: 10]
  -s, --slot-size <SLOT_SIZE>  Time slot size in seconds [default: 1]
  -a, --algorithm <ALGORITHM>  Backoff algorithm to use [default: full-jitter] [possible values: full-jitter, equal-jitter, exponential-backoff, decorrelated-jitter]
  -h, --help                   Print help
```
``

### Options

| Flag | Short | Default | Description |
|---|---|---|---|
| `--clients` | `-n` | `100` | Number of competing clients |
| `--base` | `-b` | `0.1` | Base delay, in seconds |
| `--cap` | `-c` | `1.0` | Hard ceiling on any computed delay, in seconds |
| `--tries` | `-t` | `10` | Number of independent trials to run and average over |
| `--slot-size` | `-s` | `1.0` | Duration of one discrete time slot, in seconds |
| `--algorithm` | `-a` | `full-jitter` | One of: `exponential-backoff`, `full-jitter`, `equal-jitter`, `decorrelated-jitter` |


### Example

```bash
oh-my-jitter -n 20 -t 50000 -b 1.0 -c 4 -s 1.0 -a full-jitter
```

```
Arguments:
 Args { clients: 20, base: 1.0, cap: 4.0, tries: 50000, slot_size: 1.0, algorithm: FullJitter }
Results (50000 tries, and 20 served client):
  mean_completion_time: 20.68s
  max_completion_time: 28.00s
  p90_completion_time: 22.00s
  p95_completion_time: 23.00s
  p99_completion_time: 24.00s
Attempts Analysis:
  mean_attempts: 9.06
  max_attempts: 15
  p90_attempts: 10.00
  p95_attempts: 11.00
  p99_attempts: 12.00

```

- **completion_time**: how many time slots elapsed before every client was served.
- **attempts**: the *worst-case* client's retry count per trial (i.e. the client that
  had to retry the most times).
- **p90 / p95 / p99**: percentiles across all trials, computed via nearest-rank.
  Higher percentiles need larger `--tries` values to be meaningful — with too few
  trials they collapse toward the max.



## TODO

### More simulation scenarios (beyond retry contention)
- [ ] **Leader election** — simulate nodes racing to become leader (e.g. randomized election timeouts like Raft)
- [ ] **Distributed lock acquisition** — multiple nodes contend for a single lock

### Tooling
- [ ] `--algorithm all` to run every algorithm and print a comparison table
- [ ] `--seed` for reproducible runs (currently unseeded — results vary between runs)
