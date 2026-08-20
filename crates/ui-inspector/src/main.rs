//! Agent-friendly command-line access to UI inspector references.

#![warn(rust_2018_idioms)]

mod args;
mod client;
mod run;

use clap::Parser;

fn main() {
    let code = match run::run(args::Cli::parse()) {
        Ok(()) => 0,
        Err(error) => {
            if let Some(message) = error.message() {
                eprintln!("{message}");
            }
            error.code()
        }
    };
    std::process::exit(code);
}
