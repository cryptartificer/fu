#!/usr/bin/env bash
# 04 — Color, grid, axis limits, labels, transpose
# Most of these are fu-specific features; uplot comparison where applicable.
set -euo pipefail
source "$(dirname "$0")/_helpers.sh"

section "1. Explicit color — red line"

DATA_SINE="seq 1 50 | awk '{print \$1, sin(\$1*0.3)*10+20}' OFS='\t'"

compare \
    "$DATA_SINE" \
    "line -c red -C -t 'Red Sine' -w 50 -h 12" \
    "line -c red -t 'Red Sine' -w 50 -h 12"

pause

section "2. 256-color index (fu only — uplot doesn't support indexed colors)"

show "seq 1 50 | awk '...' | fu line -c 208 -C -t 'Orange (color 208)'" \
    "seq 1 50 | awk '{print \$1, cos(\$1*0.2)*15+30}' OFS='\t' \
    | $FU line -c 208 -C -t 'Orange (color 208)' -w 50 -h 12"

pause

section "3. Auto-color multi-series — 4 series"

compare \
    "python3 -c \"
import math
print('x\tsin\tcos\ttan_clip\tsawtooth')
for i in range(100):
    t = i * 0.1
    s = math.sin(t)
    c = math.cos(t)
    tc = max(-1, min(1, math.tan(t) * 0.3))
    sw = (t % 2) / 2 - 0.5
    print(f'{t:.1f}\t{s:.3f}\t{c:.3f}\t{tc:.3f}\t{sw:.3f}')
\"" \
    "lines -H -C -t 'Four Waves' -w 60 -h 14" \
    "lines -H -t 'Four Waves' -w 60 -h 14"

pause

section "4. Monochrome (fu only)"

show "python3 -c '...' | fu lines -H -M -t 'Monochrome'" \
    "python3 -c \"
import math
print('x\tsin\tcos')
for i in range(100):
    t = i * 0.1
    print(f'{t:.1f}\t{math.sin(t):.3f}\t{math.cos(t):.3f}')
\" | $FU lines -H -M -t 'Monochrome' -w 60 -h 14"

pause

section "5. Axis labels"

compare \
    "seq 1 100 | awk '{print \$1, sin(\$1*0.15)*3+sin(\$1*0.4)*1.5}' OFS='\t'" \
    "line --xlabel 'Time (s)' --ylabel 'mV' -t 'Signal' -w 50 -h 12" \
    "line --xlabel 'Time (s)' --ylabel 'mV' -t 'Signal' -w 50 -h 12"

pause

section "6. Transpose"

compare \
    "printf '1\t2\t3\t4\t5\n10\t20\t15\t30\t25\n'" \
    "line -T -t 'Transposed' -w 40 -h 10" \
    "line -T -t 'Transposed' -w 40 -h 10"

printf "\n${C_BOLD}Done.${C_RESET}\n"
