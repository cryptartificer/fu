use std::env;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Line,
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
    pub width: usize,
    pub height: usize,
    pub output: Output,
    pub files: Vec<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            command: Command::Line,
            delimiter: '\t',
            has_headers: false,
            title: None,
            width: 40,
            height: 15,
            output: Output::Stderr,
            files: Vec::new(),
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
        "line" | "lineplot" => Command::Line,
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
                opts.width = val.parse().map_err(|_| "-w must be a positive integer")?;
            }
            "-h" | "--height" => {
                i += 1;
                let val = arg_value(&args, i, "-h")?;
                opts.height = val.parse().map_err(|_| "-h must be a positive integer")?;
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
    lineplot   line    draw a line chart

General options:
    -d DELIM            field delimiter (default: tab)
    -H                  input has header row
    -t TITLE            title above plot
    -w WIDTH            plot width in characters (default: 40)
    -h HEIGHT           plot height in rows (default: 15)
    -o [FILE]           output to file or stdout (default: stderr)
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
        assert_eq!(opts.width, 40);
        assert_eq!(opts.height, 15);
        assert_eq!(opts.output, Output::Stderr);
    }

    #[test]
    fn parse_line_with_options() {
        let opts = parse_from(args("line -d , -H -t MyTitle -w 60 -h 20")).unwrap();
        assert_eq!(opts.delimiter, ',');
        assert!(opts.has_headers);
        assert_eq!(opts.title.as_deref(), Some("MyTitle"));
        assert_eq!(opts.width, 60);
        assert_eq!(opts.height, 20);
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
    fn parse_unknown_command_is_err() {
        assert!(parse_from(args("bogus")).is_err());
    }
}
