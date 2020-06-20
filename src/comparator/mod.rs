//mod fs_tree;
mod compare;
mod delta;
mod report;
mod virtual_fs_node;

use clap;
use crate::ConvertibleResult;
use crate::apperror::AppError;
use crate::db_models::fs_node::FsNode;
use crate::errorwrapper::ErrorWrapper;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::io;
use crate::comparator::delta::Attribute;

macro_rules! validate_roots {
    ($roots_ref:expr, $index_name:literal) => {
        {
            log::debug!("validating roots: '{}'...", $index_name);
            if let Err(invalid_roots) = _validate_roots($roots_ref) {
                let error = AppError::WithMessage(
                    format!("index '{}': invalid roots: {:?}\nroots cannot be direct descendants of each other", $index_name, invalid_roots)
                );
                log::error!("{}", error);
                return Err(ErrorWrapper::AppError(error));
            }
            log::debug!("roots '{}' OK: {:?}", $index_name, $roots_ref);
        }
    }
}

pub fn run(args: &clap::ArgMatches<'_>) -> ConvertibleResult<()> {
    let first_index = fetch_fs_nodes(args, "first-index")?;
    let second_index = fetch_fs_nodes(args, "second-index")?;

    let attrs = match args.value_of("mode") {
        None => Attribute::all(),
        Some(m) => Attribute::from_arg(m)?
    };

    let roots_a = roots(args, "root-a");
    let roots_b = roots(args, "root-b");

    validate_roots!(&roots_a, "a");
    validate_roots!(&roots_b, "b");

    let pool_a = compare::make_pool(&first_index,  roots_a)?;
    let pool_b = compare::make_pool(&second_index, roots_b)?;

    let deltas = compare::compare(pool_a, pool_b, &attrs);

    let output_stream = match args.value_of("directory") {
        None => { io::stdout() },
        Some(_dir) => { unimplemented!("writing to file not implemented") },
    };

    report::write(output_stream, deltas)?;

    Ok(())
}

fn fetch_fs_nodes(args: &clap::ArgMatches<'_>, arg_name: &str) -> crate::ConvertibleResult<Vec<FsNode>> {
    log::debug!("fetching fs_nodes for '{}'", arg_name);

    let index_db_path = Path::new(args.value_of(arg_name).unwrap());
    if !index_db_path.exists() {
        let error = AppError::WithMessage(
            format!("database '{}' not found.", index_db_path.to_str().unwrap_or("(.to_str() failed)"))
        );
        log::error!("{}", error);
        return Err(ErrorWrapper::AppError(error))
    }

    let mut fs_nodes = Vec::new();
    { // open for db work
        let conn = rusqlite::Connection::open(index_db_path)?;
        log::debug!("{}: database connection opened", index_db_path.to_string_lossy().as_ref());
        std::mem::drop(fs_nodes);
        fs_nodes = FsNode::select(&conn)?;
        log::debug!("{}: retrieved {} rows.", index_db_path.to_string_lossy().as_ref(), fs_nodes.len());
    } // drops all db connections
    log::debug!("{}: database connection closed", index_db_path.to_string_lossy().as_ref());

    Ok(fs_nodes)
}

fn roots(args: &clap::ArgMatches<'_>, arg_name: &str) -> Vec<String> {
    log::debug!("collecting roots for '{}'...", arg_name);
    let mut roots = Vec::new();
    match args.values_of(arg_name) {
        None => {
            roots.push(String::from("/"));
        },
        Some(values) => {
            for value in values {
                roots.push(String::from(value));
            }
        }
    }
    log::debug!("found {} roots", roots.len());
    return roots;
}

fn _validate_roots(roots: &Vec<String>) -> Result<(), HashSet<String>> {
    let mut invalid_roots: HashSet<String> = HashSet::new();

    for i in 0..roots.len() {
        let root_i = PathBuf::from(&roots[i]);
        for j in i+1..roots.len() {
            let root_j = PathBuf::from(&roots[j]);
            if root_j.starts_with(&root_i) {
                invalid_roots.insert(String::from(root_i.to_str().unwrap()));
                invalid_roots.insert(String::from(root_j.to_str().unwrap()));
            }
        }
    }

    if invalid_roots.is_empty() {
        return Ok(());
    }
    Err(invalid_roots)
}

pub fn cmdline<'a>() -> clap::App<'a, 'a> {
    clap::App::new("compare")
        .about("Compare two database files of indexing-runs and generate html report of differences.")
        .arg(clap::Arg::with_name("first-index")
            .short("a")
            .long("first-index")
            .value_name("FILE")
            .help("The first input database file.")
            .required(true))
        .arg(clap::Arg::with_name("second-index")
            .short("b")
            .long("second-index")
            .value_name("FILE")
            .help("Second input database file.")
            .required(true))
        .arg(clap::Arg::with_name("directory")
            .short("o")
            .long("output-dir")
            .value_name("DIRECTORY")
            .help("The directory to store the generated comparison report in.")
            .required(false))
        .arg(clap::Arg::with_name("root-a")
            .long("root-a")
            .value_name("ROOT")
            .next_line_help(true)
            .multiple(true)
            .help("Add ROOT as a comparison root for the 'a' (i.e. first) index.\n\
                  This option can be specified multiple times.\n\
                  If no root is specified, '/' is assumed."))
        .arg(clap::Arg::with_name("root-b")
            .long("root-b")
            .value_name("ROOT")
            .next_line_help(true)
            .multiple(true)
            .help("Add ROOT as a comparison root for the 'b' (i.e. second) index.\n\
                  This option can be specified multiple times.\n\
                  If no root is specified, '/' is assumed."))
        .arg(clap::Arg::with_name("mode")
            .long("mode")
            .short("m")
            .value_name("MODE")
            .next_line_help(true)
            .help("What attributes should count towards being a change.\n\
                  node(t)ype, (c)hecksum, (s)ize, (u)ser, (g)roup, (p)ermissions,\n\
                  (b)irthdate, (m)odifieddate, (l)inksto, (i)node, (n)links"))
}