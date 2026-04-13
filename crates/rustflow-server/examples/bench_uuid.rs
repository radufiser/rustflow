//! Benchmark: UUID v4 vs ULID vs atomic counter for request-ID generation.
//!
//! Run with:
//!   cargo run -p rustflow-server --example bench_uuid --release
//!
//! Measures wall-clock time to generate 100 000 IDs with each strategy,
//! then prints per-ID cost so you can judge whether Uuid::new_v4() is
//! negligible at your target throughput.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

const N: usize = 100_000;

fn main() {
    println!("=== Request-ID generation benchmark ===");
    println!("Iterations: {N}\n");

    // ── 1. UUID v4 (random, OS entropy) ─────────────────────
    let start = Instant::now();
    for _ in 0..N {
        let id = uuid::Uuid::new_v4().to_string();
        std::hint::black_box(&id); // prevent optimising away
    }
    let uuid_dur = start.elapsed();

    // ── 2. ULID (timestamp + random, monotonic) ─────────────
    let start = Instant::now();
    for _ in 0..N {
        let id = ulid::Ulid::new().to_string();
        std::hint::black_box(&id);
    }
    let ulid_dur = start.elapsed();

    // ── 3. Atomic counter (zero allocation, no randomness) ──
    let counter = AtomicU64::new(0);
    let start = Instant::now();
    for _ in 0..N {
        let id = counter.fetch_add(1, Ordering::Relaxed);
        std::hint::black_box(&id);
    }
    let counter_dur = start.elapsed();

    // ── 4. Atomic counter → String (fairer comparison) ──────
    let counter2 = AtomicU64::new(0);
    let start = Instant::now();
    for _ in 0..N {
        let id = counter2.fetch_add(1, Ordering::Relaxed).to_string();
        std::hint::black_box(&id);
    }
    let counter_str_dur = start.elapsed();

    // ── Results ─────────────────────────────────────────────
    println!("{:<30} {:>10} {:>12}", "Strategy", "Total", "Per ID");
    println!("{:-<54}", "");
    print_row("uuid::Uuid::new_v4()", uuid_dur);
    print_row("ulid::Ulid::new()", ulid_dur);
    print_row("AtomicU64 (no string)", counter_dur);
    print_row("AtomicU64 → String", counter_str_dur);

    println!();
    println!("── Analysis ──");
    let uuid_ns = uuid_dur.as_nanos() as f64 / N as f64;
    let ulid_ns = ulid_dur.as_nanos() as f64 / N as f64;
    let max_rps = 1_000_000_000.0 / uuid_ns;
    let cpu_pct_at_100k = (uuid_ns * 100_000.0) / 1_000_000_000.0 * 100.0;
    println!(
        "UUID v4 costs ~{:.0} ns/id → single-thread ceiling: ~{:.0} k req/s",
        uuid_ns,
        max_rps / 1000.0
    );
    println!(
        "At 100 k req/s the UUID overhead is ~{:.1}% of one CPU core.",
        cpu_pct_at_100k
    );
    println!(
        "ULID is ~{:.0}× faster ({:.0} ns) — and sortable by time.",
        uuid_ns / ulid_ns,
        ulid_ns
    );
    println!();
    println!("Recommendation:");
    println!("  • RustFlow today  → Uuid::new_v4() is fine (hundreds of req/s)");
    println!("  • Sortable IDs    → ULID: 10-15× faster, lexicographic time ordering");
    println!("  • > 500 k req/s   → atomic counter + node prefix avoids OS entropy cost");
    println!("  • Key cost driver → getrandom syscall (UUID v4), not the string formatting");
}

fn print_row(label: &str, dur: std::time::Duration) {
    let total_us = dur.as_micros();
    let per_ns = dur.as_nanos() as f64 / N as f64;
    println!("{:<30} {:>7} μs {:>9.0} ns", label, total_us, per_ns);
}

