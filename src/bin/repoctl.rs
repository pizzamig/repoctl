extern crate structopt;
extern crate repoctl;
extern crate syslog;
extern crate walkdir;

use repoctl::repository::*;
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
#[structopt(about = "A tool to manage FreeBSD repositories definition for pkg(8)")]
struct Opt {
    /// Increase verbosity. It can be used multiple times
    #[structopt(short = "v", parse(from_occurrences))]
    verbosity: u64,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// It shows all available repositories
    #[structopt(name = "show")]
    Show,
}

fn main() {
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Show => {
            let repodirs = ["/etc/pkg", "/usr/local/etc/pkg/repos"];
            let mut repos: Vec<Repo> = Vec::new();
            for repodir in &repodirs {
                for repofile in WalkDir::new(repodir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .unwrap_or_else(|| std::ffi::OsStr::new(""))
                            == "conf"
                    }) {
                    println!("Parsing file: {:?}", repofile.path());
                    for r in multi_parse_filename(repofile.path()) {
                        merge_repo(&mut repos, r);
                    }
                }
            }
            println!("Available repos:");
            for r in &repos {
                println!("{}", r);
            }
        }
    }
}
