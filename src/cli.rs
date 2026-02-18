use std::env;

use crate::color::{self, Color};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Line,
    Lines,
    Scatter,
    Bar,
    Hist,
    Count,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Stderr,
    Stdout,
    File(String),
}

#[derive(Debug)]
pub struct Options {
    pub command: Command,
    pub delimiter: char,
    pub has_headers: bool,
    pub title: Option<String>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub output: Output,
    pub files: Vec<String>,
    pub nbins: Option<usize>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub transpose: bool,
    pub color: Option<Color>,
    pub force_color: bool,
    pub monochrome: bool,
    pub grid: bool,
    pub xlim: Option<(f64, f64)>,
    pub ylim: Option<(f64, f64)>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            command: Command::Line,
            delimiter: '\t',
            has_headers: false,
            title: None,
            width: None,
            height: None,
            output: Output::Stderr,
            files: Vec::new(),
            nbins: None,
            xlabel: None,
            ylabel: None,
            transpose: false,
            color: None,
            force_color: false,
            monochrome: false,
            grid: false,
            xlim: None,
            ylim: None,
        }
    }
}

pub fn parse() -> Result<Options, String> {
    parse_from(env::args().skip(1).collect())
}

pub fn parse_from(args: Vec<String>) -> Result<Options, String> {
    if args.is_empty() {
        return Err(usage());
    }

    if args[0] == "--help" {
        return Err(usage());
    }

    let command = match args[0].as_str() {
        "line" | "lineplot" | "l" => Command::Line,
        "lines" | "lineplots" => Command::Lines,
        "scatter" | "s" => Command::Scatter,
        "bar" | "barplot" => Command::Bar,
        "hist" | "histogram" => Command::Hist,
        "count" | "c" => Command::Count,
        other => return Err(format!("unknown command: {other}\n\n{}", usage())),
    };

    let mut opts = Options {
        command,
        ..Default::default()
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" => return Err(usage_line()),
            "-d" | "--delimiter" => {
                i += 1;
                let val = arg_value(&args, i, "-d")?;
                opts.delimiter = val.chars().next().unwrap_or('\t');
            }
            "-H" | "--headers" => {
                opts.has_headers = true;
            }
            "-t" | "--title" => {
                i += 1;
                opts.title = Some(arg_value(&args, i, "-t")?);
            }
            "-w" | "--width" => {
                i += 1;
                let val = arg_value(&args, i, "-w")?;
                opts.width = Some(val.parse().map_err(|_| "-w must be a positive integer")?);
            }
            "-h" | "--height" => {
                i += 1;
                let val = arg_value(&args, i, "-h")?;
                opts.height = Some(val.parse().map_err(|_| "-h must be a positive integer")?);
            }
            "-T" | "--transpose" => {
                opts.transpose = true;
            }
            "--xlabel" => {
                i += 1;
                opts.xlabel = Some(arg_value(&args, i, "--xlabel")?);
            }
            "--ylabel" => {
                i += 1;
                opts.ylabel = Some(arg_value(&args, i, "--ylabel")?);
            }
            "-c" | "--color" => {
                i += 1;
                let val = arg_value(&args, i, "-c")?;
                opts.color = Some(color::parse_color(&val)?);
            }
            "--xlim" => {
                i += 1;
                let val = arg_value(&args, i, "--xlim")?;
                opts.xlim = Some(parse_range(&val, "--xlim")?);
            }
            "--ylim" => {
                i += 1;
                let val = arg_value(&args, i, "--ylim")?;
                opts.ylim = Some(parse_range(&val, "--ylim")?);
            }
            "--grid" => {
                opts.grid = true;
            }
            "-C" | "--color-output" => {
                opts.force_color = true;
            }
            "-M" | "--monochrome" => {
                opts.monochrome = true;
            }
            "-n" | "--nbins" => {
                i += 1;
                let val = arg_value(&args, i, "-n")?;
                opts.nbins = Some(val.parse().map_err(|_| "-n must be a positive integer")?);
            }
            "-o" | "--output" => {
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    let val = &args[i];
                    if val == "-" {
                        opts.output = Output::Stdout;
                    } else {
                        opts.output = Output::File(val.clone());
                    }
                } else {
                    opts.output = Output::Stdout;
                }
            }
            arg if !arg.starts_with('-') => {
                opts.files.push(arg.to_string());
            }
            other => {
                return Err(format!("unknown option: {other}"));
            }
        }
        i += 1;
    }

    Ok(opts)
}

fn parse_range(s: &str, flag: &str) -> Result<(f64, f64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(format!(
            "{flag} requires two comma-separated numbers (e.g. 0,100)"
        ));
    }
    let lo: f64 = parts[0]
        .trim()
        .parse()
        .map_err(|_| format!("{flag}: cannot parse {0:?} as number", parts[0].trim()))?;
    let hi: f64 = parts[1]
        .trim()
        .parse()
        .map_err(|_| format!("{flag}: cannot parse {0:?} as number", parts[1].trim()))?;
    Ok((lo, hi))
}

fn arg_value(args: &[String], i: usize, flag: &str) -> Result<String, String> {
    args.get(i)
        .cloned()
        .ok_or_else(|| format!("{flag} requires an argument"))
}

fn usage() -> String {
    "\
Program: fu (brutally fast terminal plotting)
Source:  https://github.com/CryptArtificer/fu

Usage:   fu <command> [options] <in.tsv>

Commands:
    lineplot   line l  draw a line chart
    lineplots  lines   draw multi-series line chart
    scatter    s       draw a scatter plot
    barplot    bar     draw a horizontal bar chart
    histogram  hist    draw a histogram
    count      c       count occurrences and bar chart

General options:
    -d DELIM            field delimiter (default: tab)
    -H                  input has header row
    -T                  transpose rows and columns
    -t TITLE            title above plot
    -w WIDTH            plot width in characters (default: 40)
    -h HEIGHT           plot height in rows (default: 15)
    -n BINS             number of histogram bins (default: 10)
    -o [FILE]           output to file or stdout (default: stderr)
    -c COLOR            drawing color (name or 0-255)
    -C                  force color output in pipes
    -M                  monochrome (no color)
    --grid              show grid lines
    --xlim MIN,MAX      x-axis range limits
    --ylim MIN,MAX      y-axis range limits
    --xlabel LABEL      x-axis label
    --ylabel LABEL      y-axis label
    --help              print help menu"
        .to_string()
}

fn usage_line() -> String {
    "\
Usage: fu line [options] <in.tsv>

Options for line:
    (none yet)

Common options:
    -d DELIM            field delimiter (default: tab)
    -H                  input has header row
    -t TITLE            title above plot
    -w WIDTH            plot width in characters (default: 40)
    -h HEIGHT           plot height in rows (default: 15)
    -o [FILE]           output to file or stdout (default: stderr)
    --help              print sub-command help menu"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn parse_line_defaults() {
        let opts = parse_from(args("line")).unwrap();
        assert_eq!(opts.command, Command::Line);
        assert_eq!(opts.delimiter, '\t');
        assert!(!opts.has_headers);
        assert_eq!(opts.width, None);
        assert_eq!(opts.height, None);
        assert_eq!(opts.output, Output::Stderr);
    }

    #[test]
    fn parse_line_with_options() {
        let opts = parse_from(args("line -d , -H -t MyTitle -w 60 -h 20")).unwrap();
        assert_eq!(opts.delimiter, ',');
        assert!(opts.has_headers);
        assert_eq!(opts.title.as_deref(), Some("MyTitle"));
        assert_eq!(opts.width, Some(60));
        assert_eq!(opts.height, Some(20));
    }

    #[test]
    fn parse_output_stdout() {
        let opts = parse_from(args("line -o")).unwrap();
        assert_eq!(opts.output, Output::Stdout);
    }

    #[test]
    fn parse_output_file() {
        let opts = parse_from(args("line -o chart.txt")).unwrap();
        assert_eq!(opts.output, Output::File("chart.txt".to_string()));
    }

    #[test]
    fn parse_file_args() {
        let opts = parse_from(args("line data.tsv")).unwrap();
        assert_eq!(opts.files, vec!["data.tsv"]);
    }

    #[test]
    fn parse_no_args_is_err() {
        assert!(parse_from(vec![]).is_err());
    }

    #[test]
    fn parse_bar_command() {
        let opts = parse_from(args("bar")).unwrap();
        assert_eq!(opts.command, Command::Bar);
    }

    #[test]
    fn parse_barplot_alias() {
        let opts = parse_from(args("barplot -t Test")).unwrap();
        assert_eq!(opts.command, Command::Bar);
        assert_eq!(opts.title.as_deref(), Some("Test"));
    }

    #[test]
    fn parse_hist_command() {
        let opts = parse_from(args("hist -n 20")).unwrap();
        assert_eq!(opts.command, Command::Hist);
        assert_eq!(opts.nbins, Some(20));
    }

    #[test]
    fn parse_count_command() {
        let opts = parse_from(args("count")).unwrap();
        assert_eq!(opts.command, Command::Count);
    }

    #[test]
    fn parse_transpose_and_labels() {
        let opts = parse_from(args("line -T --xlabel Time --ylabel Value")).unwrap();
        assert!(opts.transpose);
        assert_eq!(opts.xlabel.as_deref(), Some("Time"));
        assert_eq!(opts.ylabel.as_deref(), Some("Value"));
    }

    #[test]
    fn parse_lines_command() {
        let opts = parse_from(args("lines")).unwrap();
        assert_eq!(opts.command, Command::Lines);
    }

    #[test]
    fn parse_lineplots_alias() {
        let opts = parse_from(args("lineplots -t Multi")).unwrap();
        assert_eq!(opts.command, Command::Lines);
        assert_eq!(opts.title.as_deref(), Some("Multi"));
    }

    #[test]
    fn parse_scatter_command() {
        let opts = parse_from(args("scatter")).unwrap();
        assert_eq!(opts.command, Command::Scatter);
    }

    #[test]
    fn parse_scatter_alias() {
        let opts = parse_from(args("s -t Dots")).unwrap();
        assert_eq!(opts.command, Command::Scatter);
        assert_eq!(opts.title.as_deref(), Some("Dots"));
    }

    #[test]
    fn parse_unknown_command_is_err() {
        assert!(parse_from(args("bogus")).is_err());
    }
}
