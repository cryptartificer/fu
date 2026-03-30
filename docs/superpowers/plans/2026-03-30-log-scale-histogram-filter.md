# Log-Scale Histogram & Value Filtering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `--log` (logarithmic bin edges), `--gt N`, and `--lt N` (pre-filter) flags to `fu hist`.

**Architecture:** Three new fields on `Options`, a `filter_values` helper, a `bin_values_log` function that bins in log10-space, and dispatch in `main.rs`. Tests are inline `#[cfg(test)]` per module. Showcase sections use `show` (fu-only, no uplot comparison).

**Tech Stack:** Rust, cargo test, bash showcase scripts

**Spec:** `docs/superpowers/specs/2026-03-30-log-scale-histogram-filter-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `src/cli.rs` | Modify | Parse `--log`, `--gt`, `--lt`; add to `Options` and `usage()` |
| `src/data.rs` | Modify | `filter_values`, `bin_values_log` |
| `src/main.rs` | Modify | Wire filter + log dispatch in `Command::Hist` arm |
| `showcase/03-bar-hist-count.sh` | Modify | Add sections 6-8 for new features |
| `README.md` | Modify | Add log-scale histogram gallery entry |
| `img/render.py` | Modify | Add log-scale histogram render command |

---

### Task 1: CLI flag parsing (`--log`, `--gt`, `--lt`)

**Files:**
- Modify: `src/cli.rs:83-131` (Options struct + Default impl)
- Modify: `src/cli.rs:137-261` (parse_from)
- Modify: `src/cli.rs:287-323` (usage)
- Test: `src/cli.rs:344+` (inline tests)

- [ ] **Step 1: Write failing tests for new flags**

Add these tests at the end of the `mod tests` block in `src/cli.rs`:

```rust
#[test]
fn parse_log_flag() {
    let opts = parse_from(args("hist --log")).unwrap();
    assert_eq!(opts.command, Command::Hist);
    assert!(opts.log_scale);
}

#[test]
fn parse_gt_flag() {
    let opts = parse_from(args("hist --gt 5")).unwrap();
    assert_eq!(opts.gt, Some(5.0));
}

#[test]
fn parse_lt_flag() {
    let opts = parse_from(args("hist --lt 100")).unwrap();
    assert_eq!(opts.lt, Some(100.0));
}

#[test]
fn parse_gt_lt_combined() {
    let opts = parse_from(args("hist --gt 0 --lt 1000")).unwrap();
    assert_eq!(opts.gt, Some(0.0));
    assert_eq!(opts.lt, Some(1000.0));
}

#[test]
fn parse_gt_missing_value_is_err() {
    assert!(parse_from(args("hist --gt")).is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib cli::tests -- --nocapture 2>&1 | tail -20`
Expected: compile errors — `log_scale`, `gt`, `lt` don't exist on `Options` yet.

- [ ] **Step 3: Add fields to Options struct and Default**

In `src/cli.rs`, add three fields to the `Options` struct (after `padding`):

```rust
pub log_scale: bool,
pub gt: Option<f64>,
pub lt: Option<f64>,
```

And in `Default for Options` (after `padding: None,`):

```rust
log_scale: false,
gt: None,
lt: None,
```

- [ ] **Step 4: Add flag parsing in parse_from**

In `src/cli.rs` `parse_from`, add these arms inside the `match args[i].as_str()` block, before the catch-all `arg if !arg.starts_with('-')` arm:

```rust
"--log" => {
    opts.log_scale = true;
}
"--gt" => {
    i += 1;
    let val = arg_value(&args, i, "--gt")?;
    opts.gt = Some(val.parse().map_err(|_| "--gt must be a number")?);
}
"--lt" => {
    i += 1;
    let val = arg_value(&args, i, "--lt")?;
    opts.lt = Some(val.parse().map_err(|_| "--lt must be a number")?);
}
```

- [ ] **Step 5: Update usage() help text**

In `src/cli.rs` `usage()`, add these lines after the `--ylim` entry:

```
    --log               logarithmic bin edges (histogram)
    --gt N              exclude values <= N (histogram filter)
    --lt N              exclude values >= N (histogram filter)
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --lib cli::tests 2>&1 | tail -5`
Expected: all cli tests pass (existing + 5 new).

- [ ] **Step 7: Run full lint**

Run: `cargo fmt && cargo clippy -- -D warnings 2>&1 | tail -5`
Expected: clean.

- [ ] **Step 8: Commit**

```bash
git add src/cli.rs
git commit -m "cli: add --log, --gt, --lt flags for histogram filtering"
```

---

### Task 2: filter_values function

**Files:**
- Modify: `src/data.rs` (add function + tests)

- [ ] **Step 1: Write failing tests**

Add these tests at the end of the `mod tests` block in `src/data.rs`:

```rust
#[test]
fn filter_values_gt() {
    let vals = vec![1.0, 5.0, 10.0, 15.0, 20.0];
    let out = filter_values(vals, Some(5.0), None);
    assert_eq!(out, vec![10.0, 15.0, 20.0]);
}

#[test]
fn filter_values_lt() {
    let vals = vec![1.0, 5.0, 10.0, 15.0, 20.0];
    let out = filter_values(vals, None, Some(15.0));
    assert_eq!(out, vec![1.0, 5.0, 10.0]);
}

#[test]
fn filter_values_both() {
    let vals = vec![1.0, 5.0, 10.0, 15.0, 20.0];
    let out = filter_values(vals, Some(1.0), Some(20.0));
    assert_eq!(out, vec![5.0, 10.0, 15.0]);
}

#[test]
fn filter_values_none() {
    let vals = vec![1.0, 2.0, 3.0];
    let out = filter_values(vals, None, None);
    assert_eq!(out, vec![1.0, 2.0, 3.0]);
}

#[test]
fn filter_values_all_excluded() {
    let vals = vec![1.0, 2.0, 3.0];
    let out = filter_values(vals, Some(10.0), None);
    assert!(out.is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib data::tests::filter 2>&1 | tail -10`
Expected: compile error — `filter_values` not defined.

- [ ] **Step 3: Implement filter_values**

Add this function in `src/data.rs`, after the `bin_values` function:

```rust
pub fn filter_values(mut values: Vec<f64>, gt: Option<f64>, lt: Option<f64>) -> Vec<f64> {
    if let Some(lo) = gt {
        values.retain(|&v| v > lo);
    }
    if let Some(hi) = lt {
        values.retain(|&v| v < hi);
    }
    values
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib data::tests::filter 2>&1 | tail -5`
Expected: 5 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/data.rs
git commit -m "data: add filter_values for --gt/--lt pre-filtering"
```

---

### Task 3: bin_values_log function

**Files:**
- Modify: `src/data.rs` (add function + tests)

- [ ] **Step 1: Write failing tests**

Add these tests at the end of `mod tests` in `src/data.rs`:

```rust
#[test]
fn bin_values_log_basic() {
    // Values spanning 3 decades: 1..1000
    let vals: Vec<f64> = (0..300).map(|i| 10f64.powf(i as f64 / 100.0)).collect();
    let bar = bin_values_log(&vals, 6).unwrap();
    // All values accounted for
    let total: f64 = bar.values.iter().sum();
    assert_eq!(total, 300.0);
    // Labels are [lo, hi) format with real-space values
    assert!(bar.labels[0].starts_with('['));
    assert!(bar.labels[0].contains(','));
    // Bins span the data range
    assert!(bar.labels.len() >= 2);
}

#[test]
fn bin_values_log_single_decade() {
    let vals = vec![1.0, 2.0, 3.0, 5.0, 7.0, 9.0];
    let bar = bin_values_log(&vals, 4).unwrap();
    let total: f64 = bar.values.iter().sum();
    assert_eq!(total, 6.0);
}

#[test]
fn bin_values_log_negative_errors() {
    let vals = vec![1.0, -5.0, 10.0];
    let result = bin_values_log(&vals, 5);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("--log requires all values > 0"));
}

#[test]
fn bin_values_log_zero_errors() {
    let vals = vec![0.0, 1.0, 10.0];
    let result = bin_values_log(&vals, 5);
    assert!(result.is_err());
}

#[test]
fn bin_values_log_all_same() {
    let vals = vec![100.0, 100.0, 100.0];
    let bar = bin_values_log(&vals, 5).unwrap();
    let total: f64 = bar.values.iter().sum();
    assert_eq!(total, 3.0);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib data::tests::bin_values_log 2>&1 | tail -10`
Expected: compile error — `bin_values_log` not defined.

- [ ] **Step 3: Implement bin_values_log**

Add this function in `src/data.rs`, after the `filter_values` function:

```rust
pub fn bin_values_log(values: &[f64], nbins: usize) -> Result<BarData, String> {
    // Validate all positive
    for &v in values {
        if v <= 0.0 {
            return Err(format!("--log requires all values > 0 (found {v})"));
        }
    }

    let log_vals: Vec<f64> = values.iter().map(|v| v.log10()).collect();
    let raw_min = log_vals.iter().cloned().reduce(f64::min).unwrap();
    let raw_max = log_vals.iter().cloned().reduce(f64::max).unwrap();
    let range = raw_max - raw_min;

    if range <= 0.0 {
        // All values identical — single bin in real space
        let v = values[0];
        let labels = vec![format!("[{}, {})", format_compact(v), format_compact(v * 10.0))];
        let values = vec![values.len() as f64];
        return Ok(BarData { labels, values });
    }

    let raw_width = range / nbins as f64;
    let bin_width = nice_bin_width(raw_width);
    let nice_lo = (raw_min / bin_width).floor() * bin_width;
    let nice_hi = (raw_max / bin_width).ceil() * bin_width;
    let actual_bins = ((nice_hi - nice_lo) / bin_width).round() as usize;

    let mut counts = vec![0u64; actual_bins];
    for &lv in &log_vals {
        let mut idx = ((lv - nice_lo) / bin_width) as usize;
        if idx >= actual_bins {
            idx = actual_bins - 1;
        }
        counts[idx] += 1;
    }

    let labels: Vec<String> = (0..actual_bins)
        .map(|i| {
            let lo = 10f64.powf(nice_lo + i as f64 * bin_width);
            let hi = 10f64.powf(nice_lo + (i + 1) as f64 * bin_width);
            format!("[{}, {})", format_compact(lo), format_compact(hi))
        })
        .collect();

    let values: Vec<f64> = counts.iter().map(|&c| c as f64).collect();

    Ok(BarData { labels, values })
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib data::tests::bin_values_log 2>&1 | tail -10`
Expected: 5 tests pass.

- [ ] **Step 5: Run all tests**

Run: `cargo test --lib 2>&1 | tail -5`
Expected: all 81 tests pass (71 existing + 5 filter + 5 log).

- [ ] **Step 6: Commit**

```bash
git add src/data.rs
git commit -m "data: add bin_values_log for logarithmic histogram binning"
```

---

### Task 4: Wire up main.rs

**Files:**
- Modify: `src/main.rs:102-121` (Command::Hist arm)

- [ ] **Step 1: Update the Command::Hist arm**

Replace the `Command::Hist` arm in `src/main.rs` with:

```rust
Command::Hist => {
    let values = match data::read_hist_input(&opts) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("fu: {e}");
            process::exit(1);
        }
    };
    let values = data::filter_values(values, opts.gt, opts.lt);
    if values.is_empty() {
        eprintln!("fu: no data after filtering");
        process::exit(1);
    }
    let nbins = opts.nbins.unwrap_or(10);
    let bar_data = if opts.log_scale {
        match data::bin_values_log(&values, nbins) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("fu: {e}");
                process::exit(1);
            }
        }
    } else {
        data::bin_values(&values, nbins)
    };
    plot::render_barplot(
        &bar_data,
        width,
        opts.title.as_deref(),
        &color_mode,
        &margin,
        &padding,
        '▇',
    )
}
```

- [ ] **Step 2: Run full test suite**

Run: `cargo test --lib 2>&1 | tail -5`
Expected: all 81 tests pass.

- [ ] **Step 3: Run lint**

Run: `cargo fmt && cargo clippy -- -D warnings 2>&1 | tail -5`
Expected: clean.

- [ ] **Step 4: Manual smoke test — filter**

Run: `python3 -c 'import random; random.seed(42); [print(random.gauss(50,15)) for _ in range(500)]' | cargo run -- hist --gt 30 --lt 70 -t "Filtered: 30<x<70" -w 50 2>&1`
Expected: histogram showing only the center of the distribution, labels within 30-70 range.

- [ ] **Step 5: Manual smoke test — log scale**

Run: `python3 -c 'import random; random.seed(42); [print(10**random.uniform(0,4)) for _ in range(500)]' | cargo run -- hist --log -t "Log-Scale Bins" -w 50 2>&1`
Expected: histogram with logarithmic bin edges like `[1.0, 3.2)`, `[3.2, 10.0)`, etc.

- [ ] **Step 6: Manual smoke test — combined**

Run: `python3 -c 'import random; random.seed(42); [print(10**random.uniform(-1,5)) for _ in range(500)]' | cargo run -- hist --log --gt 1 --lt 10000 -t "Log + Filter" -w 50 2>&1`
Expected: histogram with log bins, only showing values between 1 and 10000.

- [ ] **Step 7: Commit**

```bash
git add src/main.rs
git commit -m "main: wire --log, --gt, --lt into histogram pipeline"
```

---

### Task 5: Showcase sections

**Files:**
- Modify: `showcase/03-bar-hist-count.sh`

- [ ] **Step 1: Build release binary**

Run: `cargo build --release 2>&1 | tail -3`

- [ ] **Step 2: Add showcase sections 6-8**

In `showcase/03-bar-hist-count.sh`, replace the final `printf "\n${C_BOLD}Done.${C_RESET}\n"` line with:

```bash
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
```

- [ ] **Step 3: Run showcase to verify**

Run: `bash showcase/03-bar-hist-count.sh`
Expected: all 8 sections render without errors. Sections 6-8 show log bins, filtered data, and combined.

- [ ] **Step 4: Commit**

```bash
git add showcase/03-bar-hist-count.sh
git commit -m "showcase: add log-scale and filter histogram demos"
```

---

### Task 6: README gallery entry and image

**Files:**
- Modify: `README.md` (add gallery entry)
- Modify: `img/render.py` (add render command)

- [ ] **Step 1: Check current render.py chart list**

Read `img/render.py` to find the list of chart definitions and the pattern for adding a new one.

- [ ] **Step 2: Add log histogram to render.py**

Add a new entry to the charts list in `img/render.py` following the existing pattern. The data command:

```python
data_cmd = (
    "python3 -c \""
    "import random; random.seed(42)\n"
    "for _ in range(500):\n"
    "    print(10**random.uniform(1, 4))"
    "\""
)
```

The fu command: `hist --log -t "File Sizes (log bins)" -w 60 -h 15`

Output file: `log_hist.png`

- [ ] **Step 3: Generate the image**

Run: `make images`
Expected: `img/log_hist.png` created alongside existing images.

- [ ] **Step 4: Add gallery entry to README.md**

Add after the existing histogram/bar gallery entries (find the right spot by reading README.md):

````markdown
**Log-scale histogram** — 500 values across 3 decades, logarithmic bin edges

```
python3 -c '
import random; random.seed(42)
for _ in range(500):
    print(10**random.uniform(1, 4))
' | fu hist --log -t "File Sizes (log bins)" -w 60 -h 15
```

<img src="img/log_hist.png" width="650" alt="Log-scale histogram">
````

- [ ] **Step 5: Commit**

```bash
git add img/render.py img/log_hist.png README.md
git commit -m "docs: add log-scale histogram to gallery and README"
```

---

### Task 7: Final verification

- [ ] **Step 1: Full CI check**

Run: `make ci`
Expected: fmt, clippy, test all pass. 81 tests.

- [ ] **Step 2: Run full showcase**

Run: `make showcase`
Expected: all showcase scripts run cleanly.

- [ ] **Step 3: Review help text**

Run: `cargo run -- --help 2>&1`
Expected: `--log`, `--gt`, `--lt` appear in help output with descriptions.
