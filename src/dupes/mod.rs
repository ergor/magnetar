
use clap;

pub fn cmdline<'a>() -> clap::App<'a, 'a> {
    clap::App::new("dupes")
        .about("Finds duplicates in indexing-run database(s).")
        .setting(clap::AppSettings::TrailingVarArg)
        .arg(clap::Arg::with_name("indexes")
            .value_name("FILES")
            .index(1)
            .help("The index database(s) to check for duplicates.")
            .required(true)
            .multiple(true))
}