#!/usr/bin/env python3
"""Render fu output to PNG by drawing braille dots directly, with ANSI color support."""

import re
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

FU = Path(__file__).resolve().parent.parent / "target" / "release" / "fu"
IMG_DIR = Path(__file__).resolve().parent

BG = (13, 17, 23)
FG = (230, 237, 243)
FONT_PATH = "/System/Library/Fonts/Menlo.ttc"
FONT_SIZE = 16
CELL_W = 10
CELL_H = 20
DOT_S = 2
PAD = 20

# Braille dot positions within a 2x4 grid mapped to cell pixels
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

# Standard ANSI → RGB (dark terminal theme)
ANSI_16 = {
    30: (0, 0, 0),
    31: (205, 49, 49),
    32: (13, 188, 121),
    33: (229, 229, 16),
    34: (36, 114, 200),
    35: (188, 63, 188),
    36: (17, 168, 205),
    37: (229, 229, 229),
    90: (102, 102, 102),
    91: (241, 76, 76),
    92: (35, 209, 139),
    93: (245, 245, 67),
    94: (59, 142, 234),
    95: (214, 112, 214),
    96: (41, 184, 219),
    97: (229, 229, 229),
}

ANSI_ESC = re.compile(r"\x1b\[([0-9;]*)m")


def ansi_256_to_rgb(n):
    """Convert 256-color index to RGB."""
    if n < 16:
        code = (30 + n) if n < 8 else (90 + n - 8)
        return ANSI_16.get(code, FG)
    if n < 232:
        n -= 16
        r = (n // 36) * 51
        g = ((n % 36) // 6) * 51
        b = (n % 6) * 51
        return (r, g, b)
    gray = 8 + (n - 232) * 10
    return (gray, gray, gray)


def parse_ansi_line(line):
    """Parse a line with ANSI escapes into [(char, color), ...]."""
    result = []
    color = FG
    pos = 0
    for m in ANSI_ESC.finditer(line):
        for ch in line[pos : m.start()]:
            result.append((ch, color))
        codes = m.group(1).split(";") if m.group(1) else ["0"]
        ci = 0
        while ci < len(codes):
            try:
                code = int(codes[ci])
            except ValueError:
                ci += 1
                continue
            if code == 0:
                color = FG
            elif code == 39:
                color = FG
            elif code == 38 and ci + 2 < len(codes):
                try:
                    if int(codes[ci + 1]) == 5:
                        color = ansi_256_to_rgb(int(codes[ci + 2]))
                        ci += 2
                except ValueError:
                    pass
            elif code in ANSI_16:
                color = ANSI_16[code]
            ci += 1
        pos = m.end()
    for ch in line[pos:]:
        result.append((ch, color))
    return result


def dot_xy(col, row, cell_x, cell_y):
    """Pixel center for a dot at grid position (col, row) in cell at (cell_x, cell_y)."""
    x = cell_x + CELL_W * 0.3 + col * CELL_W * 0.4
    y = cell_y + CELL_H * 0.12 + row * CELL_H * 0.22
    return x, y


def render(text, out_path):
    lines = text.rstrip("\n").split("\n")
    parsed = [parse_ansi_line(l) for l in lines]
    max_cols = max(len(p) for p in parsed) if parsed else 0

    img_w = int(max_cols * CELL_W + PAD * 2)
    img_h = int(len(parsed) * CELL_H + PAD * 2)

    img = Image.new("RGB", (img_w, img_h), BG)
    draw = ImageDraw.Draw(img)
    font = ImageFont.truetype(FONT_PATH, FONT_SIZE)

    for row_idx, chars in enumerate(parsed):
        cy = PAD + row_idx * CELL_H
        for col_idx, (ch, color) in enumerate(chars):
            cx = PAD + col_idx * CELL_W
            cp = ord(ch)

            if 0x2800 <= cp <= 0x28FF:
                bits = cp - 0x2800
                for bcol, brow, bit in DOT_BITS:
                    if bits & (1 << bit):
                        x, y = dot_xy(bcol, brow, cx, cy)
                        draw.rectangle([x, y, x + DOT_S, y + DOT_S], fill=color)
            else:
                draw.text((cx, cy), ch, font=font, fill=color)

    img.save(out_path, optimize=True)
    print(f"  {out_path.name}: {img_w}x{img_h}")


def run_fu(data_cmd, fu_args):
    p = subprocess.run(
        f"{data_cmd} | {FU} {fu_args}",
        shell=True,
        capture_output=True,
        text=True,
    )
    return p.stderr or p.stdout


charts = [
    (
        "sine",
        'python3 -c "import math; [print(f\'{i*math.pi/50}\\t{math.sin(i*math.pi/50)}\') for i in range(101)]"',
        "line -t 'Sine Wave' -w 70 -h 15 -C -c green",
    ),
    (
        "damped",
        "python3 -c \"import math\nfor i in range(200):\n    t = i * 0.1\n    print(f'{t}\\t{math.exp(-t * 0.15) * math.sin(t * 2)}')\"",
        "line -t 'Damped Oscillation' -w 70 -h 17 -C -c cyan",
    ),
    (
        "random_walk",
        "python3 -c \"import random; random.seed(42); price = 100.0\nfor i in range(500):\n    price += random.gauss(0, 1.5)\n    print(f'{i}\\t{price:.2f}')\"",
        "line -t 'Random Walk (500 steps)' -w 70 -h 20 -C -c yellow",
    ),
    (
        "scatter",
        "python3 -c \"import random; random.seed(7)\nfor _ in range(200):\n    x = random.gauss(50, 15)\n    y = random.gauss(50, 15)\n    print(f'{x:.2f}\\t{y:.2f}')\"",
        "scatter -t 'Random Cloud' -w 60 -h 16 -C -c magenta",
    ),
    (
        "multi_series",
        "python3 -c \"import math\nprint('hour\\ttemp\\thumidity')\nfor h in range(24):\n    temp = 20 + 8 * math.sin((h - 6) * math.pi / 12)\n    hum = 60 - 15 * math.sin((h - 6) * math.pi / 12)\n    print(f'{h}\\t{temp:.1f}\\t{hum:.1f}')\"",
        "lines -H -t 'Temperature vs Humidity' -w 60 -h 14 -C",
    ),
    (
        "bar",
        "printf 'Rust\\t48200\\nGo\\t7720\\nPython\\t4518\\nC\\t3912\\nJava\\t6100\\nZig\\t1205\\nSwift\\t3500\\n'",
        "bar -t 'GitHub Stars' -w 50 -C -c cyan",
    ),
    (
        "hist",
        "python3 -c \"import random; random.seed(42); [print(random.gauss(50, 15)) for _ in range(200)]\"",
        "hist -t 'Normal Distribution' -w 60 -n 12 -C -c green",
    ),
    (
        "count",
        "python3 -c \"import random; random.seed(7); [print(random.choice(['tcp','udp','icmp','tcp','tcp','udp'])) for _ in range(100)]\"",
        "count -t 'Protocol Distribution' -w 45 -C -c yellow",
    ),
    (
        "log_hist",
        "python3 -c \"import random; random.seed(42); [print(10**random.uniform(1, 4)) for _ in range(500)]\"",
        "hist --log -t 'File Sizes (log bins)' -w 60 -h 15 -C -c green",
    ),
    (
        "multi_line",
        "python3 -c \"\nimport math\nprint('x\\tsin\\tcos')\nfor i in range(100):\n    t = i * math.pi / 25\n    print(f'{t:.3f}\\t{math.sin(t):.4f}\\t{math.cos(t):.4f}')\"",
        "lines -H -t 'sin vs cos' -w 70 -h 15 -C",
    ),
]

print("Rendering...")
for name, data_cmd, fu_args in charts:
    text = run_fu(data_cmd, fu_args)
    if not text.strip():
        print(f"  {name}: EMPTY")
        continue
    render(text, IMG_DIR / f"{name}.png")
print("Done.")
