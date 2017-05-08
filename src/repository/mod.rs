use std::fmt;
use std::io::{BufReader, BufRead};
use std::fs::File;
use std::convert::From;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Repo {
	name: String,
	enabled: bool,
}

impl fmt::Display for Repo {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} [enabled:{}]", self.name, self.enabled)
	}
}

fn is_true_string( s : String ) -> bool {
	if s.starts_with( "yes" ) { return true; }
	if s.starts_with( "YES" ) { return true; }
	if s.starts_with( "Yes" ) { return true; }
	if s.starts_with( "true" ) { return true; }
	if s.starts_with( "TRUE" ) { return true; }
	if s.starts_with( "True" ) { return true; }
	false
}

fn line_trim( st: &str ) -> String {
	let mut ret = String::new();
	for c in st.chars().filter( |&c| !c.is_whitespace()) {
		if c == '#' { return ret }
		ret.push( c );
	}
	ret
}

/// Parse a string, containing only one repo description
pub fn parse_string( entry : String ) -> Option<Repo> {
	let trimmed = entry.lines().map( |x| line_trim(x)).fold( "".to_string(), |acc, x| acc + &x);
	let idx_desc_start = match trimmed.find('{') {
		None => return None,
		Some(x) => x
	};
	let idx_desc_stop = match trimmed.find('}') {
		None => return None,
		Some(x) => x
	};
	let idx_first_colon = match trimmed.find(':') {
		None => return None,
		Some(x) => x
	};
	if idx_first_colon > idx_desc_start { return None; }
	if idx_desc_stop < idx_desc_start { return None; }
	let r = Repo {
		name: match trimmed.splitn(2,':').nth(0) {
			None => "".to_string(),
			Some(x) => x.to_string(),
		},

		enabled: is_true_string( trimmed.split(',')
								.filter(|x| x.contains("enabled"))
								.map(|x| x.split(':').last().unwrap_or(""))
								.last().unwrap_or("").to_string() ),
	};
	Some(r)
}

pub fn parse_file( bf : &mut BufReader<File> ) -> Option<Repo> {
	let mut line = String::new();
	let mut entry = String::new();
	while let Ok(x) = bf.read_line( &mut line ) {
		if x == 0 { break; }
		entry += &line;
		line.clear();
	}
	parse_string( entry )
}

pub fn multi_parse_file( bf : &mut BufReader<File> ) -> Vec<Repo> {
	let mut line = String::new();
	let mut entry = String::new();
	let mut v : Vec<Repo> = Vec::new();
	while let Ok(x) = bf.read_line( &mut line ) {
		if x == 0 { break; }
		entry += &line;
		line.clear();
		if entry.find('}').is_some() {
			let to_parse = entry.clone();
			if let Some(x) = parse_string( to_parse ) {
				v.push(x);
				entry.clear();
			}
		}
	}
	v
}

/// Parse a string, containing only one repo description
impl From<String> for Repo {
	fn from( s: String) -> Repo {
		match parse_string( s ) {
			Some(x) => x,
			None =>
				Repo {
					name: "".to_string(),
					enabled: false
				}
		}
	}
}

pub fn merge_repo( v: &mut Vec<Repo>, r: Repo ) {
	if v.iter().any( |z| z.name == r.name ) {
		if let Some(x) = v.iter_mut().find( |x| x.name == r.name ) {
			x.enabled = r.enabled;
		}
	} else {
		v.push(r);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_true_string() {
		assert!(is_true_string( "yes".to_string() ) );
		assert!(is_true_string( "Yes".to_string() ) );
		assert!(is_true_string( "YES".to_string() ) );
		assert!(is_true_string( "true".to_string() ) );
		assert!(is_true_string( "True".to_string() ) );
		assert!(is_true_string( "TRUE".to_string() ) );

		assert!(! is_true_string( "no".to_string() ) );
		assert!(! is_true_string( "No".to_string() ) );
		assert!(! is_true_string( "NO".to_string() ) );
		assert!(! is_true_string( "false".to_string() ) );
		assert!(! is_true_string( "False".to_string() ) );
		assert!(! is_true_string( "FALSE".to_string() ) );

		assert!(! is_true_string( "TRue".to_string() ) );
		assert!(! is_true_string( "".to_string() ) );
	}

	#[test]
	fn test_line_trim() {
		assert_eq!( "asdf", line_trim( &(" asdf #asdf ") ) );
		assert_eq!( "", line_trim( &(" # asdf # ##  ") ) );
		assert_eq!( "asdf:asdf", line_trim( &("asdf : asdf#asdfasdf ") ) );
	}

	#[test]
	fn test_parse_string_1() {
		assert_eq!( None, parse_string( "".to_string() ) );
		assert_eq!( Some( Repo { name: "FreeBSD".to_string(), enabled: true } ),
					parse_string( "FreeBSD:{enabled:yes}".to_string() ) );
		assert_eq!( Some( Repo { name: "FreeBSD".to_string(), enabled: true } ),
					parse_string( "FreeBSD:{enabled:yes,url:\"http://pkg.bsd\"}".to_string() ) );
		assert_eq!( Some( Repo { name: "FreeBSD".to_string(), enabled: false } ),
					parse_string( "FreeBSD:{enabled:,url:\"http://pkg.bsd\"}".to_string() ) );
		assert_eq!( Some( Repo { name: "FreeBSD".to_string(), enabled: false } ),
					parse_string( "#\nFreeBSD:{\nenabled:,url:\"http://pkg.bsd\"}".to_string() ) );
	}

	#[test]
	fn test_from_1() {
		assert_eq!( Repo { name: "".to_string(), enabled: false},
					Repo::from( "".to_string() ) );
		assert_eq!( Repo { name: "FreeBSD".to_string(), enabled: true },
					Repo::from( "FreeBSD:{enabled:yes}".to_string() ) );
		assert_eq!( Repo { name: "FreeBSD".to_string(), enabled: true },
					Repo::from( "FreeBSD:{enabled:yes,url:\"http://pkg.bsd\"}".to_string() ) );
		assert_eq!( Repo { name: "FreeBSD".to_string(), enabled: false }, 
					Repo::from( "FreeBSD:{enabled:,url:\"http://pkg.bsd\"}".to_string() ) );
		assert_eq!( Repo { name: "FreeBSD".to_string(), enabled: false },
					Repo::from( "#\nFreeBSD:{\nenabled:,url:\"http://pkg.bsd\"}".to_string() ) );
	}

	#[test]
	fn test_merge_repo_1() {
		let mut repos: Vec<Repo> = Vec::new();
		let fb = "FreeBSD".to_string();
		merge_repo( &mut repos, Repo { name: fb.clone(), enabled: true } );
		assert_eq!( repos[0], Repo { name: fb.clone(), enabled: true } );

		merge_repo( &mut repos, Repo { name: fb.clone(), enabled: false } );
		assert_eq!( repos[0], Repo { name: fb.clone(), enabled: false } );

		merge_repo( &mut repos, Repo { name: "drmnext".to_string(), enabled: true } );
		assert_eq!( repos[0], Repo { name: fb.clone(), enabled: false } );
		assert_eq!( repos[1], Repo { name: "drmnext".to_string(), enabled: true } );
	}
}

