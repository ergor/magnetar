use crate::consts;
use crate::db_models::fs_node::{FsNode, NodeType};
use crate::error;
use rusqlite::Connection;
use std::fs;
use std::io::{Read};
use std::io;
use std::os::linux::fs::MetadataExt;
use std::time::SystemTime;

const READ_BUF_SZ: usize = 1024 * 1024;

pub fn index(conn: &Connection, dir_path: &str) -> io::Result<Vec<FsNode>> {
    let mut fs_nodes = Vec::new();
    let mut read_buf = [0 as u8; READ_BUF_SZ];

    let nodes = fs::read_dir(dir_path)?;

    for node in nodes {
        match process_dir_entry(node, &mut read_buf) {
            Ok(fs_node) => {
                println!("{:?}", fs_node);
                fs_nodes.push(fs_node);
            }
            Err(e) => {
                eprintln!("could not read file info: {:?}", e);
            },
        }
    }

    Ok(fs_nodes)
}

fn process_dir_entry(entry: io::Result<fs::DirEntry>, read_buf: &mut [u8]) -> crate::Result<FsNode> {
    let entry = entry?;

    let file_type = entry.file_type()?;
    let metadata = entry.metadata()?;
    let mut path = entry.path();
    path.pop(); // get parent dir

    let mut fs_node = FsNode::new();

    match entry.file_name().to_str() { // TODO: rewrite using the ?-operator when the Try trait becomes stable
        Some(file_name) => fs_node.name = String::from(file_name),
        None => return Err(error::Error::NoneError),
    };
    match path.to_str() { // TODO: rewrite using the ?-operator when the Try trait becomes stable
        Some(path_str) => fs_node.path = String::from(path_str),
        None => return Err(error::Error::NoneError),
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


    if file_type.is_dir() {
        fs_node.node_type = NodeType::Directory;
        return Ok(fs_node);
    }
    else if file_type.is_file() {
        fs_node.node_type = NodeType::File;
        fs_node.sha1_checksum = checksum(read_buf, &entry)?;
        return Ok(fs_node);
    }
    else if file_type.is_symlink() {
        fs_node.node_type = NodeType::Symlink;
        return Ok(fs_node); // TODO: how to handle symlinks?
    }

    unreachable!("{}: illegal filetype. expected a file, dir or symlink", consts::PROGRAM_NAME);
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