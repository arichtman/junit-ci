// TODO: thin down for production
#![allow(dead_code, unused_imports, unused_variables, unreachable_code)]

extern crate clap;
use clap::{arg, command, Arg, ArgGroup, Parser};

use std::fs;

use std::path::PathBuf;

use log::{debug, error, info, warn};

// https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
#[derive(Parser, Debug)]
#[command(name = "junit-ci", author, version, about, long_about = None)] // Read from `Cargo.toml`
struct Cli {
    /// Increments logging verbosity. May be applied multiple times.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
    /// jUnit input files. May be specified multiple times.
    #[arg(short, long, action = clap::ArgAction::Append)]
    input_files: Vec<PathBuf>,
    // Disabled test threshold
    #[arg(short, long, default_value_t = 0)]
    disabled: u64,
    // Skipped test threshold
    #[arg(short, long, default_value_t = 0)]
    skipped: u64,
    // Errored test threshold
    #[arg(short, long, default_value_t = 0)]
    errored: u64,
    // Failed test threshold
    #[arg(short, long, default_value_t = 0)]
    failed: u64,
}

pub struct Sensitivity {
    disabled: u64,
    skipped: u64,
    errored: u64,
    failed: u64,
}

fn main() {
    let cli = Cli::parse();
    let log_level = match cli.verbose {
        0 => log::Level::Error,
        1 => log::Level::Warn,
        2 => log::Level::Info,
        _ => log::Level::Debug,
    };
    simple_logger::init_with_level(log_level).expect("Error initialising logging, aborting.");

    debug!("{:#?}", cli);
    let _ = junit_ci(
        cli.input_files,
        Sensitivity {
            disabled: cli.disabled,
            skipped: cli.skipped,
            errored: cli.errored,
            failed: cli.errored,
        },
    );
}

use junit_parser::{from_reader, TestSuites};
use std::io::Cursor;

// Reference: https://github.com/tobni/merge-junit
pub fn junit_ci(input_file_paths: Vec<PathBuf>, sensitivity: Sensitivity) -> u8 {
    let mut test_suites: Vec<TestSuites> = vec![];
    for file_path in input_file_paths {
        let file_contents = match fs::read_to_string(&file_path) {
            Ok(fc) => fc,
            Err(err) => {
                error!("Unable to read file {}, Skipping.", file_path.display());
                debug!("{}", err);
                continue;
            }
        };
        let cursor = Cursor::new(file_contents);
        // TODO: Consider our error handling approach, above we deal with it more explicitly and granularly
        test_suites.push(
            junit_parser::from_reader(cursor)
                .expect("Unable to parse test suites from file contents"),
        )
    }
    let mut return_code = 0;
    // TODO: Consider an iterator
    for test_suite in test_suites {
        todo!();
    }
    return_code
}
