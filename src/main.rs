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

/// A `Result` type that uses [ErrorWrapper]() as error type, which implements the [From]() trait
/// on every error type used in this program.
pub type ConvertibleResult<T, E = ErrorWrapper> = result::Result<T, E>;

fn main() -> crate::ConvertibleResult<()> {

    #[cfg(feature = "verbose")]
    const LOGGING_LEVEL: &str = "magnetar = trace";
    #[cfg(not(feature = "verbose"))]
    const LOGGING_LEVEL: &str = "magnetar = debug";

    flexi_logger::Logger::with_str(LOGGING_LEVEL)
        .log_to_file()
        .duplicate_to_stderr(flexi_logger::Duplicate::Warn)
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    let args = App::new(consts::PROGRAM_NAME)
        .version(clap::crate_version!())
        .about("Filesystem indexer for archival management")
        .subcommand(indexer::cmdline())
        .subcommand(comparator::cmdline())
        .subcommand(dupes::cmdline())
        .get_matches();

    if let Some(args) = args.subcommand_matches("index") {
        indexer::run(args)?;
    }
    else if let Some(args) = args.subcommand_matches("compare") {
        comparator::run(args)?;
    }

    return Ok(());
}
