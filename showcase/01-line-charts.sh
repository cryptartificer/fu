#!/usr/bin/env bash
# 01 — Line charts: single series, multi-series, with options
set -euo pipefail
source "$(dirname "$0")/_helpers.sh"

section "1. Sine wave — 101 points"

compare \
    "python3 -c \"import math; [print(f'{i*math.pi/50}\t{math.sin(i*math.pi/50)}') for i in range(101)]\"" \
    "line -t 'Sine Wave' -w 70 -h 15" \
    "line -t 'Sine Wave' -w 70 -h 15"

pause

section "2. Damped oscillation — 200 points, exponential decay"

compare \
    "python3 -c \"
import math
for i in range(200):
    t = i * 0.1
    print(f'{t}\t{math.exp(-t * 0.15) * math.sin(t * 2)}')
\"" \
    "line -t 'Damped Oscillation' -w 70 -h 17" \
    "line -t 'Damped Oscillation' -w 70 -h 17"

pause

section "3. Random walk — 500 steps"

compare \
    "python3 -c \"
import random; random.seed(42); price = 100.0
for i in range(500):
    price += random.gauss(0, 1.5)
    print(f'{i}\t{price:.2f}')
\"" \
    "line -t 'Random Walk (500 steps)' -w 70 -h 20" \
    "line -t 'Random Walk (500 steps)' -w 70 -h 20"

pause

section "4. Multi-series — two series with headers"

compare \
    "printf 'hour\ttemp\thumidity\n' && python3 -c \"
import math
for h in range(24):
    temp = 20 + 8 * math.sin((h - 6) * math.pi / 12)
    hum = 60 - 15 * math.sin((h - 6) * math.pi / 12)
    print(f'{h}\t{temp:.1f}\t{hum:.1f}')
\"" \
    "lines -H -C -t 'Temperature vs Humidity' -w 60 -h 14" \
    "lines -H -t 'Temperature vs Humidity' -w 60 -h 14"

pause

section "5. Line chart with grid and y-axis limits (fu only)"

show "seq 1 30 | awk '...' | fu line --grid --ylim 0,100 -t 'Bounded with Grid'" \
    "seq 1 30 | awk '{print \$1, sin(\$1*0.3)*40+50}' OFS='\t' \
    | $FU line --grid --ylim 0,100 -t 'Bounded with Grid' -w 50 -h 12"

printf "${C_BOLD}Done.${C_RESET}\n"
