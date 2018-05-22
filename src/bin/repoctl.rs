#[macro_use]
extern crate clap;
extern crate repoctl;
extern crate syslog;
extern crate walkdir;

use clap::{App, AppSettings};
use walkdir::WalkDir;
use repoctl::repository::*;
use syslog::{Facility, Severity};

fn main() {
    // Loading the arguments description & parse it
    let yaml = load_yaml!("repoctl.yml");
    let matches = App::from_yaml(yaml)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .get_matches();

    let mut verbosity = matches.occurrences_of("verbose");
    //	let logger = match syslog::tcp("127.0.0.1","localhost".to_string(),Facility::LOG_LOCAL5) {
    let logger = match syslog::unix(Facility::LOG_LOCAL5) {
        Err(e) => panic!("Impossible to connect to syslog: {:?}", e),
        Ok(writer) => writer,
    };

    match matches.subcommand() {
        ("show", Some(show_matches)) => {
            let mut all = false;
            verbosity += show_matches.occurrences_of("verbose");
            match show_matches.subcommand() {
                ("all", Some(all_matches)) => {
                    all = true;
                    verbosity += all_matches.occurrences_of("verbose");
                }
                (boh, _) => {
                    println!("command {} unknown", boh);
                    println!("{}", show_matches.usage()); // it's not working :(
                }
            }
            let repodirs = ["/etc/pkg", "/usr/local/etc/pkg/repos"];
            let mut repos: Vec<Repo> = Vec::new();
            for repodir in &repodirs {
                for repofile in WalkDir::new(repodir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .filter(|e| e.path().extension().unwrap_or_else(|| std::ffi::OsStr::new("")) == "conf")
                {
                    let log_msg = format!("Parsing file: {:?}", repofile.path());
                    if verbosity > 0 {
                        match logger.send_3164(Severity::LOG_WARNING, log_msg) {
                            Ok(n) => println!("log sent ({} bytes)", n),
                            Err(e) => println!("log error {:?}", e),
                        };
                        println!("Parsing file: {:?}", repofile.path());
                    } else {
                        logger.send_3164(Severity::LOG_DEBUG, log_msg);
                    }
                    for r in multi_parse_filename(repofile.path()) {
                        merge_repo(&mut repos, r);
                    }
                }
            }
            if all {
                println!("Available repos:");
                for r in &repos {
                    println!("{}", r);
                }
            } else {
                println!("Enabled repos:");
                for r in repos.iter().filter(|x| x.enabled) {
                    println!("{}", r);
                }
            }
        }
        (boh, _) => {
            println!("command {} unknown", boh);
            println!("{}", matches.usage()); // it's not working :(
        }
    }
}
