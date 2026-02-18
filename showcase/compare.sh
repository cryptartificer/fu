#!/usr/bin/env bash
# Comparative visual tests: fu vs uplot, side-by-side.
# Run after significant changes to verify visual parity.
#
# Usage:
#   bash showcase/compare.sh          # all tests
#   bash showcase/compare.sh 3        # single test by number
#   bash showcase/compare.sh line     # match by keyword
#
# Requires: uplot (brew install youplot)

set -euo pipefail
source "$(dirname "$0")/_helpers.sh"

if [[ -z "$UPLOT" ]]; then
    echo "ERROR: uplot not found. Install with: brew install youplot" >&2
    exit 1
fi

W=70
H=15

# ── Test registry ────────────────────────────────────────────────

tests=(
    "1:Sine wave:line"
    "2:Multi-series sin+cos:lines"
    "3:Scatter random 2D:scatter"
    "4:Bar chart:bar"
    "5:Histogram normal dist:hist"
    "6:Count word frequency:count"
    "7:Small plot:small"
    "8:Exponential growth:exp"
    "9:Negative range:neg"
    "10:Large values:large"
    "11:Bar with color:bar-color"
    "12:Bar with margin+padding:bar-layout"
    "13:Count alphabetical tiebreak:count-tie"
    "14:Histogram narrow bins:hist-narrow"
    "15:Line with margin+padding:line-layout"
)

filter="${1:-}"

should_run() {
    [[ -z "$filter" ]] && return 0
    [[ "$1" == "$filter" || "$2" == *"$filter"* || "$3" == *"$filter"* ]]
}

# ── Data generators ──────────────────────────────────────────────

gen_sine()     { python3 -c "import math; [print(f'{i*2*math.pi/100}\t{math.sin(i*2*math.pi/100)}') for i in range(101)]"; }
gen_sincos()   { python3 -c "
import math
print('x\tsin\tcos')
for i in range(101):
    t = i * 2 * math.pi / 100
    print(f'{t}\t{math.sin(t)}\t{math.cos(t)}')
"; }
gen_scatter()  { python3 -c "import random; random.seed(7); [print(f'{random.gauss(0,2)}\t{random.gauss(0,2)}') for _ in range(50)]"; }
gen_bar()      { printf 'Apple\t50\nBanana\t30\nCherry\t80\nDate\t15\nElderberry\t45\n'; }
gen_hist()     { python3 -c "import random; random.seed(42); [print(random.gauss(0,1)) for _ in range(1000)]"; }
gen_count()    { printf 'cat\ndog\ncat\nbird\ncat\ndog\nfish\ncat\ndog\nbird\ncat\ndog\ncat\ncat\n'; }
gen_exp()      { python3 -c "[print(f'{i}\t{2**i}') for i in range(20)]"; }
gen_neg()      { python3 -c "[print(f'{i}\t{-50+i*5}') for i in range(21)]"; }
gen_large()    { python3 -c "[print(f'{i}\t{i*i*1000}') for i in range(50)]"; }
gen_count_tie(){ printf 'ant\ndog\ncat\ndog\ncat\nant\nbee\nbee\nfox\nfox\n'; }
gen_hist_n()   { python3 -c "import random; random.seed(99); [print(random.gauss(5,0.5)) for _ in range(500)]"; }

# ── Runner ───────────────────────────────────────────────────────

run_pair() {
    local num="$1" name="$2" data_cmd="$3" fu_args="$4" uplot_args="$5"
    local tmp fu_out uplot_out
    tmp=$(mktemp)
    fu_out=$(mktemp)
    uplot_out=$(mktemp)

    eval "$data_cmd" > "$tmp"

    printf "\n${C_SEC}━━ %s. %s ━━${C_RESET}\n" "$num" "$name"

    # uplot
    printf "\n  ${C_MAG}uplot %s${C_RESET}\n\n" "$uplot_args"
    local t0 t1 ms
    t0=$(_now_ns)
    eval "cat '$tmp' | $UPLOT $uplot_args" 2>"$uplot_out" || true
    t1=$(_now_ns)
    cat "$uplot_out"
    ms=$(( (t1 - t0) / 1000000 ))
    printf "\n  ${C_DIM}⏱  %d ms${C_RESET}\n" "$ms"

    # fu
    printf "\n  ${C_GRN}fu %s${C_RESET}\n\n" "$fu_args"
    t0=$(_now_ns)
    eval "cat '$tmp' | $FU $fu_args" 2>"$fu_out"
    t1=$(_now_ns)
    cat "$fu_out"
    ms=$(( (t1 - t0) / 1000000 ))
    printf "\n  ${C_DIM}⏱  %d ms${C_RESET}\n" "$ms"

    rm -f "$tmp" "$fu_out" "$uplot_out"
}

# ── Tests ────────────────────────────────────────────────────────

printf "${C_BOLD}fu vs uplot — comparative visual tests${C_RESET}\n"
count=0

for entry in "${tests[@]}"; do
    IFS=: read -r num name tag <<< "$entry"
    should_run "$num" "$name" "$tag" || continue
    count=$((count + 1))

    case "$tag" in
        line)
            run_pair "$num" "$name" \
                "gen_sine" \
                "line -t 'Sine Wave' -w $W -h $H" \
                "line -t 'Sine Wave' -w $W -h $H"
            ;;
        lines)
            run_pair "$num" "$name" \
                "gen_sincos" \
                "lines -H -t 'Trig Functions' -w $W -h $H" \
                "lines -H -t 'Trig Functions' -w $W -h $H"
            ;;
        scatter)
            run_pair "$num" "$name" \
                "gen_scatter" \
                "scatter -t 'Random 2D' -w $W -h $H" \
                "scatter -t 'Random 2D' -w $W -h $H"
            ;;
        bar)
            run_pair "$num" "$name" \
                "gen_bar" \
                "bar -t 'Fruit Sales' -w 50" \
                "bar -t 'Fruit Sales' -w 50"
            ;;
        hist)
            run_pair "$num" "$name" \
                "gen_hist" \
                "hist -t 'Normal Dist' -w 50 -n 20" \
                "hist -t 'Normal Dist' -w 50 -n 20"
            ;;
        count)
            run_pair "$num" "$name" \
                "gen_count" \
                "count -t 'Animals' -w 40" \
                "count -t 'Animals' -w 40"
            ;;
        small)
            run_pair "$num" "$name" \
                "gen_sine" \
                "line -t 'Small' -w 30 -h 5" \
                "line -t 'Small' -w 30 -h 5"
            ;;
        exp)
            run_pair "$num" "$name" \
                "gen_exp" \
                "line -t 'Exponential' -w 60 -h 10" \
                "line -t 'Exponential' -w 60 -h 10"
            ;;
        neg)
            run_pair "$num" "$name" \
                "gen_neg" \
                "line -t 'Negative Range' -w 60 -h 10" \
                "line -t 'Negative Range' -w 60 -h 10"
            ;;
        large)
            run_pair "$num" "$name" \
                "gen_large" \
                "line -t 'Large Values' -w 60 -h 10" \
                "line -t 'Large Values' -w 60 -h 10"
            ;;
        bar-color)
            run_pair "$num" "$name" \
                "gen_bar" \
                "bar -t 'Colored Bars' -w 50 -c red" \
                "bar -t 'Colored Bars' -w 50 -c red"
            ;;
        bar-layout)
            run_pair "$num" "$name" \
                "gen_bar" \
                "bar -t 'Margin+Padding' -w 50 -m 3 --padding 2" \
                "bar -t 'Margin+Padding' -w 50 -m 3 --padding 2"
            ;;
        count-tie)
            run_pair "$num" "$name" \
                "gen_count_tie" \
                "count -t 'Tied Counts' -w 40" \
                "count -t 'Tied Counts' -w 40"
            ;;
        hist-narrow)
            run_pair "$num" "$name" \
                "gen_hist_n" \
                "hist -t 'Narrow Hist' -w 50 -n 15" \
                "hist -t 'Narrow Hist' -w 50 -n 15"
            ;;
        line-layout)
            run_pair "$num" "$name" \
                "gen_sine" \
                "line -t 'Line Margin' -w 50 -h 10 -m 4 --padding 2" \
                "line -t 'Line Margin' -w 50 -h 10 -m 4 --padding 2"
            ;;
    esac
done

printf "\n${C_BOLD}Done — %d tests run.${C_RESET}\n\n" "$count"
