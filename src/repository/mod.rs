use std::fmt;
use std::io::{BufRead, BufReader};
use std::convert::From;
use std::path::Path;
use std::fs::OpenOptions;
use url::{Url, ParseError};
use ucl;

#[derive(PartialEq, Debug)]
pub enum MyError {
    URLError(ParseError),
    UCIError,
    NameError,
}

impl From<ParseError> for MyError {
    fn from(e: ParseError) -> MyError {
        MyError::URLError(e)
    }
}

#[derive(Debug, PartialEq)]
pub struct Repo {
    pub name: String,
    pub url: Option<Url>,
    pub enabled: bool,
}

impl Repo {
    pub fn new() -> Repo {
        Repo {
            name: String::new(),
            url: None,
            enabled: true,
        }
    }
}

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let url_section = match self.url {
            Some(ref url) => format!("url:{}", url),
            _ => "".to_string(),
        };
        write!(f, "{} [enabled:{}{}]", self.name, self.enabled, url_section)
    }
}

/// internal function that clean up a line, removing comments and whitespaces
fn line_trim(st: &str) -> String {
    let mut ret = String::new();
    for c in st.chars().filter(|&c| !c.is_whitespace()) {
        if c == '#' {
            return ret;
        }
        ret.push(c);
    }
    ret
}

/// Input has to be already trimmed
fn get_section_name(st: &str) -> Option<String> {
    let idx_desc_start = match st.find('{') {
        None => return None,
        Some(x) => x,
    };
    let idx_first_colon = match st.find(':') {
        None => return None,
        Some(x) => x,
    };
    if idx_first_colon > idx_desc_start {
        return None;
    }
    match st.splitn(2, ':').nth(0) {
        None => None,
        Some(x) => Some(x.to_string()),
    }
}

/// it parses one repository description
pub fn parse_string(entry: String) -> Result<Repo, MyError> {
    let trimmed = entry
        .lines()
        .map(line_trim)
        .fold(String::new(), |acc, x| acc + &x);
    let mut r = Repo::new();
    if let Some(name) = get_section_name(trimmed.as_ref()) {
        r.name = name;
    } else {
        return Err(MyError::NameError);
        return Err(MyError::NameError);
    }
    let parsy = ucl::Parser::new();
    if let Ok(config) = parsy.parse(trimmed) {
        let url_path = r.name.clone() + ".url";
        if let Some(url_obj) = config.fetch_path(url_path) {
            if let Some(url) = url_obj.as_string() {
                r.url = Some(Url::parse(&url)?);
            }
        }
        if let Some(enabled_obj) = config.fetch_path(r.name.clone() + ".enabled") {
            if let Some(enabled) = enabled_obj.as_bool() {
                r.enabled = enabled;
            }
        }
        Ok(r)
    } else {
        Err(MyError::UCIError)
    }
}

pub fn multi_parse_filename(filename: &Path) -> Vec<Repo> {
    let mut repos: Vec<Repo> = Vec::new();
    if let Ok(f) = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(filename)
    {
        let mut line = String::new();
        let mut entry = String::new();
        let buf_reader = &mut BufReader::new(f);
        while let Ok(x) = buf_reader.read_line(&mut line) {
            if x == 0 {
                break;
            }
            let trimmed = line_trim(&line);
            entry += &trimmed;
            line.clear();
            let open = entry.chars().filter(|x| *x == '{').count();
            if open != 0 && open == entry.chars().filter(|x| *x == '}').count() {
                if let Ok(x) = parse_string(entry.clone()) {
                    merge_repo(&mut repos, x);
                    entry.clear();
                }
            }
        }
    }
    repos
}

/// Parse a string, containing only one repo description
impl From<String> for Repo {
    fn from(s: String) -> Repo {
        match parse_string(s) {
            Ok(x) => x,
            _ => Repo::new(),
        }
    }
}

pub fn merge_repo(v: &mut Vec<Repo>, r: Repo) {
    if let Some(x) = v.iter().position(|z| z.name == r.name) {
        v[x].enabled = r.enabled;
        if r.url.is_some() {
            v[x].url = r.url;
        }
    } else {
        v.push(r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_new() {
        let r = Repo::new();
        assert_eq!(r.name, String::new());
        assert_eq!(r.url, None);
        assert!(r.enabled);
    }
    #[test]
    fn test_line_trim() {
        assert_eq!("asdf", line_trim(&(" asdf #asdf ")));
        assert_eq!("", line_trim(&(" # asdf # ##  ")));
        assert_eq!("asdf:asdf", line_trim(&("asdf : asdf#asdfasdf ")));
        assert_eq!("asdf:asdf", line_trim(&("asdf :	asdf#asdfasdf ")));
    }

    #[test]
    fn test_get_section_name() {
        let desc1 = "FreeBSD : { enabled: yes }";
        let rc1 = get_section_name(line_trim(desc1).as_ref());
        assert_eq!(rc1, Some(String::from("FreeBSD")));
    }

    #[test]
    fn test_parse_string() {
        let ft: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: None,
            enabled: true,
        };
        let fut: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: Some(Url::parse("http://pkg.bsd").unwrap()),
            enabled: true,
        };
        let ff: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: None,
            enabled: true,
        };
        let fuf: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: Some(Url::parse("http://pkg.bsd").unwrap()),
            enabled: true,
        };
        let fuf2: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: Some(Url::parse("http://pkg.bsd").unwrap()),
            enabled: false,
        };
        assert_eq!(Err(MyError::NameError), parse_string("".to_string()));
        assert_eq!(Ok(ff), parse_string("FreeBSD:{}".to_string()));
        assert_eq!(Ok(ft), parse_string("FreeBSD:{enabled:yes}".to_string()));
        assert_eq!(
            Ok(fut),
            parse_string("FreeBSD:{enabled:yes,url:\"http://pkg.bsd\"}".to_string())
        );
        assert_eq!(
            fuf,
            parse_string("FreeBSD:{url:\"http://pkg.bsd\"}".to_string()).unwrap_or(Repo::new())
        );
        assert_eq!(
            fuf2,
            parse_string("#\nFreeBSD:{\nenabled:NO,url:\"http://pkg.bsd\"}".to_string())
                .unwrap_or(Repo::new())
        );
    }

    #[test]
    fn test_from_1() {
        let ft: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: None,
            enabled: true,
        };
        let fut: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: Some(Url::parse("http://pkg.bsd").unwrap()),
            enabled: true,
        };
        let ff: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: None,
            enabled: true,
        };
        let fuf: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: Some(Url::parse("http://pkg.bsd").unwrap()),
            enabled: false,
        };
        let fuf2: Repo = Repo {
            name: "FreeBSD".to_string(),
            url: Some(Url::parse("http://pkg.bsd").unwrap()),
            enabled: false,
        };
        assert_eq!(Repo::new(), Repo::from(String::new()));
        assert_eq!(ff, Repo::from("FreeBSD:{}".to_string()));
        assert_eq!(ft, Repo::from("FreeBSD:{enabled:yes}".to_string()));
        assert_eq!(
            fut,
            Repo::from("FreeBSD:{enabled:yes,url:\"http://pkg.bsd\"}".to_string())
        );
        assert_eq!(
            fuf,
            Repo::from("FreeBSD:{enabled:false,url:\"http://pkg.bsd\"}".to_string())
        );
        assert_eq!(
            fuf2,
            Repo::from("#\nFreeBSD:{\nenabled:no,url:\"http://pkg.bsd\"}".to_string())
        );
    }

    #[test]
    fn test_merge_repo_1() {
        let mut repos: Vec<Repo> = Vec::new();
        let fb = "FreeBSD".to_string();
        let u = Some(Url::parse("http://10.1.3.69/103x64/libressl").unwrap());
        let u2 = Some(Url::parse("http://10.1.3.69/103x64/default").unwrap());
        merge_repo(
            &mut repos,
            Repo {
                name: fb.clone(),
                url: u.clone(),
                enabled: true,
            },
        );
        assert_eq!(
            repos[0],
            Repo {
                name: fb.clone(),
                url: u.clone(),
                enabled: true,
            }
        );

        merge_repo(
            &mut repos,
            Repo {
                name: fb.clone(),
                url: u2.clone(),
                enabled: false,
            },
        );
        assert_eq!(
            repos[0],
            Repo {
                name: fb.clone(),
                url: u2.clone(),
                enabled: false,
            }
        );

        merge_repo(
            &mut repos,
            Repo {
                name: "drmnext".to_string(),
                url: u.clone(),
                enabled: true,
            },
        );
        assert_eq!(
            repos[0],
            Repo {
                name: fb.clone(),
                url: u2.clone(),
                enabled: false,
            }
        );
        assert_eq!(
            repos[1],
            Repo {
                name: "drmnext".to_string(),
                url: u.clone(),
                enabled: true,
            }
        );

        merge_repo(
            &mut repos,
            Repo {
                name: "drmnext".to_string(),
                url: None,
                enabled: false,
            },
        );
        assert_eq!(
            repos[0],
            Repo {
                name: fb.clone(),
                url: u2.clone(),
                enabled: false,
            }
        );
        assert_eq!(
            repos[1],
            Repo {
                name: "drmnext".to_string(),
                url: u.clone(),
                enabled: false,
            }
        );
    }

    #[test]
    fn test_ucl() {
        let parsy = ucl::Parser::new();
        let doc = parsy.parse("FreeBSD: {enabled: yes}").unwrap();
        assert_eq!(doc.get_type(), ucl::object::types::Type::Object);
        let obj = doc.fetch("FreeBSD").unwrap();
        assert_eq!(obj.get_type(), ucl::object::types::Type::Object);
        let obj2 = obj.fetch("enabled").unwrap();
        assert_eq!(obj2.get_type(), ucl::object::types::Type::Boolean);
        assert_eq!(obj2.as_bool(), Some(true));
    }
}
