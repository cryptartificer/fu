#!/usr/bin/env python3
"""Render fu output to PNG by drawing braille dots directly."""

import subprocess, sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

FU = Path(__file__).resolve().parent.parent / "target" / "release" / "fu"
IMG_DIR = Path(__file__).resolve().parent

BG = (13, 17, 23)
FG = (230, 237, 243)
FONT_PATH = "/System/Library/Fonts/Menlo.ttc"
FONT_SIZE = 16
CELL_W = 10  # pixels per character cell
CELL_H = 20  # pixels per character cell
DOT_S = 2  # dot size (square side length)
PAD = 20

# Braille dot positions within a 2x4 grid mapped to cell pixels
# Bits: 0=col0row0, 1=col0row1, 2=col0row2, 3=col1row0,
#       4=col1row1, 5=col1row2, 6=col0row3, 7=col1row3
DOT_BITS = [
    (0, 0, 0),  # bit 0: left, row 0
    (0, 1, 1),  # bit 1: left, row 1
    (0, 2, 2),  # bit 2: left, row 2
    (1, 0, 3),  # bit 3: right, row 0
    (1, 1, 4),  # bit 4: right, row 1
    (1, 2, 5),  # bit 5: right, row 2
    (0, 3, 6),  # bit 6: left, row 3
    (1, 3, 7),  # bit 7: right, row 3
]


def dot_xy(col, row, cell_x, cell_y):
    """Pixel center for a dot at grid position (col, row) in cell at (cell_x, cell_y)."""
    x = cell_x + CELL_W * 0.3 + col * CELL_W * 0.4
    y = cell_y + CELL_H * 0.12 + row * CELL_H * 0.22
    return x, y


def render(text, out_path):
    lines = text.rstrip("\n").split("\n")
    max_cols = max(len(l) for l in lines)

    img_w = int(max_cols * CELL_W + PAD * 2)
    img_h = int(len(lines) * CELL_H + PAD * 2)

    img = Image.new("RGB", (img_w, img_h), BG)
    draw = ImageDraw.Draw(img)
    font = ImageFont.truetype(FONT_PATH, FONT_SIZE)

    for row_idx, line in enumerate(lines):
        cy = PAD + row_idx * CELL_H
        col = 0
        for ch in line:
            cx = PAD + col * CELL_W
            cp = ord(ch)

            if 0x2800 <= cp <= 0x28FF:
                bits = cp - 0x2800
                for bcol, brow, bit in DOT_BITS:
                    if bits & (1 << bit):
                        x, y = dot_xy(bcol, brow, cx, cy)
                        draw.rectangle(
                            [x, y, x + DOT_S, y + DOT_S],
                            fill=FG,
                        )
            else:
                draw.text((cx, cy), ch, font=font, fill=FG)
            col += 1

    img.save(out_path, optimize=True)
    print(f"  {out_path.name}: {img_w}x{img_h}")


def run_fu(data_cmd, fu_args):
    p = subprocess.run(
        f"{data_cmd} | {FU} {fu_args}",
        shell=True, capture_output=True, text=True,
    )
    return p.stderr or p.stdout


charts = [
    ("sine",
     "python3 -c \"import math; [print(f'{i*math.pi/50}\\t{math.sin(i*math.pi/50)}') for i in range(101)]\"",
     "line -t 'Sine Wave' -w 70 -h 15"),
    ("damped",
     "python3 -c \"import math\nfor i in range(200):\n    t = i * 0.1\n    print(f'{t}\\t{math.exp(-t * 0.15) * math.sin(t * 2)}')\"",
     "line -t 'Damped Oscillation' -w 70 -h 17"),
    ("random_walk",
     "python3 -c \"import random; random.seed(42); price = 100.0\nfor i in range(500):\n    price += random.gauss(0, 1.5)\n    print(f'{i}\\t{price:.2f}')\"",
     "line -t 'Random Walk (500 steps)' -w 70 -h 20"),
    ("interference",
     "seq 1 50 | awk '{print $1, sin($1*0.3)*cos($1*0.1)}' OFS=\"\\t\"",
     "line -t 'Interference' -w 50 -h 12"),
    ("scatter",
     "python3 -c \"import random; random.seed(7)\nfor _ in range(200):\n    x = random.gauss(50, 15)\n    y = random.gauss(50, 15)\n    print(f'{x:.2f}\\t{y:.2f}')\"",
     "scatter -t 'Random Cloud' -w 60 -h 16"),
    ("multi_series",
     "python3 -c \"import math\nfor h in range(24):\n    temp = 20 + 8 * math.sin((h - 6) * math.pi / 12)\n    hum = 60 - 15 * math.sin((h - 6) * math.pi / 12)\n    print(f'{h}\\t{temp:.1f}\\t{hum:.1f}')\"",
     "lines -t 'Temperature vs Humidity' -w 60 -h 14"),
    ("bar",
     "printf 'Rust\\t48200\\nGo\\t7720\\nPython\\t4518\\nC\\t3912\\nJava\\t6100\\nZig\\t1205\\nSwift\\t3500\\n'",
     "bar -t 'GitHub Stars' -w 50"),
    ("hist",
     "python3 -c \"import random; random.seed(42); [print(random.gauss(50, 15)) for _ in range(200)]\"",
     "hist -t 'Normal Distribution' -w 60 -n 12"),
    ("count",
     "python3 -c \"import random; random.seed(7); [print(random.choice(['tcp','udp','icmp','tcp','tcp','udp'])) for _ in range(100)]\"",
     "count -t 'Protocol Distribution' -w 45"),
]

print("Rendering...")
for name, data_cmd, fu_args in charts:
    text = run_fu(data_cmd, fu_args)
    if not text.strip():
        print(f"  {name}: EMPTY")
        continue
    render(text, IMG_DIR / f"{name}.png")
print("Done.")
