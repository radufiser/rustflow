#!/usr/bin/env bash
# bench_instrument.sh — Measure the overhead of #[tracing::instrument]
#
# Sends N sequential requests to two endpoints:
#   /bench/noop          — bare handler, no span
#   /bench/instrumented  — same handler wrapped in #[instrument]
#
# Prints min / avg / max latency (μs) for each and the delta.
#
# Usage:
#   cargo run -p rustflow-server &   # start the server first
#   ./scripts/bench_instrument.sh [N]
#
# Requires: curl with -w (write-out) support (standard on macOS/Linux).

set -euo pipefail

N="${1:-1000}"
BASE="http://localhost:3000"

echo "=== #[instrument] overhead benchmark ==="
echo "Requests per endpoint: $N"
echo ""

bench() {
    local label="$1"
    local url="$2"
    local total=0
    local min=999999999
    local max=0

    # Warm up — first request often includes TCP setup
    curl -s -o /dev/null "$url"

    for ((i = 1; i <= N; i++)); do
        # time_total in microseconds (curl gives seconds with 6 decimal places)
        us=$(curl -s -o /dev/null -w '%{time_total}' "$url" | awk '{printf "%.0f", $1 * 1000000}')
        total=$((total + us))
        ((us < min)) && min=$us
        ((us > max)) && max=$us
    done

    avg=$((total / N))
    printf "%-22s  min=%5dμs  avg=%5dμs  max=%5dμs\n" "$label" "$min" "$avg" "$max"
}

bench "/bench/noop"          "$BASE/bench/noop"
bench "/bench/instrumented"  "$BASE/bench/instrumented"

echo ""
echo "The difference (avg) is the per-request cost of creating and closing"
echo "one #[instrument] span. Typically sub-microsecond on modern hardware —"
echo "well within noise for sequential curl requests."

