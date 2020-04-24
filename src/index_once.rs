use crate::{create_tables, fs_indexer, consts};
use rusqlite::Connection;

pub fn start(db_path: &str, directories: clap::Values, force: bool) -> rusqlite::Result<()>{

    let conn = Connection::open(db_path)?;
    create_tables::execute(&conn);

    for dir in directories {
        if let Some(msg) = fs_indexer::index(&conn, dir).err() {
            eprintln!("{}: {}: could not index directory.", consts::PROGRAM_NAME, dir);
        }
    }

    conn.close().ok();

    Ok(())
}
