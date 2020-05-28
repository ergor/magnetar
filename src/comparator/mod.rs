pub mod compare;
pub mod comparison;

use clap::ArgMatches;
use crate::db_models::fs_node::FsNode;
use std::path::Path;

pub fn run(args: &ArgMatches<'_>) -> crate::Result<()> {
    let first_index = fetch_fs_nodes(args, "first-index")?;
    let second_index = fetch_fs_nodes(args, "second-index")?;
    let roots_a = roots(args, "a-root");
    let roots_b = roots(args, "b-root");

    let output_dir = args.value_of("directory").unwrap();

    compare::compare(first_index, second_index, roots_a, roots_b);
    Ok(())
}

fn fetch_fs_nodes(args: &ArgMatches<'_>, arg_name: &str) -> crate::Result<Vec<FsNode>> {
    let index_db_path = Path::new(args.value_of(arg_name).unwrap());
    if !index_db_path.exists() {
        eprintln!("{}: database '{}' not found.", crate::consts::PROGRAM_NAME, index_db_path.to_str().unwrap());
        return Err(crate::error::Error::Filesystem)
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