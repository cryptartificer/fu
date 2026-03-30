# Histogram performance optimization

## Summary

Eliminate per-line String allocations from the histogram pipeline. Use fast-float + memchr for byte-level parsing. Two-pass architecture: scan min/max, then bin. File input uses mmap (O(1) memory). Stdin buffers raw bytes (O(input_size), no per-line allocs).

## Architecture

New module `src/hist.rs` — fast histogram pipeline, called from main.rs instead of the existing read_hist_input → bin_values chain. Existing functions stay as-is (test oracle + used by other commands).

### File input path
1. mmap file via `memmap2`
2. Pass 1: memchr newlines, fast-float parse first field, track min/max (apply --gt/--lt filter)
3. Compute nice bin edges from actual min/max
4. Pass 2: same parse, increment bin counts directly
5. Return BarData

### Stdin input path
1. `stdin.read_to_end(&mut buf)` — single Vec<u8>
2. Pass 1 + Pass 2: same as file, over the byte buffer

### Shared byte-level line parser
Both paths share a `parse_hist_fast(bytes: &[u8], delimiter: u8, has_headers: bool, gt: Option<f64>, lt: Option<f64>) -> Result<BarData, String>` that:
- Uses memchr to find newlines
- Skips header line if flagged
- Skips empty lines
- Splits on delimiter, takes first field, trims ASCII whitespace
- Parses with fast-float
- Applies gt/lt filter
- Two-pass: min/max scan, then bin

Returns identical BarData to the existing pipeline.

## Error handling

Same errors as current code: "no data", "cannot parse as number: {field}", "--log requires all values > 0". Error messages show the problematic field as a lossy UTF-8 string from the byte slice.

## New dependencies

- `fast-float` — f64 parsing from &[u8]
- `memchr` — SIMD newline scanning
- `memmap2` — memory-mapped file I/O

## Testing

- Equivalence test: generate large dataset, run both old and new paths, assert BarData identical
- Equivalence test for --log path
- fast-float vs stdlib parse equivalence on edge-case floats
- Benchmark comparison: cargo bench before/after

## Files

| File | Action |
|------|--------|
| `Cargo.toml` | Add fast-float, memchr, memmap2 deps |
| `src/hist.rs` | New: fast histogram pipeline |
| `src/lib.rs` | Add `pub mod hist` |
| `src/main.rs` | Switch Command::Hist to use hist module |
| `benches/render.rs` | Add fast-path benchmarks for comparison |
