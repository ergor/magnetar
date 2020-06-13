use crate::{create_tables, fs_indexer, consts};

pub fn start(db_path: &str, directories: clap::Values<'_>, force: bool) -> crate::ConvertibleResult<()> {

    let conn = rusqlite::Connection::open(db_path)?;
    create_tables::execute(&conn)?;

    for dir in directories {
        match fs_indexer::index(dir) {
            Ok(fs_nodes) => {
                for fs_node in fs_nodes {
                    log::trace!("{:?}", fs_node);
                    if let Err(e) = fs_node.insert(&conn) {
                        log::error!("could not insert fsnode entry into db: {}", e);
                    }
                }
            },
            Err(e) => {
                log::error!("{}: could not index directory.", dir);
            },
        };
    }

    conn.close()?;

    Ok(())
}
