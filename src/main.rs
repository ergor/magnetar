mod create_tables;
mod fs_indexer;

use clap::{App, Arg, AppSettings};
use rusqlite::{Connection, params};
use std::env;
use std::time::{SystemTime};

const MAGNETAR_NAME: &str = "magnetar";

fn main() -> rusqlite::Result<()> {
    let args = App::new(MAGNETAR_NAME)
        .setting(AppSettings::TrailingVarArg)
        .version(clap::crate_version!())
        //.author(clap::crate_authors!())
        .about("filesystem indexer client")
        .arg(Arg::with_name("daemonize")
            .short("d")
            .long("daemonize")
            .help("Run the program in background")
            .takes_value(false))
        .arg(Arg::with_name("directories")
            .required(true)
            .multiple(true))
        .get_matches();

    let directories = args.values_of("directories").unwrap();
    println!("{:?}", directories);

    // TODO: check that no dir is subdir of other

    let time_now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("somehow, time now is before start of UNIX epoch");

    let db_filename = format!("{}-{}.db", MAGNETAR_NAME, time_now.as_secs());

    let mut tmp_dir = env::temp_dir();
    tmp_dir.push(db_filename.as_str());
    let tmp_dir_path = tmp_dir.to_str().unwrap();

    let conn = Connection::open(tmp_dir_path)?;
    create_tables::execute(&conn);

    for dir in directories {
        if let Some(msg) = fs_indexer::index(&conn, dir).err() {
            eprintln!("{}: {}: could not index directory.", MAGNETAR_NAME, dir);
        }
    }

    conn.close().ok();

    return Ok(());
}
