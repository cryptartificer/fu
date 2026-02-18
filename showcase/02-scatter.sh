#!/usr/bin/env bash
# 02 — Scatter plots
set -euo pipefail
source "$(dirname "$0")/_helpers.sh"

section "1. Random scatter — 200 points"

compare \
    "python3 -c \"
import random; random.seed(7)
for _ in range(200):
    x = random.gauss(50, 15)
    y = random.gauss(50, 15)
    print(f'{x:.2f}\t{y:.2f}')
\"" \
    "scatter -t 'Random Cloud' -w 60 -h 16" \
    "scatter -t 'Random Cloud' -w 60 -h 16"

pause

section "2. Spiral — 300 points with explicit color (fu only)"

show "python3 -c '...' | fu scatter -c cyan -C -t 'Spiral'" \
    "python3 -c \"
import math
for i in range(300):
    t = i * 0.1
    r = t * 0.5
    print(f'{r * math.cos(t):.2f}\t{r * math.sin(t):.2f}')
\" | $FU scatter -c cyan -C -t 'Spiral' -w 50 -h 16"

pause

section "3. Multi-series scatter with headers"

compare \
    "python3 -c \"
import random; random.seed(42)
print('x\tcluster_a\tcluster_b')
for i in range(80):
    x = i
    a = random.gauss(30, 10)
    b = random.gauss(70, 10)
    print(f'{x}\t{a:.1f}\t{b:.1f}')
\"" \
    "scatter -H -C -t 'Clusters' -w 60 -h 16" \
    "scatter -H -t 'Clusters' -w 60 -h 16"

printf "\n${C_BOLD}Done.${C_RESET}\n"
