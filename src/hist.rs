use memchr::memchr_iter;

use crate::data::{self, BarData};

/// Extract the first field from a line, trimming ASCII whitespace.
fn first_field(line: &[u8], delimiter: u8) -> &[u8] {
    let field = match memchr::memchr(delimiter, line) {
        Some(pos) => &line[..pos],
        None => line,
    };
    let start = field
        .iter()
        .position(|&b| b != b' ' && b != b'\t' && b != b'\r')
        .unwrap_or(field.len());
    let end = field
        .iter()
        .rposition(|&b| b != b' ' && b != b'\t' && b != b'\r')
        .map_or(start, |p| p + 1);
    &field[start..end]
}

/// Fast histogram pipeline: byte-level parsing, no per-line String allocation.
/// Produces identical output to the existing read_hist_input → filter → bin pipeline.
pub fn hist_from_bytes(
    bytes: &[u8],
    delimiter: u8,
    has_headers: bool,
    gt: Option<f64>,
    lt: Option<f64>,
    log_scale: bool,
    nbins: usize,
) -> Result<BarData, String> {
    let mut line_starts: Vec<usize> = vec![0];
    for pos in memchr_iter(b'\n', bytes) {
        line_starts.push(pos + 1);
    }

    let mut start_idx = 0;
    if has_headers {
        if line_starts.len() <= 1 && bytes.is_empty() {
            return Err("no data (expected header row)".to_string());
        }
        start_idx = 1;
    }

    let mut values = Vec::new();
    for i in start_idx..line_starts.len() {
        let line_start = line_starts[i];
        let line_end = if i + 1 < line_starts.len() {
            line_starts[i + 1] - 1
        } else {
            bytes.len()
        };
        if line_start >= line_end {
            continue;
        }
        let line = &bytes[line_start..line_end];
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() {
            continue;
        }

        let field = first_field(line, delimiter);
        if field.is_empty() {
            continue;
        }

        let v: f64 = fast_float::parse(field).map_err(|_| {
            format!(
                "cannot parse as number: {:?}",
                String::from_utf8_lossy(field)
            )
        })?;

        if let Some(lo) = gt && v <= lo {
            continue;
        }
        if let Some(hi) = lt && v >= hi {
            continue;
        }

        values.push(v);
    }

    if values.is_empty() {
        return Err("no data".to_string());
    }

    if log_scale {
        data::bin_values_log(&values, nbins)
    } else {
        Ok(data::bin_values(&values, nbins))
    }
}

/// Read from a file using mmap and run the fast histogram pipeline.
pub fn hist_from_file(
    path: &str,
    delimiter: u8,
    has_headers: bool,
    gt: Option<f64>,
    lt: Option<f64>,
    log_scale: bool,
    nbins: usize,
) -> Result<BarData, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
    let mmap = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| format!("{path}: mmap: {e}"))?;
    hist_from_bytes(&mmap, delimiter, has_headers, gt, lt, log_scale, nbins)
}

/// Read from stdin into a byte buffer and run the fast histogram pipeline.
pub fn hist_from_stdin(
    delimiter: u8,
    has_headers: bool,
    gt: Option<f64>,
    lt: Option<f64>,
    log_scale: bool,
    nbins: usize,
) -> Result<BarData, String> {
    use std::io::Read;
    let mut buf = Vec::new();
    std::io::stdin()
        .lock()
        .read_to_end(&mut buf)
        .map_err(|e| e.to_string())?;
    hist_from_bytes(&buf, delimiter, has_headers, gt, lt, log_scale, nbins)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;

    fn generate_test_data(n: usize) -> String {
        let mut state: u64 = 42;
        let mut lines = Vec::with_capacity(n);
        for _ in 0..n {
            let mut sum: f64 = 0.0;
            for _ in 0..6 {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                sum += (state >> 33) as f64 / (1u64 << 31) as f64;
            }
            let val = (sum - 3.0) * 15.0 + 50.0;
            lines.push(format!("{val:.6}"));
        }
        lines.join("\n")
    }

    fn old_pipeline(text: &str, nbins: usize) -> BarData {
        let lines: Vec<String> = text.lines().map(String::from).collect();
        let mut values = Vec::new();
        for line in lines.iter().filter(|l| !l.is_empty()) {
            let field = line.split('\t').next().unwrap_or("").trim();
            values.push(field.parse::<f64>().unwrap());
        }
        data::bin_values(&values, nbins)
    }

    fn old_pipeline_log(text: &str, nbins: usize) -> BarData {
        let lines: Vec<String> = text.lines().map(String::from).collect();
        let mut values = Vec::new();
        for line in lines.iter().filter(|l| !l.is_empty()) {
            let field = line.split('\t').next().unwrap_or("").trim();
            values.push(field.parse::<f64>().unwrap());
        }
        data::bin_values_log(&values, nbins).unwrap()
    }

    fn old_pipeline_filtered(
        text: &str,
        nbins: usize,
        gt: Option<f64>,
        lt: Option<f64>,
    ) -> BarData {
        let lines: Vec<String> = text.lines().map(String::from).collect();
        let mut values = Vec::new();
        for line in lines.iter().filter(|l| !l.is_empty()) {
            let field = line.split('\t').next().unwrap_or("").trim();
            values.push(field.parse::<f64>().unwrap());
        }
        let values = data::filter_values(values, gt, lt);
        data::bin_values(&values, nbins)
    }

    #[test]
    fn equivalence_linear_10k() {
        let text = generate_test_data(10_000);
        let old = old_pipeline(&text, 10);
        let new = hist_from_bytes(text.as_bytes(), b'\t', false, None, None, false, 10).unwrap();
        assert_eq!(old.labels, new.labels);
        assert_eq!(old.values, new.values);
    }

    #[test]
    fn equivalence_linear_100k() {
        let text = generate_test_data(100_000);
        let old = old_pipeline(&text, 20);
        let new = hist_from_bytes(text.as_bytes(), b'\t', false, None, None, false, 20).unwrap();
        assert_eq!(old.labels, new.labels);
        assert_eq!(old.values, new.values);
    }

    #[test]
    fn equivalence_log() {
        let mut state: u64 = 42;
        let mut lines = Vec::with_capacity(10_000);
        for _ in 0..10_000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = 10f64.powf((state >> 33) as f64 / (1u64 << 31) as f64 * 4.0);
            lines.push(format!("{val:.6}"));
        }
        let text = lines.join("\n");
        let old = old_pipeline_log(&text, 10);
        let new = hist_from_bytes(text.as_bytes(), b'\t', false, None, None, true, 10).unwrap();
        assert_eq!(old.labels, new.labels);
        assert_eq!(old.values, new.values);
    }

    #[test]
    fn equivalence_filtered() {
        let text = generate_test_data(10_000);
        let old = old_pipeline_filtered(&text, 10, Some(30.0), Some(70.0));
        let new = hist_from_bytes(
            text.as_bytes(),
            b'\t',
            false,
            Some(30.0),
            Some(70.0),
            false,
            10,
        )
        .unwrap();
        assert_eq!(old.labels, new.labels);
        assert_eq!(old.values, new.values);
    }

    #[test]
    fn equivalence_with_headers() {
        let text = format!("value\n{}", generate_test_data(1_000));
        let old = old_pipeline(&text.lines().skip(1).collect::<Vec<_>>().join("\n"), 10);
        let new = hist_from_bytes(text.as_bytes(), b'\t', true, None, None, false, 10).unwrap();
        assert_eq!(old.labels, new.labels);
        assert_eq!(old.values, new.values);
    }

    #[test]
    fn equivalence_csv_delimiter() {
        let mut state: u64 = 99;
        let mut lines = Vec::with_capacity(1_000);
        for _ in 0..1_000 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = (state >> 33) as f64 / (1u64 << 31) as f64 * 100.0;
            lines.push(format!("{val:.4},extra_field"));
        }
        let text = lines.join("\n");
        let line_strs: Vec<String> = text.lines().map(String::from).collect();
        let mut values = Vec::new();
        for line in line_strs.iter().filter(|l| !l.is_empty()) {
            let field = line.split(',').next().unwrap_or("").trim();
            values.push(field.parse::<f64>().unwrap());
        }
        let old = data::bin_values(&values, 10);
        let new = hist_from_bytes(text.as_bytes(), b',', false, None, None, false, 10).unwrap();
        assert_eq!(old.labels, new.labels);
        assert_eq!(old.values, new.values);
    }

    #[test]
    fn error_on_empty() {
        let result = hist_from_bytes(b"", b'\t', false, None, None, false, 10);
        assert!(result.is_err());
    }

    #[test]
    fn error_on_bad_number() {
        let result = hist_from_bytes(b"abc\n", b'\t', false, None, None, false, 10);
        assert!(result.is_err());
    }

    #[test]
    fn error_log_negative() {
        let result = hist_from_bytes(b"-1.0\n2.0\n", b'\t', false, None, None, true, 10);
        assert!(result.is_err());
    }

    #[test]
    fn fast_float_matches_stdlib() {
        let cases = [
            "0",
            "1",
            "-1",
            "0.0",
            "1.5",
            "-3.14159",
            "1e10",
            "1.23e-4",
            "999999999999999",
            "0.000000001",
            "1.7976931348623157e308",
            "2.2250738585072014e-308",
            "42.0",
            "50.123456789",
        ];
        for s in &cases {
            let stdlib: f64 = s.parse().unwrap();
            let fast: f64 = fast_float::parse(s.as_bytes()).unwrap();
            assert_eq!(stdlib.to_bits(), fast.to_bits(), "mismatch on {s}");
        }
    }
}
