/*!
Ini streaming parser
====================

Simple, straight forward, super fast, `no_std` compatible INI parser.

Examples
--------

```
use ini_core as ini;

let document = "\
[SECTION]
;this is a comment
Key=Value";

let elements = [
	ini::Item::Section("SECTION"),
	ini::Item::Comment("this is a comment"),
	ini::Item::Property("Key", "Value"),
];

for (line, item) in ini::Parser::new(document).enumerate() {
	assert_eq!(item, elements[line]);
}
```

The parser is very much line-based, it will continue no matter what and return nonsense as an item:

```
use ini_core as ini;

let document = "\
[SECTION
nonsense";

let elements = [
	ini::Item::Error("[SECTION"),
	ini::Item::Action("nonsense"),
];

for (line, item) in ini::Parser::new(document).enumerate() {
	assert_eq!(item, elements[line]);
}
```

Lines starting with `[` but contain either no closing `]` or a closing `]` not followed by a newline are returned as [`Item::Error`].
Lines missing a `=` are returned as [`Item::Action`]. See below for more details.

Format
------

INI is not a well specified format, this parser tries to make as little assumptions as possible but it does make decisions.

* Newline is either `"\r\n"`, `"\n"` or `"\r"`. It can be mixed in a single document but this is not recommended.
* Section header is `"[" section "]" newline`. `section` can be anything except contain newlines.
* Property is `key "=" value newline`. `key` and `value` can be anything except contain newlines.
* Comment is `";" comment newline` and Blank is just `newline`. The comment character can be customized.

Note that padding whitespace is not trimmed by default:
Section `[ SECTION ]`'s name is `<space>SECTION<space>`.
Property `KEY = VALUE` has key `KEY<space>` and value `<space>VALUE`.
Comment `; comment`'s comment is `<space>comment`.

No further processing of the input is done, eg. if escape sequences are necessary they must be processed by the user.
*/

#![cfg_attr(not(test), no_std)]

#[allow(unused_imports)]
use core::{fmt, str};

// All the routines here work only with and slice only at ascii characters
// This means conversion between `&str` and `&[u8]` is a noop even when slicing
#[inline]
fn from_utf8(v: &[u8]) -> &str {
	#[cfg(not(debug_assertions))]
	return unsafe { str::from_utf8_unchecked(v) };
	#[cfg(debug_assertions)]
	return str::from_utf8(v).unwrap();
}

// LLVM is big dum dum, trust me I'm a human
#[cfg(not(debug_assertions))]
macro_rules! unsafe_assert {
	($e:expr) => { unsafe { if !$e { ::core::hint::unreachable_unchecked(); } } };
}
#[cfg(debug_assertions)]
macro_rules! unsafe_assert {
	($e:expr) => {};
}

mod parse;

/// Ini element.
///
/// IMPORTANT! Values are not checked or escaped when displaying the item.
/// Ensure that the values do not contain newlines or invalid characters.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Item<'a> {
	/// Syntax error.
	///
	/// Section header element was malformed.
	Error(&'a str),

	/// Section header element.
	///
	/// Eg. `[SECTION]` results in `Item::Section("SECTION")`.
	///
	/// Section value must not contain `[` or `]`.
	Section(&'a str),

	/// Property element.
	///
	/// Eg. `KEY=VALUE` results in `Item::Property("KEY", Some("VALUE"))`.
	///
	/// Key value must not contain `=`.
	Property(&'a str, &'a str),

	/// Property without value.
	///
	/// Eg. `ACTION` results in `Item::Property("ACTION")`.
	///
	/// Action value must not contain `=`.
	Action(&'a str),

	/// Comment.
	///
	/// Eg. `;comment` results in `Item::Comment("comment")`.
	Comment(&'a str),

	/// Blank line.
	///
	/// Allows faithful reproduction of the whole ini file including blank lines.
	Blank,
}

impl<'a> fmt::Display for Item<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			&Item::Error(error) => write!(f, "{}\n", error),
			&Item::Section(section) => write!(f, "[{}]\n", section),
			&Item::Property(key, value) => write!(f, "{}={}\n", key, value),
			&Item::Action(action) => write!(f, "{}\n", action),
			&Item::Comment(comment) => write!(f, ";{}\n", comment),
			&Item::Blank => f.write_str("\n"),
		}
	}
}

/// Trims ascii whitespace from the start and end of the string slice.
///
/// See also [`Parser::auto_trim`] to automatically trim strings.
pub fn trim(s: &str) -> &str {
	s.trim_matches(|chr: char| chr.is_ascii_whitespace())
}

/// Ini streaming parser.
///
/// The whole document must be available before parsing starts.
/// The parser then returns each element as it is being parsed.
///
/// See [`crate`] documentation for more information.
#[derive(Clone, Debug)]
pub struct Parser<'a> {
	line: u32,
	comment_char: u8,
	auto_trim: bool,
	state: &'a [u8],
}

impl<'a> Parser<'a> {
	/// Constructs a new `Parser` instance.
	#[inline]
	pub const fn new(s: &'a str) -> Parser<'a> {
		let state = s.as_bytes();
		Parser { line: 0, comment_char: b';', auto_trim: false, state }
	}

	/// Sets the comment character, eg. `b'#'`.
	///
	/// The default is `b';'`.
	#[inline]
	pub const fn comment_char(self, chr: u8) -> Parser<'a> {
		// Mask off high bit to ensure we don't corrupt utf8 strings
		let comment_char = chr & 0x7f;
		Parser { comment_char, ..self }
	}

	/// Sets auto trimming of all returned strings.
	///
	/// The default is `false`.
	#[inline]
	pub const fn auto_trim(self, auto_trim: bool) -> Parser<'a> {
		Parser { auto_trim, ..self }
	}

	/// Returns the line number the parser is currently at.
	#[inline]
	pub const fn line(&self) -> u32 {
		self.line
	}

	/// Returns the remainder of the input string.
	#[inline]
	pub fn remainder(&self) -> &'a str {
		from_utf8(self.state)
	}
}

impl<'a> Iterator for Parser<'a> {
	type Item = Item<'a>;

	// #[cfg_attr(test, mutagen::mutate)]
	#[inline(never)]
	fn next(&mut self) -> Option<Item<'a>> {
		let mut s = self.state;

		match s.first().cloned() {
			// Terminal case
			None => None,
			// Blank
			Some(b'\r' | b'\n') => {
				self.skip_ln(s);
				Some(Item::Blank)
			},
			// Comment
			Some(chr) if chr == self.comment_char => {
				s = &s[1..];
				let i = parse::find_nl(s);
				unsafe_assert!(i <= s.len());
				let comment = from_utf8(&s[..i]);
				let comment = if self.auto_trim { trim(comment) } else { comment };
				self.skip_ln(&s[i..]);
				Some(Item::Comment(comment))
			},
			// Section
			Some(b'[') => {
				let i = parse::find_nl(s);
				unsafe_assert!(i >= 1 && i <= s.len());
				if s[i - 1] != b']' {
					let error = from_utf8(&s[..i]);
					self.skip_ln(&s[i..]);
					return Some(Item::Error(error));
				}
				unsafe_assert!(1 <= i - 1);
				let section = from_utf8(&s[1..i - 1]);
				let section = if self.auto_trim { trim(section) } else { section };
				self.skip_ln(&s[i..]);
				Some(Item::Section(section))
			},
			// Property
			_ => {
				let key = {
					let i = parse::find_nl_chr(s, b'=');
					unsafe_assert!(i <= s.len());
					if s.get(i) != Some(&b'=') {
						let action = from_utf8(&s[..i]);
						let action = if self.auto_trim { trim(action) } else { action };
						self.skip_ln(&s[i..]);
						return Some(Item::Action(action));
					}
					unsafe_assert!(i + 1 <= s.len());
					let key = from_utf8(&s[..i]);
					let key = if self.auto_trim { trim(key) } else { key };
					s = &s[i + 1..];
					key
				};
				let value = {
					let i = parse::find_nl(s);
					unsafe_assert!(i <= s.len());
					let value = from_utf8(&s[..i]);
					let value = if self.auto_trim { trim(value) } else { value };
					self.skip_ln(&s[i..]);
					value
				};
				Some(Item::Property(key, value))
			},
		}
	}
}

impl<'a> Parser<'a> {
	#[inline]
	fn skip_ln(&mut self, mut s: &'a [u8]) {
		if s.len() > 0 {
			if s[0] == b'\r' {
				s = &s[1..];
			}
			if s.len() > 0 {
				if s[0] == b'\n' {
					s = &s[1..];
				}
			}
			self.line += 1;
		}
		self.state = s;
	}
}

#[cfg(test)]
mod tests;
