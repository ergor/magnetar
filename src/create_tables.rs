use rusqlite::{Connection};

pub fn execute(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        include_str!("create_tables.sql")
    )
}