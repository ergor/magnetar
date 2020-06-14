use crate::{create_tables, fs_indexer, consts};

pub fn start(db_path: &str, directories: clap::Values<'_>, force: bool) -> crate::ConvertibleResult<()> {

    log::debug!("index_once.start: begin...");
    log::debug!("{}: opening connection to database...", db_path);
    let conn = rusqlite::Connection::open(db_path)?;
    create_tables::execute(&conn)?;
    log::debug!("{}: open OK; tables initialized", db_path);

    for dir in directories {
        log::debug!("{}: indexing recursively...", dir);
        match fs_indexer::index(dir) {
            Ok(fs_nodes) => {
                log::debug!("{}: indexing done, inserting into database...", dir);
                for fs_node in fs_nodes {
                    log::trace!("{:?}", fs_node);
                    if let Err(e) = fs_node.insert(&conn) {
                        log::error!("could not insert fsnode entry into db: {}", e);
                    }
                }
                log::debug!("{}: db insertions OK.", dir);
            },
            Err(e) => {
                log::error!("{}: could not index directory.", dir);
            },
        };
    }

    conn.close()?;
    log::debug!("{}: closed database connection.", db_path);

    Ok(())
}
