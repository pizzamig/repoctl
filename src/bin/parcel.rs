#[macro_use]
extern crate clap;
extern crate walkdir;
extern crate parcel;

use clap::App;
use std::fs::OpenOptions;
use std::io::BufReader;
use walkdir::WalkDir;

use parcel::repository::*;

fn main() {
	// Loading the arguments description & parse it
	let yaml = load_yaml!("parcel.yml");
	let matches = App::from_yaml(yaml).get_matches();

	if matches.is_present("repositories") {
		let repodirs = [ "/etc/pkg", "/usr/local/etc/pkg/repos" ];
		let mut repos: Vec<Repo> = Vec::new();
		for repodir in repodirs.into_iter() {
			for repofile in WalkDir::new(repodir)
				.into_iter()
				.filter_map( |e| e.ok())
				.filter( |e| e.file_type().is_file())
				.filter( |e| e.path().extension().unwrap_or( std::ffi::OsStr::new("")) == "conf" ) {
				let f = match OpenOptions::new()
					.read( true )
					.write( false )
					.create( false )
					.open( repofile.path() ) {
					Err(_) => continue,
					Ok(f) => f,
				};

				let buf_reader = &mut BufReader::new( f );
				for v in multi_parse_file( buf_reader ) {
					merge_repo( &mut repos, v );
				}
			}
		}
//		println!("{:?}", repos);
		println!("Enabled repos:");
		for r in repos.iter().filter(|x| x.enabled) {
			println!("repo: {}", r.name);
			println!("url: {}", r.url);
		}
	}
}
