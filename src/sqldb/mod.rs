extern crate rusqlite;

use self::rusqlite::*;

pub fn pkg_info() -> String {
	let mut db = match Connection::open_with_flags("/var/db/pkg/local.sqlite",SQLITE_OPEN_READ_ONLY) {
		Ok(x) => x,
		Err(_) => return "".to_string()
	};
	"pkg list".to_string()
}

