use crate::db_models::fs_node::{FsNode, NodeType};
use crate::errorwrapper::ErrorWrapper;
use crate::apperror::AppError;
use std::fs;
use std::io::{Read};
use std::io;
use std::os::linux::fs::MetadataExt;
use std::time::{SystemTime, Instant};

const READ_BUF_SZ: usize = 1024 * 1024;

/// Assumes you won't run this function twice on the same path.
/// I.e., you must ensure the paths you put in here are NOT subdirs of eachother.
pub fn index(dir_path: &str, cpu_count: usize) -> io::Result<Vec<FsNode>> {
    let mut fs_nodes: Vec<FsNode> = Vec::new();
    let mut read_buf = [0 as u8; READ_BUF_SZ];

    let start_time = Instant::now();
    log::debug!("{}: indexing files in directory...", dir_path);

    let dir_entries = fs::read_dir(dir_path)?;

    for dir_entry in dir_entries {
        let dir_entry = dir_entry?;
        if dir_entry.file_type()?.is_dir() {
            let children = index(dir_entry.path().to_str().unwrap(), cpu_count)?;
            children.into_iter().for_each(|n| fs_nodes.push(n));
        }
        match process_dir_entry(dir_entry, &mut read_buf) {
            Ok(fs_node) => {
                fs_nodes.push(fs_node);
            }
            Err(e) => {
                log::error!("could not read file info: {}", e);
            },
        }
    }

    log::debug!("{}: directory indexing done. time elapsed: {} ms.", dir_path, start_time.elapsed().as_millis());

    Ok(fs_nodes)
}

fn process_dir_entry(entry: fs::DirEntry, read_buf: &mut [u8]) -> crate::ConvertibleResult<FsNode> {

    if entry.path().is_relative() {
        panic!("TODO: convert relative paths to absolute paths");
    }

    let start_time = Instant::now();
    log::trace!("{}: indexing...", entry.path().to_str().unwrap_or("(unwrap error)"));

    let file_type = entry.file_type()?;
    let metadata = entry.metadata()?;

    let mut fs_node = FsNode::new();

    fs_node.node_type =
        if file_type.is_dir() {
            NodeType::Directory
        } else if file_type.is_file() {
            NodeType::File
        } else if file_type.is_symlink() {
            NodeType::Symlink
        } else {
            NodeType::Other
        };

    match entry.file_name().to_str() { // TODO: rewrite using the ?-operator when the Try trait becomes stable
        Some(file_name) => fs_node.name = String::from(file_name),
        None => return Err(ErrorWrapper::AppError(AppError::NoneError)),
    };

    fs_node.size = metadata.len() as i64;
    fs_node.uid = metadata.st_uid();
    fs_node.gid = metadata.st_gid();
    fs_node.permissions = metadata.st_mode();

    fs_node.creation_date = match metadata.created() {
        Ok(systime) => systime.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64,
        Err(_) => 0,
    };

    fs_node.modified_date = match metadata.modified() {
        Ok(systime) => systime.duration_since(SystemTime::UNIX_EPOCH)?.as_secs() as i64,
        Err(_) => 0,
    };

    fs_node.parent_path = entry.path().parent().map_or_else(
        || String::new(),
        |p| String::from(p.to_str().unwrap_or(""))
    );

    if let NodeType::Symlink = fs_node.node_type {
        match fs::read_link(entry.path())?.to_str() { // TODO: rewrite using the ?-operator when the Try trait becomes stable
            Some(path_str) => fs_node.links_to = String::from(path_str),
            None => return Err(ErrorWrapper::AppError(AppError::NoneError)),
        }
    }

    fs_node.sha1_checksum =
        if let NodeType::File = fs_node.node_type {
            String::from(checksum(read_buf, &entry)?)
        } else {
            String::new()
        };

    fs_node.inode = metadata.st_ino() as i64;
    fs_node.nlinks = metadata.st_nlink() as i64;
    // TODO: parent id

    log::trace!("{}: indexing of file done. time elapsed: {} ms.", entry.path().to_str().unwrap_or("(unwrap error)"), start_time.elapsed().as_millis());

    Ok(fs_node)
}

fn checksum(read_buf: &mut [u8], file_entry: &fs::DirEntry) -> io::Result<String> {

    let start_time = Instant::now();
    log::trace!("{}: calculating sha1 checksum...", file_entry.path().to_str().unwrap_or("(unwrap error)"));

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

    log::trace!("{}: sha1 checksum calculated. time elapsed: {} ms.", file_entry.path().to_str().unwrap_or("(unwrap error)"), start_time.elapsed().as_millis());

    Ok(sha1digest.digest().to_string())
}