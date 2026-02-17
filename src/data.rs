use std::fs;
use std::io::{self, BufRead};

use crate::cli::Options;

#[derive(Debug)]
pub struct DataSet {
    pub headers: Option<Vec<String>>,
    pub x: Vec<f64>,
    pub series: Vec<Vec<f64>>,
}

impl DataSet {
    pub fn x_range(&self) -> (f64, f64) {
        let min = self.x.iter().cloned().reduce(f64::min).unwrap_or(0.0);
        let max = self.x.iter().cloned().reduce(f64::max).unwrap_or(1.0);
        if (max - min).abs() < f64::EPSILON {
            (min - 1.0, max + 1.0)
        } else {
            (min, max)
        }
    }

    pub fn y_range(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for s in &self.series {
            for &v in s {
                if v < min {
                    min = v;
                }
                if v > max {
                    max = v;
                }
            }
        }
        if min == f64::INFINITY {
            return (0.0, 1.0);
        }
        if (max - min).abs() < f64::EPSILON {
            (min - 1.0, max + 1.0)
        } else {
            (min, max)
        }
    }
}

pub fn read_input(opts: &Options) -> Result<DataSet, String> {
    let lines = read_lines(opts)?;
    parse_lines(&lines, opts.delimiter, opts.has_headers)
}

fn read_lines(opts: &Options) -> Result<Vec<String>, String> {
    if opts.files.is_empty() {
        let stdin = io::stdin();
        let lines: Vec<String> = stdin
            .lock()
            .lines()
            .map(|l| l.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(lines)
    } else {
        let mut all_lines = Vec::new();
        for path in &opts.files {
            let content = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
            all_lines.extend(content.lines().map(String::from));
        }
        Ok(all_lines)
    }
}

fn parse_lines(lines: &[String], delimiter: char, has_headers: bool) -> Result<DataSet, String> {
    let mut iter = lines.iter().filter(|l| !l.is_empty());

    let headers = if has_headers {
        let header_line = iter.next().ok_or("no data (expected header row)")?;
        Some(
            header_line
                .split(delimiter)
                .map(|s| s.trim().to_string())
                .collect(),
        )
    } else {
        None
    };

    let mut rows: Vec<Vec<f64>> = Vec::new();
    for line in iter {
        let fields: Vec<f64> = line
            .split(delimiter)
            .map(|s| {
                s.trim()
                    .parse::<f64>()
                    .map_err(|_| format!("cannot parse as number: {s:?}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !fields.is_empty() {
            rows.push(fields);
        }
    }

    if rows.is_empty() {
        return Err("no data".to_string());
    }

    let ncols = rows[0].len();

    if ncols == 1 {
        let x: Vec<f64> = (1..=rows.len()).map(|i| i as f64).collect();
        let y: Vec<f64> = rows.iter().map(|r| r[0]).collect();
        Ok(DataSet {
            headers,
            x,
            series: vec![y],
        })
    } else {
        let x: Vec<f64> = rows.iter().map(|r| r[0]).collect();
        let mut series = Vec::with_capacity(ncols - 1);
        for col in 1..ncols {
            let y: Vec<f64> = rows
                .iter()
                .map(|r| if col < r.len() { r[col] } else { 0.0 })
                .collect();
            series.push(y);
        }
        Ok(DataSet { headers, x, series })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_two_columns() {
        let lines: Vec<String> = vec!["1\t10", "2\t20", "3\t15"]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', false).unwrap();
        assert_eq!(ds.x, vec![1.0, 2.0, 3.0]);
        assert_eq!(ds.series, vec![vec![10.0, 20.0, 15.0]]);
        assert!(ds.headers.is_none());
    }

    #[test]
    fn parse_single_column() {
        let lines: Vec<String> = vec!["10", "20", "15"]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', false).unwrap();
        assert_eq!(ds.x, vec![1.0, 2.0, 3.0]);
        assert_eq!(ds.series, vec![vec![10.0, 20.0, 15.0]]);
    }

    #[test]
    fn parse_with_headers() {
        let lines: Vec<String> = vec!["x\ty", "1\t10", "2\t20"]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', true).unwrap();
        assert_eq!(ds.headers.as_ref().unwrap(), &vec!["x", "y"]);
        assert_eq!(ds.x, vec![1.0, 2.0]);
        assert_eq!(ds.series, vec![vec![10.0, 20.0]]);
    }

    #[test]
    fn parse_csv_delimiter() {
        let lines: Vec<String> = vec!["1,10", "2,20"].into_iter().map(String::from).collect();
        let ds = parse_lines(&lines, ',', false).unwrap();
        assert_eq!(ds.x, vec![1.0, 2.0]);
        assert_eq!(ds.series, vec![vec![10.0, 20.0]]);
    }

    #[test]
    fn parse_multi_series() {
        let lines: Vec<String> = vec!["0\t1\t2", "1\t3\t4"]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', false).unwrap();
        assert_eq!(ds.x, vec![0.0, 1.0]);
        assert_eq!(ds.series.len(), 2);
        assert_eq!(ds.series[0], vec![1.0, 3.0]);
        assert_eq!(ds.series[1], vec![2.0, 4.0]);
    }

    #[test]
    fn parse_empty_is_err() {
        let lines: Vec<String> = vec![];
        assert!(parse_lines(&lines, '\t', false).is_err());
    }

    #[test]
    fn x_range_single_value() {
        let ds = DataSet {
            headers: None,
            x: vec![5.0],
            series: vec![vec![10.0]],
        };
        let (min, max) = ds.x_range();
        assert!(min < max);
    }

    #[test]
    fn y_range_across_series() {
        let ds = DataSet {
            headers: None,
            x: vec![1.0, 2.0],
            series: vec![vec![5.0, 10.0], vec![3.0, 20.0]],
        };
        assert_eq!(ds.y_range(), (3.0, 20.0));
    }

    #[test]
    fn skip_empty_lines() {
        let lines: Vec<String> = vec!["1\t10", "", "2\t20", ""]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', false).unwrap();
        assert_eq!(ds.x, vec![1.0, 2.0]);
    }
}
