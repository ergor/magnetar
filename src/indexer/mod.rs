pub mod fs_indexer;
pub mod index_once;
pub mod listener;

use clap::ArgMatches;
use crate::consts;
use std::env;
use std::process::exit;
use std::time::SystemTime;

pub fn run(args: &ArgMatches) -> crate::Result<()> {
    let directories = args.values_of("directories").unwrap();
    for dir in directories.clone() { // TODO: naive subdir check. doesn't guard against links
        for other_dir in directories.clone() {
            if dir == other_dir {
                continue;
            }
            if dir.starts_with(other_dir) {
                eprintln!("{} is subdirectory of {}. abort.", dir, other_dir);
                exit(consts::EXIT_INVALID_ARGS);
            }
        }
    }

    if args.is_present("daemonize") {
        unimplemented!()
    }

    let time_now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("somehow, time now is before start of UNIX epoch");

    let db_filename = format!("{}-{}.db", consts::PROGRAM_NAME, time_now.as_secs());

    let mut tmp_dir = env::temp_dir();
    tmp_dir.push(db_filename.as_str());
    let db_path = tmp_dir.to_str()
        .expect("could not create temporary database (illegal filename)");

    if args.is_present("listen") {
        listener::start();
    } else {
        index_once::start(db_path, directories, args.is_present("force"))?;
    }

    Ok(())
}

