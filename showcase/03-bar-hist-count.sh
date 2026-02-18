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

printf "\n${C_BOLD}Done.${C_RESET}\n"
