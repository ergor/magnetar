
use clap;
use crate::ConvertibleResult;

pub fn run(args: &clap::ArgMatches<'_>) -> ConvertibleResult<()>{

    let dbs = args.values_of("indexes");

    for db in dbs {
        
    }

    Ok(())
}

pub fn cmdline<'a>() -> clap::App<'a, 'a> {
    clap::App::new("dup")
        .about("Finds duplicates in indexing-run database(s).")
        .setting(clap::AppSettings::TrailingVarArg)
        .arg(clap::Arg::with_name("indexes")
            .value_name("FILES")
            .index(1)
            .help("The index database(s) to check for duplicates.")
            .required(true)
            .multiple(true))
        .arg(clap::Arg::with_name("merge")
            .long("merge")
            .short("m")
            .help("If multiple databases are given, treat them as one.")
            .takes_value(false)
            .required(false))
}