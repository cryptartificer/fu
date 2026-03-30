# Log-scale histograms and value filtering

## Summary

Add three CLI flags to `fu hist`: `--log` for logarithmic bin edges, `--gt N` and `--lt N` for pre-filter data before binning. Focused on histograms only — XY charts already have `--xlim`/`--ylim`.

## CLI flags

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--log` | bool | false | Use logarithmic bin edges |
| `--gt N` | f64 | none | Exclude values <= N |
| `--lt N` | f64 | none | Exclude values >= N |

Flags are parsed for all commands but only acted on by `hist`. Silently ignored for other commands.

Examples:
```
fu hist --log data.tsv                  # log-scale bins
fu hist --gt 0 --lt 10000 data.tsv      # filter then linear bins
fu hist --log --gt 0.1 --lt 1e6 data.tsv  # filter then log bins
```

## Data pipeline

Current:
```
read_hist_input -> Vec<f64> -> bin_values(values, nbins) -> BarData -> render_barplot
```

New:
```
read_hist_input -> Vec<f64> -> filter_values(values, gt, lt)
                            -> bin_values(values, nbins)        [linear, existing]
                            OR bin_values_log(values, nbins)    [new]
                            -> BarData -> render_barplot
```

### filter_values

```rust
fn filter_values(values: Vec<f64>, gt: Option<f64>, lt: Option<f64>) -> Vec<f64>
```

Simple `retain` predicate. Strict inequality: `--gt 0` keeps values > 0, `--lt 100` keeps values < 100. Applied before binning.

### bin_values_log

```rust
pub fn bin_values_log(values: &[f64], nbins: usize) -> Result<BarData, String>
```

Algorithm:
1. Validate all values > 0 (error otherwise)
2. Take log10 of all values
3. Compute nice bin edges in log-space (reuse `nice_bin_width` on the log-transformed range)
4. Count values into bins using log-transformed comparisons
5. Convert bin edges back to real-space: 10^edge
6. Format labels with `format_compact` on the real-space edges: `[1, 3.2)`, `[3.2, 10)`, etc.

Returns `Result` because it can fail on non-positive values.

## Error handling

| Condition | Behavior |
|-----------|----------|
| `--log` with values <= 0 (after filtering) | Error: `"--log requires all values > 0 (found {v})"` |
| `--gt`/`--lt` filters eliminate all data | Existing `"no data"` error from empty values |
| `--gt`/`--lt` on non-hist commands | Silently ignored |

## Changes by file

### cli.rs
- Add `log_scale: bool`, `gt: Option<f64>`, `lt: Option<f64>` to `Options`
- Parse `--log`, `--gt`, `--lt` flags
- Update `usage()` help text
- Tests for new flag parsing

### data.rs
- Add `filter_values(values: Vec<f64>, gt: Option<f64>, lt: Option<f64>) -> Vec<f64>`
- Add `bin_values_log(values: &[f64], nbins: usize) -> Result<BarData, String>`
- Tests for filtering and log binning

### main.rs
- In `Command::Hist` arm: apply `filter_values`, then dispatch to `bin_values` or `bin_values_log` based on `opts.log_scale`

## Tests

### cli.rs tests
- `parse_log_flag` — `hist --log` sets `log_scale: true`
- `parse_gt_flag` — `hist --gt 5` sets `gt: Some(5.0)`
- `parse_lt_flag` — `hist --lt 100` sets `lt: Some(100.0)`
- `parse_gt_lt_combined` — `hist --gt 0 --lt 1000` sets both
- `parse_gt_missing_value_is_err` — `hist --gt` with no arg errors

### data.rs tests
- `filter_values_gt` — keeps only values > threshold
- `filter_values_lt` — keeps only values < threshold
- `filter_values_both` — combined gt + lt
- `filter_values_none` — no filters returns all values
- `filter_values_all_excluded` — returns empty vec
- `bin_values_log_basic` — values spanning decades produce correct bin edges and counts
- `bin_values_log_single_decade` — values within one decade still produce sensible bins
- `bin_values_log_negative_errors` — values <= 0 return error
- `bin_values_log_all_same` — all-equal values produce single bin (edge case, mirrors linear)

## Showcase

Add sections to `showcase/03-bar-hist-count.sh`:

### Section 6: Log-scale histogram — exponential distribution
Generate log-normal data (file sizes, latencies), show with `--log`. Demonstrates the core feature — skewed data that's unreadable with linear bins becomes clear with log bins.

### Section 7: Filtered histogram — zoom into range
Same normal distribution as section 2, but with `--gt 30 --lt 70` to zoom into the center. Demonstrates pre-filter narrowing.

### Section 8: Log-scale + filter combined
Log-normal data with `--log --gt 1 --lt 100000`. Shows the two features working together.

Note: `--log`, `--gt`, `--lt` are fu-only features (no uplot equivalent), so showcase sections use `show` not `compare`.

## README

Add a log-scale histogram example to the gallery with a generated PNG (same pattern as existing gallery entries).

## Non-goals

- Log-scale y-axis (count axis) for histograms
- Log-scale for line/scatter/bar charts
- `--gte`/`--lte` inclusive variants (unnecessary with float boundaries)
- `--scale` generalized axis concept
