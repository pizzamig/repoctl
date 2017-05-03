use std::fmt;

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

pub fn parse_entry ( entry : String ) -> Option<Repo> {
	let idx_desc_start = match entry.find('{') {
		None => return None,
		Some(x) => x
	};
	let idx_desc_stop = match entry.find('}') {
		None => return None,
		Some(x) => x
	};
	let idx_first_colon = match entry.find(':') {
		None => return None,
		Some(x) => x
	};
	if idx_first_colon > idx_desc_start { return None; }
	if idx_desc_stop < idx_desc_start { return None; }
	let r = Repo {
		name: match entry.splitn(2,':').nth(0) {
			None => "".to_string(),
			Some(x) => x.to_string(),
		},

		enabled: is_true_string( entry.split(',')
								.filter(|x| x.contains("enabled"))
								.map(|x| x.split(':').last().unwrap())
								.last().unwrap().to_string() ),
	};
	Some(r)
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
	}

	#[test]
	fn test_parse_entry_1() {
		assert_eq!( None, parse_entry( "".to_string() ) );
		assert_eq!( Some( Repo { name: "FreeBSD".to_string(), enabled: true } ),
					parse_entry( "FreeBSD:{enabled:yes}".to_string() ) );
		assert_eq!( Some( Repo { name: "FreeBSD".to_string(), enabled: true } ),
					parse_entry( "FreeBSD:{enabled:yes,url:\"http://pkg.bsd\"}".to_string() ) );
	}
}
