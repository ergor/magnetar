use rusqlite::{Connection};

pub(crate) fn execute(conn: &Connection) {
    conn.execute_batch(
        include_str!("create_tables.sql")
    ).expect("Failed to create tables");
}