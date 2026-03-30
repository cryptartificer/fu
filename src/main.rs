use std::fs::File;
use std::io::{self, IsTerminal, Write};
use std::process;

use fu::cli::{self, Command, Output, Sides};
use fu::color::{ColorMode, PALETTE};
use fu::data;
use fu::plot::{self, PlotOptions};
use fu::term;

fn resolve_color(opts: &cli::Options) -> ColorMode {
    if opts.monochrome {
        return ColorMode::Off;
    }

    let output_is_tty = match &opts.output {
        Output::Stderr => io::stderr().is_terminal(),
        Output::Stdout => io::stdout().is_terminal(),
        Output::File(_) => false,
    };

    if !opts.force_color && !output_is_tty {
        return ColorMode::Off;
    }

    if let Some(c) = opts.color {
        ColorMode::Single(c)
    } else {
        ColorMode::Auto(PALETTE.to_vec())
    }
}

fn main() {
    let opts = match cli::parse() {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("{msg}");
            process::exit(if msg.contains("Usage:") { 0 } else { 1 });
        }
    };

    // Resolve width/height: explicit flag > terminal auto-detect > fallback
    let term_dims = term::size();
    let width = opts
        .width
        .unwrap_or_else(|| term_dims.map(|(c, _)| c.saturating_sub(4)).unwrap_or(40));
    let height = opts.height.unwrap_or_else(|| {
        term_dims
            .map(|(_, r)| r.saturating_sub(6).max(5))
            .unwrap_or(15)
    });

    let color_mode = resolve_color(&opts);
    let default_margin = match opts.command {
        Command::Bar | Command::Hist | Command::Count => Sides::all(0),
        _ => Sides::new(0, 0, 0, 3),
    };
    let margin = opts.margin.unwrap_or(default_margin);
    let padding = opts.padding.unwrap_or(Sides::all(0));

    let rendered = match opts.command {
        Command::Line | Command::Lines | Command::Scatter => {
            let dataset = match data::read_input(&opts) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("fu: {e}");
                    process::exit(1);
                }
            };
            let plot_opts = PlotOptions {
                width,
                height,
                title: opts.title.as_deref(),
                xlabel: opts.xlabel.as_deref(),
                ylabel: opts.ylabel.as_deref(),
                color_mode: &color_mode,
                grid: opts.grid,
                xlim: opts.xlim,
                ylim: opts.ylim,
                margin,
                padding,
            };
            match opts.command {
                Command::Scatter => plot::render_scatter(&dataset, &plot_opts),
                _ => plot::render_lineplot(&dataset, &plot_opts),
            }
        }
        Command::Bar => {
            let bar_data = match data::read_bar_input(&opts) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("fu: {e}");
                    process::exit(1);
                }
            };
            plot::render_barplot(
                &bar_data,
                width,
                opts.title.as_deref(),
                &color_mode,
                &margin,
                &padding,
                '■',
            )
        }
        Command::Hist => {
            let exact_n = opts.nbins.is_some();
            let nbins = opts.nbins.unwrap_or(10);
            let delimiter = opts.delimiter as u8;
            let bar_data = if opts.files.is_empty() {
                fu::hist::hist_from_stdin(
                    delimiter,
                    opts.has_headers,
                    opts.gt,
                    opts.lt,
                    opts.log_scale,
                    nbins,
                    exact_n,
                )
            } else if opts.files.len() == 1 {
                fu::hist::hist_from_file(
                    &opts.files[0],
                    delimiter,
                    opts.has_headers,
                    opts.gt,
                    opts.lt,
                    opts.log_scale,
                    nbins,
                    exact_n,
                )
            } else {
                let mut buf = Vec::new();
                for path in &opts.files {
                    match std::fs::read(path) {
                        Ok(content) => {
                            buf.extend_from_slice(&content);
                            if buf.last() != Some(&b'\n') {
                                buf.push(b'\n');
                            }
                        }
                        Err(e) => {
                            eprintln!("fu: {path}: {e}");
                            process::exit(1);
                        }
                    }
                }
                fu::hist::hist_from_bytes(
                    &buf,
                    delimiter,
                    opts.has_headers,
                    opts.gt,
                    opts.lt,
                    opts.log_scale,
                    nbins,
                    exact_n,
                )
            };
            match bar_data {
                Ok(d) => plot::render_barplot(
                    &d,
                    width,
                    opts.title.as_deref(),
                    &color_mode,
                    &margin,
                    &padding,
                    '▇',
                ),
                Err(e) => {
                    eprintln!("fu: {e}");
                    process::exit(1);
                }
            }
        }
        Command::Count => {
            let bar_data = match data::read_count_input(&opts) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("fu: {e}");
                    process::exit(1);
                }
            };
            plot::render_barplot(
                &bar_data,
                width,
                opts.title.as_deref(),
                &color_mode,
                &margin,
                &padding,
                '■',
            )
        }
    };

    let result = match &opts.output {
        Output::Stderr => io::stderr().write_all(rendered.as_bytes()),
        Output::Stdout => io::stdout().write_all(rendered.as_bytes()),
        Output::File(path) => File::create(path).and_then(|mut f| f.write_all(rendered.as_bytes())),
    };

    if let Err(e) = result {
        eprintln!("fu: write error: {e}");
        process::exit(1);
    }
}
