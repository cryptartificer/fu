use std::fs::File;
use std::io::{self, Write};
use std::process;

use fu::cli::{self, Command, Output};
use fu::data;
use fu::plot;

fn main() {
    let opts = match cli::parse() {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("{msg}");
            process::exit(if msg.contains("Usage:") { 0 } else { 1 });
        }
    };

    let dataset = match data::read_input(&opts) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("fu: {e}");
            process::exit(1);
        }
    };

    let rendered = match opts.command {
        Command::Line => {
            plot::render_lineplot(&dataset, opts.width, opts.height, opts.title.as_deref())
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
