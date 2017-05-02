extern crate clap;
use clap::{Arg, App, SubCommand};

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
}
