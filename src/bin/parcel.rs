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
		for repofile in WalkDir::new("/etc/pkg")
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
			match parse_file( buf_reader ) {
				None => println!("Not a valid repo description"),
				Some(x) => println!("{:#?}",x),
			};
		}
	}
}
