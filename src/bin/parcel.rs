extern crate clap;
extern crate walkdir;
extern crate parcel;

use clap::{Arg, App};
use std::fs::OpenOptions;
use std::io::{BufReader, BufRead};
use walkdir::WalkDir;

use parcel::repository::*;

fn trim(st: &String) -> String {
	let mut ret = String::new();
	for c in st.chars().filter( |&c| !c.is_whitespace()) {
		if c == '#' { return ret; }
		ret.push( c );
	}
	ret
}

fn main() {
	let matches = App::new("parcel - rust'ed pkg")
		.version("0.1.0")
		.author("Luca Pizzamiglio <luca.pizzamiglio@gmail.com>")
		.arg(Arg::with_name("repositories")
			.short("R")
			.long("repositories")
			.help("It checks the repository configuration")
			.takes_value(false))
		.get_matches();

	if matches.is_present("repositories") {
		for repofile in WalkDir::new("/etc/pkg")
			.into_iter()
			.filter_map( |e| e.ok())
			.filter( |e| e.file_type().is_file())
			.filter( |e| e.path().extension().unwrap() == "conf" ) {
			let f = match OpenOptions::new()
				.read( true )
				.write( false )
				.create( false )
				.open( repofile.path() ) {
				Err(_) => continue,
				Ok(f) => f,
			};
			let mut buf_reader = BufReader::new( f );
			let mut line = String::new();
			let mut entry = String::new();
			println!("file {}", repofile.path().display() );
			while buf_reader.read_line( &mut line ).unwrap() > 0 {
				//println!("read {} lines", line.len());
				entry += &(trim(&line));
				line.clear();
			}
			println!("entry: {}", entry );
			match parse_entry( entry ) {
				None => println!("Not a valid repo description"),
				Some(x) => println!("{:#?}",x),
			};
		}
	}
}
