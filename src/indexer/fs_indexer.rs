use crate::db_models::fs_node::{FsNode, NodeType};
use crate::errorwrapper::ErrorWrapper;
use crate::apperror::AppError;
use std::fs;
use std::io::{Read};
use std::io;
use std::os::linux::fs::MetadataExt;
use std::time::{SystemTime, Instant};
use std::fs::{DirEntry, ReadDir};
use std::cell::RefCell;

const READ_BUF_SZ: usize = 1024 * 1024;

/// Assumes you won't run this function twice on the same path.
/// I.e., you must ensure the paths you put in here are NOT subdirs of eachother.
pub fn depth_first_indexer(dir_path: &str) -> io::Result<Vec<FsNode>> {
    let mut fs_nodes: Vec<FsNode> = Vec::new();
    let mut read_buf = [0 as u8; READ_BUF_SZ];
    let mut dir_iter_stack: Vec<RefCell<ReadDir>> = Vec::new();
    let mut visit_log_stack: Vec<(Instant, String)> = Vec::new(); // for logging purposes

    let start_time = Instant::now();
    log::debug!("depth_first_indexer: '{}': start...", dir_path);

    let root_level_entries_iter = fs::read_dir(dir_path)?;
    dir_iter_stack.push(RefCell::new(root_level_entries_iter));
    visit_log_stack.push((Instant::now(), dir_path.to_string()));

    while !dir_iter_stack.is_empty() {
        let current_dir_iter = dir_iter_stack.last().unwrap(); // we know it's Some, because of loop condition
        let next_child = current_dir_iter.borrow_mut().next();
        if next_child.is_some() {
            let child: DirEntry = next_child.unwrap()?;

            let fs_node = process_single_dir_entry(&child, &mut read_buf);
            fs_nodes.push(fs_node);

            if child.file_type()?.is_dir() {
                let child_path = child.path();
                let child_path_lossy = child_path.to_string_lossy();
                log::debug!("'{}': now descending into...", child_path_lossy);
                dir_iter_stack.push(
                    RefCell::new(fs::read_dir(child.path())?)
                );
                visit_log_stack.push((Instant::now(), child_path_lossy.to_string()));
            }
        }
        else {
            let (time_elapsed, visited_path) = match visit_log_stack.pop() {
                Some((time, path)) => (time.elapsed().as_millis(), path),
                None => (u128::max_value(), "(unwrap error)".to_string())
            };
            log::debug!("'{}': directory indexing done. time elapsed: {} ms.", visited_path, time_elapsed);
            dir_iter_stack.pop();
        }
    }

    log::debug!("depth_first_indexer: '{}': done. time elapsed: {} ms.", dir_path, start_time.elapsed().as_millis());

    Ok(fs_nodes)
}

fn process_single_dir_entry(entry: &fs::DirEntry, read_buf: &mut [u8]) -> FsNode {

    if entry.path().is_relative() {
        panic!("TODO: convert relative paths to absolute paths");
    }


    let entry_path = entry.path();
    let entry_path_lossy = entry_path.to_string_lossy();
    let start_time = Instant::now();

    log::trace!("'{}': collecting file metadata...", entry_path_lossy);

    let mut fs_node = FsNode::new();

    fs_node.name = entry_path_lossy.clone().to_string();
    fs_node.node_type =
        match entry.file_type() {
            Ok(ft) => {
                if ft.is_dir() {
                    NodeType::Directory
                } else if ft.is_file() {
                    NodeType::File
                } else if ft.is_symlink() {
                    NodeType::Symlink
                } else {
                    NodeType::Other
                }
            },
            Err(e) => {
                log::warn!("'{}': could not read node type: {}", entry_path_lossy, e);
                NodeType::Error
            }
        };

    fn date_to_i64(path_for_log: &str, date: io::Result<SystemTime>) -> i64 {
        match date {
            Ok(systime) =>
                systime.duration_since(SystemTime::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs() as i64),
            Err(e) => {
                log::warn!("'{}': could not read date: {}", path_for_log, e);
                0
            },
        }
    }

    match entry.metadata() {
        Ok(metadata) => {
            fs_node.size = metadata.len() as i64;
            fs_node.uid = metadata.st_uid();
            fs_node.gid = metadata.st_gid();
            fs_node.permissions = metadata.st_mode();
            fs_node.inode = metadata.st_ino() as i64;
            fs_node.nlinks = metadata.st_nlink() as i64;
            fs_node.creation_date = date_to_i64(entry_path_lossy.as_ref(), metadata.created());
            fs_node.modified_date = date_to_i64(entry_path_lossy.as_ref(), metadata.modified());
        },
        Err(e) => {
            log::warn!("'{}': could not read metadata: {}", entry_path_lossy, e);
        }
    }

    fs_node.parent_path = entry.path().parent().map_or_else(
        || String::new(), // root or relative path
        |p| String::from(p.to_string_lossy())
    );

    if let NodeType::Symlink = fs_node.node_type {
        match fs::read_link(entry.path()) {
            Ok(path) => fs_node.links_to = path.to_string_lossy().to_string(),
            Err(e) => log::warn!("'{}': could not resolve symlink path: {}", entry_path_lossy, e),
        }
    }

    fs_node.sha1_checksum =
        if let NodeType::File = fs_node.node_type {
            String::from(checksum(read_buf, &entry).unwrap_or("[ERR]".to_string()))
        } else {
            String::new()
        };
    // TODO: parent id

    log::trace!("'{}': indexing of file done. time elapsed: {} ms.", entry_path_lossy, start_time.elapsed().as_millis());

    fs_node
}

fn checksum(read_buf: &mut [u8], file_entry: &fs::DirEntry) -> io::Result<String> {

    let start_time = Instant::now();
    log::trace!("'{}': calculating sha1 checksum...", file_entry.path().to_string_lossy());

    let mut file = fs::File::open(file_entry.path())?;
    let mut sha1digest = sha1::Sha1::new();

    loop {
        let bytes_read = file.read(read_buf)?;
        if bytes_read > 0 {
            sha1digest.update(&read_buf[..bytes_read]);
        } else {
            break;
        }
    }

    log::trace!("'{}': sha1 checksum calculated. time elapsed: {} ms.", file_entry.path().to_string_lossy(), start_time.elapsed().as_millis());

    Ok(sha1digest.digest().to_string())
}