//! Valve's key-values text (VDF), as Steam writes `steamapps/libraryfolders.vdf` and each library's
//! `appmanifest_<appid>.acf` (docs/plan.md slice 35; Valve's Game State Integration page: "iterate every library listed,
//! including the one holding libraryfolders.vdf"): quoted strings with the escapes `\\`, `\"`, `\n` and `\t`, unquoted
//! tokens, `{` `}` blocks, `//` comments and `[$WIN32]`-style conditions, which are skipped. Only what the helper reads
//! of Steam is built on it: the libraries' folders, a game's install folder and its name. A file that does not read
//! gives nothing, never a partial answer.

use std::path::PathBuf;

/// The deepest block a file may nest: Steam's files nest three or four; a deeper one is no Steam file.
const MAX_DEPTH: usize = 32;

/// A value: text, or a block of pairs in the file's order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kv {
	Str(String),
	Obj(Vec<(String, Kv)>),
}

impl Kv {
	/// The value of a block's first pair named `key`, in any case (Steam writes `AppState` and `appstate` alike); none
	/// for text.
	#[must_use]
	pub fn get(&self, key: &str) -> Option<&Kv> {
		self.pairs().iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v)
	}

	/// The text, when the value is text.
	#[must_use]
	pub fn text(&self) -> Option<&str> {
		match self {
			Self::Str(s) => Some(s),
			Self::Obj(_) => None,
		}
	}

	/// A block's pairs; none for text.
	#[must_use]
	pub fn pairs(&self) -> &[(String, Kv)] {
		match self {
			Self::Obj(pairs) => pairs,
			Self::Str(_) => &[],
		}
	}
}

/// Why a file did not read.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
	#[error("a quoted string never closes")]
	String,
	#[error("a block never closes")]
	Open,
	#[error("a closing brace closes no block")]
	Close,
	#[error("a key has no value")]
	Value,
	#[error("blocks nest deeper than {MAX_DEPTH}")]
	Deep,
}

enum Token {
	Text(String),
	Open,
	Close,
}

/// The file's tokens, in order.
fn tokens(text: &str) -> Result<Vec<Token>, Error> {
	let mut out = Vec::new();
	let mut chars = text.trim_start_matches('\u{feff}').chars().peekable();
	while let Some(c) = chars.next() {
		match c {
			c if c.is_whitespace() => {}
			'{' => out.push(Token::Open),
			'}' => out.push(Token::Close),
			'/' if chars.peek() == Some(&'/') => {
				for c in chars.by_ref() {
					if c == '\n' {
						break;
					}
				}
			}
			// a condition such as [$WIN32] after a value: no Steam file the helper reads depends on one
			'[' => {
				for c in chars.by_ref() {
					if c == ']' {
						break;
					}
				}
			}
			'"' => {
				let mut s = String::new();
				loop {
					match chars.next().ok_or(Error::String)? {
						'"' => break,
						'\\' => match chars.next().ok_or(Error::String)? {
							'n' => s.push('\n'),
							't' => s.push('\t'),
							'\\' => s.push('\\'),
							'"' => s.push('"'),
							// an unknown escape keeps its backslash, as Valve's reader does
							other => {
								s.push('\\');
								s.push(other);
							}
						},
						other => s.push(other),
					}
				}
				out.push(Token::Text(s));
			}
			first => {
				let mut s = String::from(first);
				while let Some(&c) = chars.peek() {
					if c.is_whitespace() || matches!(c, '{' | '}' | '"') {
						break;
					}
					s.push(c);
					chars.next();
				}
				out.push(Token::Text(s));
			}
		}
	}
	Ok(out)
}

/// A whole file as one block of its top-level pairs.
///
/// # Errors
/// A string or a block that never closes, a stray closing brace, a key without a value, or blocks nested past
/// `MAX_DEPTH`.
pub fn parse(text: &str) -> Result<Kv, Error> {
	// the blocks open now, each with the key it hangs under; the bottom one is the file
	let mut open: Vec<(String, Vec<(String, Kv)>)> = vec![(String::new(), Vec::new())];
	let mut key: Option<String> = None;
	for token in tokens(text)? {
		match token {
			Token::Text(t) => match key.take() {
				None => key = Some(t),
				Some(k) => open.last_mut().ok_or(Error::Close)?.1.push((k, Kv::Str(t))),
			},
			Token::Open => {
				let k = key.take().ok_or(Error::Value)?;
				if open.len() > MAX_DEPTH {
					return Err(Error::Deep);
				}
				open.push((k, Vec::new()));
			}
			Token::Close => {
				if key.is_some() {
					return Err(Error::Value);
				}
				if open.len() == 1 {
					return Err(Error::Close);
				}
				let (k, pairs) = open.pop().ok_or(Error::Close)?;
				open.last_mut().ok_or(Error::Close)?.1.push((k, Kv::Obj(pairs)));
			}
		}
	}
	if key.is_some() {
		return Err(Error::Value);
	}
	match open.pop() {
		Some((_, pairs)) if open.is_empty() => Ok(Kv::Obj(pairs)),
		_ => Err(Error::Open),
	}
}

/// The Steam libraries `libraryfolders.vdf` lists, in its order: each numbered entry's `path` (or, in the format
/// before 2021, the entry's own text). The library that holds the file is among them in Steam's own copies; the caller
/// adds it when it is not.
#[must_use]
pub fn library_paths(text: &str) -> Vec<PathBuf> {
	let Ok(file) = parse(text) else { return Vec::new() };
	let Some(folders) = file.get("libraryfolders") else { return Vec::new() };
	folders
		.pairs()
		.iter()
		.filter(|(k, _)| !k.is_empty() && k.bytes().all(|b| b.is_ascii_digit()))
		.filter_map(|(_, v)| v.get("path").and_then(Kv::text).or_else(|| v.text()))
		.filter(|p| !p.is_empty())
		.map(PathBuf::from)
		.collect()
}

/// A game's folder under its library's `steamapps/common`, as its `appmanifest_<appid>.acf` names it
/// (`AppState.installdir`); none when the file does not read or names something other than one plain folder.
#[must_use]
pub fn install_dir(acf: &str) -> Option<String> {
	let file = parse(acf).ok()?;
	let dir = file.get("AppState")?.get("installdir")?.text()?;
	let plain = !dir.is_empty() && dir != "." && dir != ".." && !dir.contains(['/', '\\', ':']);
	plain.then(|| dir.to_owned())
}

/// A game's name as its `appmanifest_<appid>.acf` holds it (`AppState.name`, Steam's own for the app; slice 48's
/// detector names a game Steam runs by it); none when the file does not read or holds no name.
#[must_use]
pub fn app_name(acf: &str) -> Option<String> {
	let file = parse(acf).ok()?;
	let name = file.get("AppState")?.get("name")?.text()?.trim();
	(!name.is_empty()).then(|| name.to_owned())
}
