extern crate ucl;
use std::fmt;
use std::io::{BufReader, BufRead};
use std::fs::File;
use std::convert::From;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Repo {
	pub name: String,
	pub url: String,
	pub enabled: bool,
}

impl fmt::Display for Repo {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} [enabled:{}]", self.name, self.enabled)
	}
}

//fn is_true_string( s : String ) -> bool {
//	if s.starts_with( "yes" ) { return true; }
//	if s.starts_with( "YES" ) { return true; }
//	if s.starts_with( "Yes" ) { return true; }
//	if s.starts_with( "true" ) { return true; }
//	if s.starts_with( "TRUE" ) { return true; }
//	if s.starts_with( "True" ) { return true; }
//	false
//}

fn line_trim( st: &str ) -> String {
	let mut ret = String::new();
	for c in st.chars().filter( |&c| !c.is_whitespace()) {
		if c == '#' { return ret }
		ret.push( c );
	}
	ret
}

/// Input has to be already trimmed
fn get_section_name( st: &str ) -> Option<String> {
	let idx_desc_start = match st.find('{') {
		None => return None,
		Some(x) => x
	};
	let idx_first_colon = match st.find(':') {
		None => return None,
		Some(x) => x
	};
	if idx_first_colon > idx_desc_start { return None; }
	match st.splitn(2,':').nth(0) {
		None => None,
		Some(x) => Some(x.to_string()),
	}
}

/// Parse a string, containing only one repo description
//pub fn parse_string_legacy( entry : String ) -> Option<Repo> {
//	let trimmed = entry.lines().map(line_trim).fold( String::new(), |acc, x| acc + &x);
//	let idx_desc_start = match trimmed.find('{') {
//		None => return None,
//		Some(x) => x
//	};
//	let idx_desc_stop = match trimmed.find('}') {
//		None => return None,
//		Some(x) => x
//	};
//	let idx_first_colon = match trimmed.find(':') {
//		None => return None,
//		Some(x) => x
//	};
//	if idx_first_colon > idx_desc_start { return None; }
//	if idx_desc_stop < idx_desc_start { return None; }
//	let r = Repo {
//		name: match trimmed.splitn(2,':').nth(0) {
//			None => "".to_string(),
//			Some(x) => x.to_string(),
//		},
//		url: trimmed.split(',')
//				.filter(|x| x.contains("url:"))
//				.map(|x| x.split('"').nth(1).unwrap_or(""))
//				.last().unwrap_or("").to_string()
//		,
//		enabled: match trimmed.split(',')
//						.filter(|x| x.contains("enabled:"))
//						.map(|x| x.split(':').last().unwrap_or(""))
//						.last() {
//				Some(x) => is_true_string(x.to_string()),
//				None => false,
//		}
//	};
//	Some(r)
//}

pub fn parse_string( entry : String ) -> Option<Repo> {
	let trimmed = entry.lines().map(line_trim).fold( String::new(), |acc, x| acc + &x);
	let mut r: Repo = Repo {name: String::new(), url: String::new(), enabled: false };
	if let Some(name) = get_section_name( trimmed.as_ref() ) {
		r.name = name;
	} else {
		return None;
	}
	let parsy = ucl::Parser::new();
	if let Ok(config) = parsy.parse(trimmed) {
		let url_path = r.name.clone() + ".url";
		if let Some(url_obj) = config.fetch_path(url_path) {
			if let Some(url) = url_obj.as_string() {
				r.url = url;
			}
		}
		if let Some(enabled_obj) = config.fetch_path(r.name.clone() + ".enabled") {
			if let Some(enabled) = enabled_obj.as_bool() {
				r.enabled = enabled;
			}
		}
		Some(r)
	} else {
		None
	}
}

pub fn parse_file_ucl( bf : &mut BufReader<File> ) -> Option<Repo> {
	let mut line = String::new();
	let mut stringfile = String::new();
	while let Ok(x) = bf.read_line( &mut line ) {
		if x == 0 { break; }
		stringfile += &line;
		line.clear();
	}
	let parser = ucl::Parser::new();
	if let Ok(_) = parser.parse(stringfile) {
		Some( Repo {
			name: "".to_string(),
			url: "".to_string(),
			enabled: false
		})
	} else {
		None
	}
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
					url: "".to_string(),
					enabled: false
				}
		}
	}
}

pub fn merge_repo( v: &mut Vec<Repo>, r: Repo ) {
	if let Some(x) = v.iter().position( |z| z.name == r.name ) {
		v[x].enabled = r.enabled;
		if ! r.url.is_empty() {
			v[x].url = r.url;
		}
	} else {
		v.push(r);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

//	#[test]
//	fn test_is_true_string() {
//		assert!(is_true_string( "yes".to_string() ) );
//		assert!(is_true_string( "Yes".to_string() ) );
//		assert!(is_true_string( "YES".to_string() ) );
//		assert!(is_true_string( "true".to_string() ) );
//		assert!(is_true_string( "True".to_string() ) );
//		assert!(is_true_string( "TRUE".to_string() ) );
//
//		assert!(! is_true_string( "no".to_string() ) );
//		assert!(! is_true_string( "No".to_string() ) );
//		assert!(! is_true_string( "NO".to_string() ) );
//		assert!(! is_true_string( "false".to_string() ) );
//		assert!(! is_true_string( "False".to_string() ) );
//		assert!(! is_true_string( "FALSE".to_string() ) );
//
//		assert!(! is_true_string( "TRue".to_string() ) );
//		assert!(! is_true_string( "".to_string() ) );
//	}

	#[test]
	fn test_line_trim() {
		assert_eq!( "asdf", line_trim( &(" asdf #asdf ") ) );
		assert_eq!( "", line_trim( &(" # asdf # ##  ") ) );
		assert_eq!( "asdf:asdf", line_trim( &("asdf : asdf#asdfasdf ") ) );
		assert_eq!( "asdf:asdf", line_trim( &("asdf :	asdf#asdfasdf ") ) );
	}

	#[test]
	fn test_get_section_name() {
		let desc1 = "FreeBSD : { enabled: yes }";
		let rc1 = get_section_name( line_trim(desc1).as_ref() );
		assert_eq!(rc1, Some(String::from("FreeBSD")));
	}

	#[test]
	fn test_parse_string() {
		let ft: Repo = Repo { name: "FreeBSD".to_string(), url: "".to_string(), enabled: true };
		let fut: Repo = Repo { name: "FreeBSD".to_string(), url: "http://pkg.bsd".to_string(), enabled: true };
		let ff: Repo = Repo { name: "FreeBSD".to_string(), url: "".to_string(), enabled: false };
		let fuf: Repo = Repo { name: "FreeBSD".to_string(), url: "http://pkg.bsd".to_string(), enabled: false };
		let fuf2: Repo = Repo { name: "FreeBSD".to_string(), url: "http://pkg.bsd".to_string(), enabled: false };
		assert_eq!( None, parse_string( "".to_string() ) );
		assert_eq!( Some( ff ),
					parse_string( "FreeBSD:{}".to_string() ) );
		assert_eq!( Some( ft ),
					parse_string( "FreeBSD:{enabled:yes}".to_string() ) );
		assert_eq!( Some( fut ),
					parse_string( "FreeBSD:{enabled:yes,url:\"http://pkg.bsd\"}".to_string() ) );
		assert_eq!( Some( fuf ),
					parse_string( "FreeBSD:{url:\"http://pkg.bsd\"}".to_string() ) );
		assert_eq!( Some( fuf2 ),
					parse_string( "#\nFreeBSD:{\nenabled:NO,url:\"http://pkg.bsd\"}".to_string() ) );
	}

	#[test]
	fn test_from_1() {
		let ft: Repo = Repo { name: "FreeBSD".to_string(), url: "".to_string(), enabled: true };
		let fut: Repo = Repo { name: "FreeBSD".to_string(), url: "http://pkg.bsd".to_string(), enabled: true };
		let ff: Repo = Repo { name: "FreeBSD".to_string(), url: "".to_string(), enabled: false };
		let fuf: Repo = Repo { name: "FreeBSD".to_string(), url: "http://pkg.bsd".to_string(), enabled: false };
		let fuf2: Repo = Repo { name: "FreeBSD".to_string(), url: "http://pkg.bsd".to_string(), enabled: false };
		assert_eq!( Repo { name: "".to_string(), url: "".to_string(), enabled: false},
					Repo::from( "".to_string() ) );
		assert_eq!( ff, Repo::from( "FreeBSD:{}".to_string() ) );
		assert_eq!( ft, Repo::from( "FreeBSD:{enabled:yes}".to_string() ) );
		assert_eq!( fut, Repo::from( "FreeBSD:{enabled:yes,url:\"http://pkg.bsd\"}".to_string() ) );
		assert_eq!( fuf, Repo::from( "FreeBSD:{enabled:false,url:\"http://pkg.bsd\"}".to_string() ) );
		assert_eq!( fuf2, Repo::from( "#\nFreeBSD:{\nenabled:no,url:\"http://pkg.bsd\"}".to_string() ) );
	}

	#[test]
	fn test_merge_repo_1() {
		let mut repos: Vec<Repo> = Vec::new();
		let fb = "FreeBSD".to_string();
		let u = "http://10.1.3.69/103x64/libressl".to_string();
		let u2 = "http://10.1.3.69/103x64/default".to_string();
		merge_repo( &mut repos, Repo { name: fb.clone(), url: u.clone(), enabled: true } );
		assert_eq!( repos[0], Repo { name: fb.clone(), url: u.clone(), enabled: true } );

		merge_repo( &mut repos, Repo { name: fb.clone(), url: u2.clone(), enabled: false } );
		assert_eq!( repos[0], Repo { name: fb.clone(), url: u2.clone(), enabled: false } );

		merge_repo( &mut repos, Repo { name: "drmnext".to_string(), url: u.clone(), enabled: true } );
		assert_eq!( repos[0], Repo { name: fb.clone(), url: u2.clone(), enabled: false } );
		assert_eq!( repos[1], Repo { name: "drmnext".to_string(), url: u.clone(), enabled: true } );

		merge_repo( &mut repos, Repo { name: "drmnext".to_string(), url: "".to_string(), enabled: false } );
		assert_eq!( repos[0], Repo { name: fb.clone(), url: u2.clone(), enabled: false } );
		assert_eq!( repos[1], Repo { name: "drmnext".to_string(), url: u.clone(), enabled: false } );
	}

	#[test]
	fn test_ucl() {
		let parsy = ucl::Parser::new();
		let doc = parsy.parse("FreeBSD: {enabled: yes}").unwrap();
		assert_eq!( doc.get_type(), ucl::object::types::Type::Object);
		let obj = doc.fetch("FreeBSD").unwrap();
		assert_eq!(obj.get_type(), ucl::object::types::Type::Object);
		let obj2 = obj.fetch("enabled").unwrap();
		assert_eq!(obj2.get_type(), ucl::object::types::Type::Boolean);
		assert_eq!(obj2.as_bool(), Some(true) );
	}
}

