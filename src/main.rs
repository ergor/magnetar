mod consts;
mod create_tables;
mod error;
mod indexer;
mod comparator;
mod db_models;

use clap::{App, Arg, AppSettings};
use indexer::fs_indexer;
use indexer::index_once;
use indexer::listener;
use std::env;
use std::process::exit;
use std::result;
use std::time::{SystemTime};


pub type Result<T, E = crate::error::Error> = result::Result<T, E>;

fn main() -> crate::Result<()> {

    let subcmd_indexer = App::new("index")
        .about("Create index of chosen directories and store in a database file.")
        .setting(AppSettings::TrailingVarArg)
        .arg(Arg::with_name("daemonize")
            .short("d")
            .long("daemonize")
            .help("Run the program in background")
            .takes_value(false))
        .arg(Arg::with_name("force")
            .short("f")
            .long("force")
            .help("Create a new index instead of diffing current")
            .takes_value(false))
        .arg(Arg::with_name("listen")
            .short("l")
            .long("listen")
            .help("Listen for filesystem changes instead of active indexing")
            .takes_value(false))
        .arg(Arg::with_name("output-dir")
            .short("o")
            .long("output-dir")
            .value_name("OUTPUT DIR")
            .help("Store database file in OUTPUT DIR"))
        .arg(Arg::with_name("directories")
            .value_name("DIRECTORIES")
            .help("The directories to index")
            .required(true)
            .multiple(true));

    let subcmd_comparator = App::new("compare")
        .about("Compare two database files of indexing-runs and generate html report of differences.")
        .arg(Arg::with_name("first-index")
            .short("a")
            .long("first-index")
            .value_name("FILE")
            .help("The first input database file.")
            .required(true))
        .arg(Arg::with_name("second-index")
            .short("b")
            .long("second-index")
            .value_name("FILE")
            .help("Second input database file.")
            .required(true))
        .arg(Arg::with_name("a-root")
            .long("a-root")
            .value_name("ROOT")
            .next_line_help(true)
            .multiple(true)
            .help("Add ROOT as a comparison root for the 'a' (i.e. first) index.\n\
                  This option can be specified multiple times.\n\
                  If no root is specified, '/' is assumed."))
        .arg(Arg::with_name("b-root")
            .long("b-root")
            .value_name("ROOT")
            .next_line_help(true)
            .multiple(true)
            .help("Add ROOT as a comparison root for the 'b' (i.e. second) index.\n\
                  This option can be specified multiple times.\n\
                  If no root is specified, '/' is assumed."))
        .arg(Arg::with_name("directory")
            .value_name("DIRECTORY")
            .index(1)
            .help("The directory to store the generated comparison report in.")
            .required(true));

    let args = App::new(consts::PROGRAM_NAME)
        .version(clap::crate_version!())
        .about("Filesystem indexer for archival management")
        .subcommand(subcmd_indexer)
        .subcommand(subcmd_comparator)
        .get_matches();

    if let Some(args) = args.subcommand_matches("index") {
        indexer::run(args);
    }
    else if let Some(args) = args.subcommand_matches("compare") {
        comparator::run(args);
    }

    return Ok(());
}
