use std::fs::File;
use std::io::{self, Write};
use std::process;

use fu::cli::{self, Command, Output};
use fu::data;
use fu::plot;
use fu::term;

fn main() {
    let mut opts = match cli::parse() {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("{msg}");
            process::exit(if msg.contains("Usage:") { 0 } else { 1 });
        }
    };

    // Auto-detect terminal size if defaults weren't overridden
    if let Some((cols, rows)) = term::size() {
        if opts.width == 40 {
            opts.width = cols.saturating_sub(4);
        }
        if opts.height == 15 {
            opts.height = rows.saturating_sub(6).max(5);
        }
    }

    let rendered = match opts.command {
        Command::Line => {
            let dataset = match data::read_input(&opts) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("fu: {e}");
                    process::exit(1);
                }
            };
            plot::render_lineplot(
                &dataset,
                opts.width,
                opts.height,
                opts.title.as_deref(),
                opts.xlabel.as_deref(),
                opts.ylabel.as_deref(),
            )
        }
        Command::Bar => {
            let bar_data = match data::read_bar_input(&opts) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("fu: {e}");
                    process::exit(1);
                }
            };
            plot::render_barplot(&bar_data, opts.width, opts.title.as_deref())
        }
        Command::Hist => {
            let values = match data::read_hist_input(&opts) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("fu: {e}");
                    process::exit(1);
                }
            };
            let nbins = opts.nbins.unwrap_or(10);
            let bar_data = data::bin_values(&values, nbins);
            plot::render_barplot(&bar_data, opts.width, opts.title.as_deref())
        }
        Command::Count => {
            let bar_data = match data::read_count_input(&opts) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("fu: {e}");
                    process::exit(1);
                }
            };
            plot::render_barplot(&bar_data, opts.width, opts.title.as_deref())
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
