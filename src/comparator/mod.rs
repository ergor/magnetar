pub mod compare;
pub mod comparison;

use clap::ArgMatches;
use crate::db_models::fs_node::FsNode;
use std::path::Path;

pub fn run(args: &ArgMatches<'_>) -> crate::Result<()> {
    let first_index = Path::new(args.value_of("first-index").unwrap());
    let second_index = Path::new(args.value_of("second-index").unwrap());

    if !first_index.exists() {
        eprintln!("{}: database '{}' not found.", crate::consts::PROGRAM_NAME, first_index.to_str().unwrap());
        return Err(crate::error::Error::Filesystem)
    }
    if !second_index.exists() {
        eprintln!("{}: database '{}' not found.", crate::consts::PROGRAM_NAME, second_index.to_str().unwrap());
        return Err(crate::error::Error::Filesystem)
    }

    let output_dir = args.value_of("directory").unwrap();

    let mut a = Vec::new();
    let mut b = Vec::new();

    { // open for db work
        let conn_a = rusqlite::Connection::open(first_index)?;
        let conn_b = rusqlite::Connection::open(second_index)?;

        for fs_node in FsNode::select(&conn_a)? {
            a.push(fs_node);
        }
        for fs_node in FsNode::select(&conn_b)? {
            b.push(fs_node);
        }
    } // drops all db connections

    compare::compare(a, b);
    Ok(())
}