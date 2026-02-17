# fu

A brutally fast terminal plotting CLI — a Rust clone of [YouPlot](https://github.com/red-data-tools/YouPlot).

Reads delimited data from stdin or files. Draws charts in the terminal using Unicode braille characters. Aims for call-compatible feature parity with `uplot`, then improvements.

## Gallery

**Sine wave** — 101 data points

```
python3 -c 'import math; [print(f"{i*math.pi/50}\t{math.sin(i*math.pi/50)}") for i in range(101)]' \
| fu line -t "Sine Wave" -w 70 -h 15
```

<img src="img/sine.png" width="700" alt="Sine wave plot">

**Damped oscillation** — 200 data points, exponential decay envelope

```
python3 -c '
import math
for i in range(200):
    t = i * 0.1
    print(f"{t}\t{math.exp(-t * 0.15) * math.sin(t * 2)}")
' | fu line -t "Damped Oscillation" -w 70 -h 17
```

<img src="img/damped.png" width="700" alt="Damped oscillation plot">

**Random walk** — 500 steps, Gaussian increments

```
python3 -c '
import random; random.seed(42); price = 100.0
for i in range(500):
    price += random.gauss(0, 1.5)
    print(f"{i}\t{price:.2f}")
' | fu line -t "Random Walk (500 steps)" -w 70 -h 20
```

<img src="img/random_walk.png" width="700" alt="Random walk plot">

**Bar chart** — horizontal bars with category labels

```
printf "Rust\t48200\nGo\t7720\nPython\t4518\nC\t3912\n" | fu bar -t "GitHub Stars" -w 50
```

<img src="img/bar.png" width="500" alt="Bar chart">

**Histogram** — automatic binning of continuous data

```
python3 -c 'import random; random.seed(42); [print(random.gauss(50, 15)) for _ in range(200)]' \
| fu hist -t "Normal Distribution" -w 60 -n 12
```

<img src="img/hist.png" width="580" alt="Histogram">

**Count** — occurrence counting, sorted by frequency

```
echo -e "tcp\ntcp\nudp\ntcp\nicmp\nudp" | fu count -t "Protocols" -w 45
```

<img src="img/count.png" width="450" alt="Count chart">

## Why

YouPlot is great but it's Ruby. Every invocation pays ~200ms of startup tax before any data is touched. `fu` does the same job in single-digit milliseconds — even on 100k rows.

| Rows | fu | uplot | Speedup |
|-----:|---:|------:|--------:|
| 10k | 6ms | 180ms | **30x** |
| 100k | 15ms | 521ms | **35x** |

## Install

```
cargo install --path .
```

Or build from source:

```
git clone https://github.com/CryptArtificer/fu
cd fu
make
```

## Usage

```
fu <command> [options] [file]
```

Pipe data in or pass a file:

```
cat data.tsv | fu line -t "preview"
fu line measurements.csv -d,
```

Plots go to stderr by default, so you can insert `fu` mid-pipeline without corrupting data:

```
cat data.tsv | fu line -t "peek" | next_command
```

## Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `line` | `lineplot`, `l` | Line chart |
| `bar` | `barplot` | Horizontal bar chart |
| `hist` | `histogram` | Histogram with auto-binning |
| `count` | `c` | Count occurrences and bar chart |

## Options

```
-d DELIM      field delimiter (default: tab)
-H            input has header row
-T            transpose rows and columns
-t TITLE      title above plot
-w WIDTH      plot width in characters (default: terminal width)
-h HEIGHT     plot height in rows (default: terminal height)
-o [FILE]     output to file or stdout (default: stderr)
-n BINS       number of histogram bins (default: 10)
--xlabel      x-axis label
--ylabel      y-axis label
```

## Roadmap

- [x] Line chart with braille canvas
- [x] Bar chart, histogram, count
- [x] Terminal size auto-detect
- [x] Transpose, axis labels
- [ ] Scatter, density, boxplot
- [ ] Multi-series, color, legend
- [ ] Canvas types (block, ascii, density)
- [ ] Tail mode — live-updating charts from streaming data
- [ ] SVG output mode
- [ ] Full YouPlot CLI compatibility

## License

[MIT](LICENSE)
