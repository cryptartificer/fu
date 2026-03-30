#!/usr/bin/env bash
# 03 — Bar charts, histograms, count
set -euo pipefail
source "$(dirname "$0")/_helpers.sh"

section "1. Horizontal bar chart"

compare \
    "printf 'Rust\t48200\nGo\t7720\nPython\t4518\nC\t3912\nJava\t6100\nZig\t1205\nSwift\t3500\n'" \
    "bar -t 'GitHub Stars' -w 50" \
    "bar -t 'GitHub Stars' -w 50"

pause

section "2. Histogram — normal distribution, 12 bins"

compare \
    "python3 -c \"
import random; random.seed(42)
for _ in range(500):
    print(random.gauss(50, 15))
\"" \
    "hist -t 'Normal Distribution' -w 60 -n 12" \
    "hist -t 'Normal Distribution' -w 60 --nbins 12"

pause

section "3. Histogram — bimodal distribution, 20 bins"

compare \
    "python3 -c \"
import random; random.seed(99)
for _ in range(300):
    print(random.gauss(30, 5))
for _ in range(300):
    print(random.gauss(70, 8))
\"" \
    "hist -t 'Bimodal' -w 60 -n 20" \
    "hist -t 'Bimodal' -w 60 --nbins 20"

pause

section "4. Count — occurrence frequency"

compare \
    "python3 -c \"
import random; random.seed(7)
protocols = ['tcp', 'udp', 'icmp', 'tcp', 'tcp', 'udp']
for _ in range(200):
    print(random.choice(protocols))
\"" \
    "count -t 'Protocol Distribution' -w 50" \
    "count -t 'Protocol Distribution' -w 50"

pause

section "5. Count — word frequency"

compare \
    "echo 'the quick brown fox jumps over the lazy dog the fox the the dog quick' | tr ' ' '\n'" \
    "count -t 'Word Frequency' -w 45" \
    "count -t 'Word Frequency' -w 45"

pause

section "6. Log-scale histogram — file sizes (3 decades)"

show "fu hist --log -t 'File Sizes (log bins)' -w 55" \
    "python3 -c \"
import random; random.seed(42)
for _ in range(500):
    print(10**random.uniform(1, 4))
\" | $FU hist --log -t 'File Sizes (log bins)' -w 55 -C"

pause

section "7. Filtered histogram — zoom into center"

show "fu hist --gt 30 --lt 70 -t 'Normal (30 < x < 70)' -w 55 -n 12" \
    "python3 -c \"
import random; random.seed(42)
for _ in range(500):
    print(random.gauss(50, 15))
\" | $FU hist --gt 30 --lt 70 -t 'Normal (30 < x < 70)' -w 55 -n 12 -C"

pause

section "8. Log-scale + filter combined"

show "fu hist --log --gt 1 --lt 100000 -t 'Latencies 1-100k μs (log)' -w 55" \
    "python3 -c \"
import random; random.seed(7)
for _ in range(1000):
    print(10**random.uniform(-1, 6))
\" | $FU hist --log --gt 1 --lt 100000 -t 'Latencies 1-100k μs (log)' -w 55 -C"

printf "\n${C_BOLD}Done.${C_RESET}\n"
