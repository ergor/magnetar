use crate::{create_tables, fs_indexer, consts};
use rusqlite::Connection;
use crate::db_models::fs_node::FsNode;
use std::io::Error;

pub fn start(db_path: &str, directories: clap::Values, force: bool) -> crate::Result<()> {

    let conn = Connection::open(db_path)?;
    create_tables::execute(&conn)?;

    for dir in directories {
        let dir_index_result = fs_indexer::index(&conn, dir);
        match fs_indexer::index(&conn, dir) {
            Ok(fs_nodes) => {
                for fs_node in fs_nodes {
                    println!("{:?}", fs_node);
                    if let Err(e) = fs_node.insert(&conn) {
                        eprintln!("could not insert fsnode entry into db: {}", e);
                    }
                }
            },
            Err(e) => {
                eprintln!("{}: {}: could not index directory.", consts::PROGRAM_NAME, dir);
            },
        };
    }

    conn.close()?;

    Ok(())
}
