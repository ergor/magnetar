pub mod fs_indexer;
pub mod index_once;
pub mod listener;

use clap;
use crate::consts;
use std::env;
use std::process::exit;
use std::time::SystemTime;

pub fn run(args: &clap::ArgMatches<'_>) -> crate::ConvertibleResult<()> {
    let directories = args.values_of("directories").unwrap();

    // disallow indexing of subdirectories
    for dir in directories.clone() { // TODO: naive subdir check. doesn't guard against links
        for other_dir in directories.clone() {
            if dir == other_dir {
                continue;
            }
            if dir.starts_with(other_dir) {
                log::error!("{} is subdirectory of {}. abort.", dir, other_dir);
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
        #[cfg(target_os = "linux")]
        listener::start();

        #[cfg(not(target_os = "linux"))]
        {
            log::error!("listening mode is only supported on linux (inotify)");
            exit(consts::EXIT_INVALID_ARGS);
        }
    } else {
        #[cfg(target_family = "unix")]
        index_once::start(db_path, directories)?;

        #[cfg(target_family = "windows")]
        {
            unimplemented!("{}: indexing not supported on windows yet. pull requests welcomed :)", consts::PROGRAM_NAME);
        }
    }

    Ok(())
}

pub fn cmdline<'a>() -> clap::App<'a, 'a> {
    clap::App::new("idx")
        .about("Create index of chosen directories and store in a database file.")
        .setting(clap::AppSettings::TrailingVarArg)
        .arg(clap::Arg::with_name("daemonize")
            .short("d")
            .long("daemonize")
            .help("Run the program in background")
            .takes_value(false))
        .arg(clap::Arg::with_name("output-dir")
            .short("o")
            .long("output-dir")
            .value_name("OUTPUT DIR")
            .help("Store database file in OUTPUT DIR"))
        .arg(clap::Arg::with_name("directories")
            .value_name("DIRECTORIES")
            .help("The directories to index")
            .required(true)
            .multiple(true))
}