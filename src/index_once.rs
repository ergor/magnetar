use crate::{create_tables, fs_indexer, consts};
use rusqlite::Connection;

pub fn start(db_path: &str, directories: clap::Values, force: bool) -> crate::Result<()> {

    let conn = Connection::open(db_path)?;
    create_tables::execute(&conn)?;

    for dir in directories {
        let dir_index_result = fs_indexer::index(&conn, dir);
        if let Some(msg) = dir_index_result.err() {
            eprintln!("{}: {}: could not index directory.", consts::PROGRAM_NAME, dir);
        }
    }

    conn.close()?;

    Ok(())
}
