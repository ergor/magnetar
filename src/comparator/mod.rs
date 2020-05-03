pub mod compare;
pub mod comparison;

use clap::ArgMatches;
use crate::db_models::fs_node::FsNode;
use std::path::Path;

pub fn run(args: &ArgMatches) -> crate::Result<()> {
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

    // TODO: check that both dbs exists
    let output_dir = args.value_of("directory").unwrap();

    let mut a = Vec::new();
    let mut b = Vec::new();

    { // open for db work
        let conn_a = rusqlite::Connection::open(first_index)?;
        let conn_b = rusqlite::Connection::open(second_index)?;

        // TODO: try rewrite with explicit columns in select to fix InvalidColumnType error
        let mut stmt_a = conn_a.prepare("SELECT * FROM fs_node")?;
        let mut stmt_b = conn_b.prepare("SELECT * FROM fs_node")?;

        let stmt_a_iter = stmt_a.query_map(rusqlite::NO_PARAMS, |row| FsNode::map_from_row(row))?;
        let stmt_b_iter = stmt_b.query_map(rusqlite::NO_PARAMS, |row| FsNode::map_from_row(row))?;

        for fs_node in stmt_a_iter {
            let fs_node = fs_node?;
            a.push(fs_node);
        }
        for fs_node in stmt_b_iter {
            let fs_node = fs_node?;
            b.push(fs_node);
        }
    } // drops all db connections

    println!("a\n-----------------------{:?}\n\nb\n-----------------------{:?}", a, b);

    compare::compare(a, b);
    Ok(())
}