pub mod compare;
pub mod comparison;
pub mod virtual_fs_node;

use clap::ArgMatches;
use crate::db_models::fs_node::FsNode;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use crate::error::ErrorWrapper;

const MSG_DESCENDANT_ROOTS: &str = "roots cannot be direct descendants of each other";

pub fn run(args: &ArgMatches<'_>) -> crate::ConvertibleResult<()> {
    let first_index = fetch_fs_nodes(args, "first-index")?;
    let second_index = fetch_fs_nodes(args, "second-index")?;

    let roots_a = roots(args, "root-a");
    let roots_b = roots(args, "root-b");

    if let Err(invalid_roots) = validate_roots(&roots_a) {
        eprintln!("comparator: index 'a': invalid roots: {:?}\n{}", invalid_roots, MSG_DESCENDANT_ROOTS);
        return Err(ErrorWrapper::WithMessage(MSG_DESCENDANT_ROOTS));
    }
    if let Err(invalid_roots) = validate_roots(&roots_a) {
        eprintln!("comparator: index 'b': invalid roots: {:?}\n{}", invalid_roots, MSG_DESCENDANT_ROOTS);
        return Err(ErrorWrapper::WithMessage(MSG_DESCENDANT_ROOTS));
    }

    let output_dir = args.value_of("directory").unwrap();

    let pool_a = compare::make_pool(&first_index,  roots_a)?;
    let pool_b = compare::make_pool(&second_index, roots_b)?;

    let comparisons = compare::compare(pool_a, pool_b);
    Ok(())
}

fn fetch_fs_nodes(args: &ArgMatches<'_>, arg_name: &str) -> crate::ConvertibleResult<Vec<FsNode>> {
    let index_db_path = Path::new(args.value_of(arg_name).unwrap());
    if !index_db_path.exists() {
        eprintln!("{}: database '{}' not found.", crate::consts::PROGRAM_NAME, index_db_path.to_str().unwrap());
        return Err(crate::error::ErrorWrapper::Filesystem)
    }

    let mut fs_nodes = Vec::new();
    { // open for db work
        let conn = rusqlite::Connection::open(index_db_path)?;
        for fs_node in FsNode::select(&conn)? {
            fs_nodes.push(fs_node);
        }
    } // drops all db connections

    Ok(fs_nodes)
}

fn roots(args: &ArgMatches<'_>, arg_name: &str) -> Vec<String> {
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