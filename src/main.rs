//!
//! magnetar
//!

#![warn(missing_debug_implementations, rust_2018_idioms, missing_docs)]

mod apperror;
mod comparator;
mod consts;
mod create_tables;
mod db_models;
mod dupes;
mod errorwrapper;
mod indexer;

use clap::App;
use crate::errorwrapper::ErrorWrapper;
use indexer::fs_indexer;
use std::env;
use std::result;
use flexi_logger;
use log::LevelFilter;

const LOGGING_LEVEL_VERBOSE: &str = "magnetar = trace";
const LOGGING_LEVEL_DEFAULT: &str = "magnetar = debug";

/// A `Result` type that uses [ErrorWrapper]() as error type, which implements the [From]() trait
/// on every error type used in this program.
pub type ConvertibleResult<T, E = ErrorWrapper> = result::Result<T, E>;

fn main() -> crate::ConvertibleResult<()> {

    let mut log_handle = flexi_logger::Logger::with_str(LOGGING_LEVEL_DEFAULT)
        .log_to_file()
        .directory("logs")
        .rotate(
            flexi_logger::Criterion::Size(1_000_000),
            flexi_logger::Naming::Timestamps,
            flexi_logger::Cleanup::KeepLogFiles(3)
        )
        .duplicate_to_stderr(flexi_logger::Duplicate::Warn)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let args = App::new(consts::PROGRAM_NAME)
        .version(clap::crate_version!())
        .about("Filesystem indexer for archival management")
        .arg(clap::Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Enables verbose logging")
            .takes_value(false))
        .subcommand(indexer::cmdline())
        .subcommand(comparator::cmdline())
        .subcommand(dupes::cmdline())
        .get_matches();

    if args.is_present("verbose") {
        log_handle.parse_new_spec(LOGGING_LEVEL_VERBOSE);
    }

    if let Some(args) = args.subcommand_matches("index") {
        indexer::run(args)?;
    }
    else if let Some(args) = args.subcommand_matches("compare") {
        comparator::run(args)?;
    }

    return Ok(());
}
