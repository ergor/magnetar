use crate::{create_tables, fs_indexer, consts};
use crate::indexer::balancer;
use std::time::Instant;
use std::thread;

pub fn start(db_path: &str, directories: clap::Values<'_>, cpu_count: usize, force: bool) -> crate::ConvertibleResult<()> {

    let start_time = Instant::now();
    log::debug!("index_once.start: begin...");
    log::debug!("{}: opening connection to database...", db_path);
    let conn = rusqlite::Connection::open(db_path)?;
    create_tables::execute(&conn)?;
    log::debug!("{}: open OK; tables initialized", db_path);

    let directories: Vec<String> = directories.map(|v| v.to_string()).collect();
    let dirs_pretty = directories.join(", ");

    log::debug!("directories selected for indexing: '{}'", dirs_pretty);
    log::debug!("thread count: {}", cpu_count);

    let mut threads = Vec::new();
    let mut workloads = balancer::split_workload(&directories, cpu_count);

    for _ in 0..cpu_count {
        let workload = workloads.pop().unwrap();
        let thread_num = workload.thread;
        let thread = thread::spawn(move || fs_indexer::index(workload));
        threads.push(thread);
        log::trace!("thread {} spawned", thread_num);
    }

    for t in threads {
        let result = t.join()?;
        let fs_nodes = result?;
        for fs_node in fs_nodes {
            log::trace!("INSERT {:?}", fs_node);
            if let Err(e) = fs_node.insert(&conn) {
                log::error!("could not insert fsnode entry into db: {}", e);
            }
        }
    }

    log::debug!("{}: db insertions OK.", dirs_pretty);

    conn.close()?;
    log::debug!("{}: closed database connection.", db_path);
    log::debug!("index_once.start: done. total time elapsed: {} ms", start_time.elapsed().as_millis());

    Ok(())
}
