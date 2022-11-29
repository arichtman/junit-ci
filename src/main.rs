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
    let exit_code = junit_ci(
        cli.input_files,
        Sensitivity {
            skipped: cli.skipped,
            errored: cli.errored,
            failed: cli.failed,
        },
    );
    std::process::exit(exit_code);
}

use junit_parser::{from_reader, TestSuites};
use std::io::Cursor;

// Reference: https://github.com/tobni/merge-junit
pub fn junit_ci(input_file_paths: Vec<PathBuf>, sensitivity: Sensitivity) -> i32 {
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
    let mut total_skipped: u64 = 0;
    let mut total_errored: u64 = 0;
    let mut total_failed: u64 = 0;
    // TODO: Consider this approach.
    for test_suite in test_suites {
        total_skipped += test_suite.skipped;
        total_errored += test_suite.errors;
        total_failed += test_suite.failures;
        debug!("{:?}", test_suite);
        info!(
            "Skipped: {}, Errored: {}, Failed: {}",
            total_skipped, total_errored, total_failed
        );
    }
    let mut return_code = 0;
    // TODO: Consider matching?
    // match (test_suite.skipped, test_suite.errors, test_suite.failures) {
    //     ( x > Sensitivity.skipped, _, _) => None
    // }
    if total_skipped > sensitivity.skipped {
        return_code += 1;
        info!(
            "Total skipped {} greater than threshold {}",
            total_skipped, sensitivity.skipped
        )
    }
    if total_errored > sensitivity.errored {
        return_code += 1;
        info!(
            "Total errored {} greater than threshold {}",
            total_errored, sensitivity.errored
        )
    }
    if total_failed > sensitivity.failed {
        return_code += 1;
        info!(
            "Total failed {} greater than threshold {}",
            total_failed, sensitivity.failed
        )
    }
    return_code
}
