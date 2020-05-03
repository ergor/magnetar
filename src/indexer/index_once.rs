use crate::{create_tables, fs_indexer, consts};

pub fn start(db_path: &str, directories: clap::Values<'_>, force: bool) -> crate::Result<()> {

    let conn = rusqlite::Connection::open(db_path)?;
    create_tables::execute(&conn)?;

    for dir in directories {
        match fs_indexer::index(dir) {
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
