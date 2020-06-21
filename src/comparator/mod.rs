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
            if let Err(invalid_roots) = validate_roots($roots_ref) {
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
    let db_path_a = args.value_of("first-index").expect("path to database is required");
    let db_path_b = args.value_of("second-index").expect("path to database is required");

    let first_index = FsNode::select_n(db_path_a)?;
    let second_index = FsNode::select_n(db_path_b)?;

    let attrs_opt =
        if args.is_present("mode-all") {
            Some(Attribute::all())
        } else if args.is_present("mode-min") {
            Some(Attribute::minimum())
        } else { None };

    let attrs = match attrs_opt {
        Some(a) => a,
        None => {
            match args.value_of("mode") {
                Some(m) => Attribute::from_arg(m)?,
                None => Attribute::medium(),
            }
        }
    };

    let roots_a = roots(args, "root-a");
    let roots_b = roots(args, "root-b");

    validate_roots!(&roots_a, "a");
    validate_roots!(&roots_b, "b");

    let summary = report::ReportSummary {
        db_a_name: db_path_a.to_string(),
        db_b_name: db_path_b.to_string(),
        roots_a: roots_a.clone(),
        roots_b: roots_b.clone()
    };

    let pool_a = compare::make_pool(&first_index,  roots_a)?;
    let pool_b = compare::make_pool(&second_index, roots_b)?;

    let deltas = compare::compare(pool_a, pool_b, &attrs);

    let output_stream = match args.value_of("directory") {
        None => { io::stdout() },
        Some(_dir) => { unimplemented!("writing to file not implemented") },
    };

    report::write(output_stream, deltas, summary)?;

    Ok(())
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

fn validate_roots(roots: &Vec<String>) -> Result<(), HashSet<String>> {
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
                  If not specified, defaults to 'csugpcm'.\n\
                  node(t)ype, (c)hecksum, (s)ize, (u)ser, (g)roup, (p)ermissions,\n\
                  (b)irthdate, (m)odifieddate, (l)inksto, (i)node, (n)links"))
        .arg(clap::Arg::with_name("mode-all")
            .long("mode-all")
            .short("A")
            .conflicts_with_all(&["mode", "mode-min"])
            .takes_value(false)
            .help("Enable all flags for mode. Equivalent to --mode tcsugpbmlin"))
        .arg(clap::Arg::with_name("mode-min")
            .long("mode-min")
            .short("M")
            .conflicts_with_all(&["mode", "mode-all"])
            .takes_value(false)
            .help("Enable a small subset of flags for mode. Equivalent to --mode csm"))
}