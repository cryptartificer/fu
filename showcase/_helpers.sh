#!/usr/bin/env bash
# Shared helpers for fu showcase scripts.
# Source this file, don't run it directly.

FU="${FU:-$(dirname "$0")/../target/release/fu}"
UPLOT="${UPLOT:-$(command -v uplot 2>/dev/null || true)}"

# ── Nanosecond timer ────────────────────────────────────────────────
if command -v gdate >/dev/null 2>&1; then
    _now_ns() { gdate +%s%N; }
elif date +%s%N >/dev/null 2>&1; then
    _now_ns() { date +%s%N; }
else
    _now_ns() { python3 -c 'import time; print(int(time.time()*1e9))'; }
fi

# ── Color codes ─────────────────────────────────────────────────────
C_RESET=$'\033[0m'
C_BOLD=$'\033[1m'
C_DIM=$'\033[2m'
C_CYAN=$'\033[1;96m'
C_YEL=$'\033[93m'
C_GRN=$'\033[1;92m'
C_RED=$'\033[91m'
C_MAG=$'\033[1;35m'
C_SEC=$'\033[1;36m'

section() { printf "\n${C_SEC}━━ %s ━━${C_RESET}\n\n" "$1"; }

# Run a single command (fu-only, no comparison).
show() {
    local label="$1"; shift
    printf "  ${C_DIM}\$${C_RESET} ${C_YEL}%s${C_RESET}\n\n" "$label"
    eval "$@"
    echo ""
}

# Compare fu and uplot on the same data.
#   compare "data_cmd" "fu_args" "uplot_args"
#
# Generates data once, runs uplot first (with timing), then fu (with timing).
# If uplot is not installed, silently skips it.
compare() {
    local data_cmd="$1"
    local fu_args="$2"
    local uplot_args="$3"

    local tmp out_tmp
    tmp=$(mktemp)
    out_tmp=$(mktemp)
    eval "$data_cmd" > "$tmp"

    if [[ -n "$UPLOT" ]]; then
        printf "  ${C_MAG}uplot %s${C_RESET}\n\n" "$uplot_args"
        local t0 t1 ms
        t0=$(_now_ns)
        eval "cat '$tmp' | $UPLOT $uplot_args -C" 2>"$out_tmp"
        t1=$(_now_ns)
        cat "$out_tmp"
        ms=$(( (t1 - t0) / 1000000 ))
        printf "\n  ${C_DIM}⏱  %d ms${C_RESET}\n\n" "$ms"
    fi

    printf "  ${C_GRN}fu %s${C_RESET}\n\n" "$fu_args"
    local t0 t1 ms
    t0=$(_now_ns)
    eval "cat '$tmp' | $FU $fu_args -C" 2>"$out_tmp"
    t1=$(_now_ns)
    cat "$out_tmp"
    ms=$(( (t1 - t0) / 1000000 ))
    printf "\n  ${C_DIM}⏱  %d ms${C_RESET}\n\n" "$ms"

    rm -f "$tmp" "$out_tmp"
}

pause() {
    sleep 0.3
}
