use crate::{create_tables, fs_indexer};
use std::time::Instant;

pub fn start(db_path: &str, directories: clap::Values<'_>) -> crate::ConvertibleResult<()> {

    let start_time = Instant::now();
    log::debug!("index_once.start: begin...");

    log::debug!("'{}': opening connection to database...", db_path);
    let mut conn = rusqlite::Connection::open(db_path)?;
    create_tables::execute(&conn)?;
    log::debug!("'{}': open OK; tables initialized", db_path);

    log::debug!("'{}': beginning transaction...", db_path);
    let transaction = conn.transaction()?;

    let directories: Vec<String> = directories.map(|v| v.to_string()).collect();
    log::debug!("directories selected for indexing: '{}'", directories.join(", "));
    for dir in directories {
        match fs_indexer::depth_first_indexer(dir.as_str()) {
            Ok(fs_nodes) => {
                log::debug!("'{}': indexing done, inserting into database...", dir);
                for fs_node in fs_nodes {
                    log::trace!("INSERT {:?}", fs_node);
                    if let Err(e) = fs_node.insert(&transaction) {
                        log::error!("could not insert fsnode entry into db: {}. {:?}", e, fs_node);
                    }
                }
                log::debug!("'{}': db insertions OK.", dir);
            },
            Err(e) => {
                log::warn!("'{}': abort indexing of directory. reason: {}", dir, e);
            },
        };
    }

    transaction.commit()?;

    conn.close()?;
    log::debug!("{}: closed database connection.", db_path);
    log::debug!("index_once.start: done. total time elapsed: {} ms", start_time.elapsed().as_millis());

    Ok(())
}
