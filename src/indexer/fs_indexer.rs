use crate::db_models::fs_node::{FsNode, NodeType};
use crate::error::{ErrorWrapper, AppError};
use std::fs;
use std::io::{Read};
use std::io;
use std::os::linux::fs::MetadataExt;
use std::time::SystemTime;

const READ_BUF_SZ: usize = 1024 * 1024;

/// Assumes you won't run this function twice on the same path.
/// I.e., you must ensure the paths you put in here are NOT subdirs of eachother.
pub fn index(dir_path: &str) -> io::Result<Vec<FsNode>> {
    let mut fs_nodes = Vec::new();
    let mut read_buf = [0 as u8; READ_BUF_SZ];

    let nodes = fs::read_dir(dir_path)?;

    for node in nodes {
        match process_dir_entry(node, &mut read_buf) {
            Ok(fs_node) => {
                fs_nodes.push(fs_node);
            }
            Err(e) => {
                eprintln!("could not read file info: {:?}", e);
            },
        }
    }

    Ok(fs_nodes)
}

fn process_dir_entry(entry: io::Result<fs::DirEntry>, read_buf: &mut [u8]) -> crate::ConvertibleResult<FsNode> {
    let entry = entry?;

    if entry.path().is_relative() {
        panic!("TODO: convert relative paths to absolute paths");
    }

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

    Ok(fs_node)
}

fn checksum(read_buf: &mut [u8], file_entry: &fs::DirEntry) -> io::Result<String> {
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

    Ok(sha1digest.digest().to_string())
}