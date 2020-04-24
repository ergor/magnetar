use crate::consts;
use rusqlite::Connection;
use std::fs;
use std::io::{Error, Read};
use std::io;

const READ_BUF_SZ: usize = 128 * 1024;

pub fn index(conn: &Connection, dir_path: &str) -> io::Result<()> {
    let mut read_buf = [0 as u8; READ_BUF_SZ];
    let nodes = fs::read_dir(dir_path)?;
    for node in nodes {
        match unwrap_dir_entry(node) {
            Ok(entry) => {
                if let Some(file) = entry {
                    let sum = checksum(&mut read_buf, &file);
                    if let Ok(s) = sum {
                        println!("{}\t{}", &s, file.path().to_str().unwrap());
                    }
                }
            },
            Err(e) => {
                eprintln!("{:?}", e);
            },
        }
    }

    Ok(())
}

fn unwrap_dir_entry(entry: io::Result<fs::DirEntry>) -> io::Result<Option<fs::DirEntry>> {
    let entry = entry?;
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
        return Ok(None);
    } else if file_type.is_file() {
        return Ok(Some(entry));
    } else if file_type.is_symlink() {
        return Ok(None); // TODO: how to handle symlinks?
    } else {
        panic!("{}: illegal filetype. expected a file, dir or symlink", consts::PROGRAM_NAME);
    }
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