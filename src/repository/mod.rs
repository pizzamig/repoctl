use std::fmt;

#[derive(Debug)]
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

		enabled: is_true_string( entry.split(',').filter(|x| x.contains("enabled")).map(|x| x.split(':').last().unwrap()).last().unwrap().to_string() ),
	};
	Some(r)
}
