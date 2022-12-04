// Note: Uncomment for development
// #![allow(dead_code, unused_imports, unused_variables, unreachable_code)]

use clap::{arg, command, Parser};

use std::path::PathBuf;

use log::{debug, error, info, trace, warn};

// TODO: Update env use when issue is resolved https://github.com/clap-rs/clap/issues/3221
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    // Note: I wasn't able to find success specifying multiple files via env var
    // I tried spaces, semicolons, and commas, but it seems to only read in as a whole string.
    // There may be other features that would enable sending an array but I'm not putting any more effort into it for now
    /// jUnit input files.
    #[arg(short, long, action = clap::ArgAction::Append, env = "JCI_FILE", long_help = "Required. May be specified multiple times.")]
    input_files: Vec<PathBuf>,
    // Note: I'm not sure that setting failure threshold 0 on unspecified conditions is a good default
    // TODO: Consider Options that default to u64 max, effectively ignoring counts of anything not specified
    /// Skipped test threshold
    #[arg(short, long, env = "JCI_SKIPPED", default_value_t = 0)]
    skipped: u64,
    /// Errored test threshold
    #[arg(short, long, env = "JCI_ERRORED", default_value_t = 0)]
    errored: u64,
    /// Failed test threshold
    #[arg(short, long, env = "JCI_FAILED", default_value_t = 0)]
    failed: u64,
    /// Increments logging verbosity.
    #[arg(short, long, action = clap::ArgAction::Count, env = "JCI_VERBOSE", long_help = "Optional. May be applied up to 4 times. Environment variable takes integer.")]
    verbose: u8,
}

fn main() {
    let cli = Cli::parse();
    // TODO: Is this the idiomatic way to handle it?
    let log_level = match cli.verbose {
        0 => log::Level::Error,
        1 => log::Level::Warn,
        2 => log::Level::Info,
        3 => log::Level::Debug,
        4.. => log::Level::Trace,
    };
    simple_logger::init_with_level(log_level).expect("Error initialising logging, aborting.");
    // TODO: Learn best logging practices.
    // Specifically: The debug here redundifies the info level and should we use "{:?}" or "{:#?}"
    info!(
        "Log level {}, skip threshold {}, error threshold {}, fail threshold {}, input files {:?}",
        cli.verbose, cli.skipped, cli.errored, cli.failed, cli.input_files
    );
    debug!("{:?}", cli);
    if cli.input_files.is_empty() {
        error!("No input files specified, aborting.");
        std::process::exit(-1);
    }
    match junit_ci(cli.input_files, cli.skipped, cli.errored, cli.failed) {
        Ok(_) => return,
        Err(exit_code) => std::process::exit(exit_code),
    }
}

use std::io::Cursor;

use std::fs;

// TODO: Reconsider Result return type. It's logical but would be easier to return the error code directly?
pub fn junit_ci(
    input_file_paths: Vec<PathBuf>,
    skip_threshold: u64,
    error_threshold: u64,
    fail_threshold: u64,
) -> Result<(), i32> {
    let mut test_suites: Vec<junit_parser::TestSuites> = vec![];
    let mut file_count: u8 = 0;
    for file_path in input_file_paths {
        let file_contents = match fs::read_to_string(&file_path) {
            Ok(fc) => fc,
            Err(err) => {
                warn!("Unable to read file {}, Skipping.", file_path.display());
                debug!("{}", err);
                continue;
            }
        };
        file_count += 1;
        let mut xml_documents: Vec<String> = vec![];
        split_xml_documents(file_contents, &mut xml_documents);
        for xml_doc in xml_documents {
            let cursor = Cursor::new(xml_doc);
            // TODO: Consider our error handling approach, above we deal with it more explicitly and granularly
            test_suites.push(
                junit_parser::from_reader(cursor)
                    .expect("Unable to parse test suites from document contents"),
            )
        }
    }
    if file_count == 0 {
        error!("No files read, aborting.");
        return Err(-3);
    }
    let mut total_tests: u64 = 0;
    let mut total_skipped: u64 = 0;
    let mut total_errored: u64 = 0;
    let mut total_failed: u64 = 0;
    // TODO: Reconsider this approach.
    for test_suite in test_suites {
        // Iterate the subsuites as top-level node is optional
        for test_suite in test_suite.suites {
            info!("Processing: {}", test_suite.name);
            if test_suite.tests == 0 {
                warn!("No tests in this test suite, proceeding.");
                continue;
            }
            debug!(
                "{} tests in {}. {} skipped, {} errored, {} failed.",
                test_suite.tests,
                test_suite.time,
                test_suite.skipped,
                test_suite.errors,
                test_suite.failures
            );
            trace!("{:?}", test_suite);
            total_tests += test_suite.tests;
            total_skipped += test_suite.skipped;
            total_errored += test_suite.errors;
            total_failed += test_suite.failures;
        }
    }
    debug!(
        "Total {}: skipped: {}, errored: {}, failed: {}.",
        total_tests, total_skipped, total_errored, total_failed
    );
    if total_tests == 0 {
        error!("No tests found, aborting.");
        return Err(-2);
    }
    let mut exit_code: i32 = 0;
    // TODO: Consider matching?
    // match (total_skipped, total_errored, total_failed) {
    //     ( x > Sensitivity.skipped, _, _) => None
    // }
    if total_skipped > skip_threshold {
        exit_code += 1;
        info!(
            "Total skipped {} greater than threshold {}.",
            total_skipped, skip_threshold
        )
    }
    if total_errored > error_threshold {
        exit_code += 2;
        info!(
            "Total errored {} greater than threshold {}.",
            total_errored, error_threshold
        )
    }
    if total_failed > fail_threshold {
        exit_code += 4;
        info!(
            "Total failed {} greater than threshold {}.",
            total_failed, fail_threshold
        )
    }
    match exit_code {
        0 => Ok(()),
        c => {
            // TODO: Use the logger? But Error should be reserved for actual application errors?
            //  If we put it as INFO it won't display by default though...
            println!("Successfully parsed results!");
            Err(c as i32)
        }
    }
}

// TODO: Rework this extremely hacky XML splitting to handle multi-document files
fn split_xml_documents(all_docs_string: String, return_vector: &mut Vec<String>) {
    const XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>"#;
    let mut residual_string = all_docs_string;
    loop {
        // Note: Actually parse _backwards_ so we can avoid being stuck on the first case
        //  where the byte location can be 0 in perpetuity
        let byte_index = match residual_string.rfind(XML_HEADER) {
            None => break,
            Some(x) => x,
        };
        trace!("{}", residual_string);
        trace!("{}", byte_index);
        let split_xml_content = residual_string.split_at(byte_index);
        trace!("{}", split_xml_content.1);
        return_vector.push(split_xml_content.1.to_string());
        residual_string = split_xml_content.0.to_string();
    }
}
