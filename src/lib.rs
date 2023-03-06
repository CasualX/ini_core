/*!
Ini streaming parser
====================

Simple, straight forward, super fast, `no_std` compatible streaming INI parser.

Examples
--------

```
use ini_core as ini;

let document = "\
[SECTION]
;this is a comment
Key=Value";

let elements = [
	ini::Item::SectionEnd,
	ini::Item::Section("SECTION"),
	ini::Item::Comment("this is a comment"),
	ini::Item::Property("Key", Some("Value")),
	ini::Item::SectionEnd,
];

for (index, item) in ini::Parser::new(document).enumerate() {
	assert_eq!(item, elements[index]);
}
```

The `SectionEnd` pseudo element is returned before a new section and at the end of the document.
This helps processing sections after their properties finished parsing.

The parser is very much line-based, it will continue no matter what and return nonsense as an item:

```
use ini_core as ini;

let document = "\
[SECTION
nonsense";

let elements = [
	ini::Item::SectionEnd,
	ini::Item::Error("[SECTION"),
	ini::Item::Property("nonsense", None),
	ini::Item::SectionEnd,
];

for (index, item) in ini::Parser::new(document).enumerate() {
	assert_eq!(item, elements[index]);
}
```

Lines starting with `[` but contain either no closing `]` or a closing `]` not followed by a newline are returned as [`Item::Error`].
Lines missing a `=` are returned as [`Item::Property`] with `None` value. See below for more details.

Format
------

INI is not a well specified format, this parser tries to make as little assumptions as possible but it does make decisions.

* Newline is either `"\r\n"`, `"\n"` or `"\r"`. It can be mixed in a single document but this is not recommended.
* Section header is `"[" section "]" newline`. `section` can be anything except contain newlines.
* Property is `key "=" value newline`. `key` and `value` can be anything except contain newlines.
* Comment is `";" comment newline` and Blank is just `newline`. The comment character can be customized.

Note that padding whitespace is not trimmed by default:

* Section `[ SECTION ]`'s name is `<space>SECTION<space>`.
* Property `KEY = VALUE` has key `KEY<space>` and value `<space>VALUE`.
* Comment `; comment`'s comment is `<space>comment`.

No further processing of the input is done, eg. if escape sequences are necessary they must be processed by the caller.
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

mod parse;

/// Ini element.
///
/// # Notes
///
/// Strings are not checked or escaped when displaying the item.
///
/// Ensure that they do not contain newlines or invalid characters.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Item<'a> {
	/// Syntax error.
	///
	/// Section header element was malformed.
	/// Malformed section headers are defined by a line starting with `[` but not ending with `]`.
	///
	/// ```
	/// assert_eq!(
	/// 	ini_core::Parser::new("[Error").nth(1),
	/// 	Some(ini_core::Item::Error("[Error")));
	/// ```
	Error(&'a str),

	/// Section header element.
	///
	/// ```
	/// assert_eq!(
	/// 	ini_core::Parser::new("[Section]").nth(1),
	/// 	Some(ini_core::Item::Section("Section")));
	/// ```
	Section(&'a str),

	/// End of section.
	///
	/// Pseudo element emitted before a [`Section`](Item::Section) and at the end of the document.
	/// This helps processing sections after their properties finished parsing.
	///
	/// ```
	/// assert_eq!(
	/// 	ini_core::Parser::new("").next(),
	/// 	Some(ini_core::Item::SectionEnd));
	/// ```
	SectionEnd,

	/// Property element.
	///
	/// Key value must not contain `=`.
	///
	/// The value is `None` if there is no `=`.
	///
	/// ```
	/// assert_eq!(
	/// 	ini_core::Parser::new("Key=Value").next(),
	/// 	Some(ini_core::Item::Property("Key", Some("Value"))));
	/// assert_eq!(
	/// 	ini_core::Parser::new("Key").next(),
	/// 	Some(ini_core::Item::Property("Key", None)));
	/// ```
	Property(&'a str, Option<&'a str>),

	/// Comment.
	///
	/// ```
	/// assert_eq!(
	/// 	ini_core::Parser::new(";comment").next(),
	/// 	Some(ini_core::Item::Comment("comment")));
	/// ```
	Comment(&'a str),

	/// Blank line.
	///
	/// Allows faithful reproduction of the whole ini document including blank lines.
	///
	/// ```
	/// assert_eq!(
	/// 	ini_core::Parser::new("\n").next(),
	/// 	Some(ini_core::Item::Blank));
	/// ```
	Blank,
}

impl<'a> fmt::Display for Item<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Item::Error(error) => writeln!(f, "{}", error),
			Item::Section(section) => writeln!(f, "[{}]", section),
			Item::SectionEnd => Ok(()),
			Item::Property(key, Some(value)) => writeln!(f, "{}={}", key, value),
			Item::Property(key, None) => writeln!(f, "{}", key),
			Item::Comment(comment) => writeln!(f, ";{}", comment),
			Item::Blank => f.write_str("\n"),
		}
	}
}

/// Trims ascii whitespace from the start and end of the string slice.
///
/// See also [`Parser::auto_trim`] to automatically trim strings.
#[inline(never)]
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
	section_ended: bool,
	state: &'a [u8],
}

impl<'a> Parser<'a> {
	/// Constructs a new `Parser` instance.
	#[inline]
	pub const fn new(s: &'a str) -> Parser<'a> {
		let state = s.as_bytes();
		Parser { line: 0, comment_char: b';', auto_trim: false, section_ended: false, state }
	}

	/// Sets the comment character, eg. `b'#'`.
	///
	/// The default is `b';'`.
	#[must_use]
	#[inline]
	pub const fn comment_char(self, chr: u8) -> Parser<'a> {
		// Mask off high bit to ensure we don't corrupt utf8 strings
		let comment_char = chr & 0x7f;
		Parser { comment_char, ..self }
	}

	/// Sets auto trimming of all returned strings.
	///
	/// The default is `false`.
	#[must_use]
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
			None => {
				if self.section_ended {
					None
				}
				else {
					self.section_ended = true;
					Some(Item::SectionEnd)
				}
			},
			// Blank
			Some(b'\r' | b'\n') => {
				self.skip_ln(s);
				Some(Item::Blank)
			},
			// Comment
			Some(chr) if chr == self.comment_char => {
				s = &s[1..];
				let i = parse::find_nl(s);
				let comment = from_utf8(&s[..i]);
				let comment = if self.auto_trim { trim(comment) } else { comment };
				self.skip_ln(&s[i..]);
				Some(Item::Comment(comment))
			},
			// Section
			Some(b'[') => {
				if self.section_ended {
					self.section_ended = false;
					let i = parse::find_nl(s);
					if s[i - 1] != b']' {
						let error = from_utf8(&s[..i]);
						self.skip_ln(&s[i..]);
						return Some(Item::Error(error));
					}
					let section = from_utf8(&s[1..i - 1]);
					let section = if self.auto_trim { trim(section) } else { section };
					self.skip_ln(&s[i..]);
					Some(Item::Section(section))
				}
				else {
					self.section_ended = true;
					Some(Item::SectionEnd)
				}
			},
			// Property
			_ => {
				let key = {
					let i = parse::find_nl_chr(s, b'=');
					let key = from_utf8(&s[..i]);
					let key = if self.auto_trim { trim(key) } else { key };
					if s.get(i) != Some(&b'=') {
						self.skip_ln(&s[i..]);
						if key.is_empty() {
							return Some(Item::Blank);
						}
						return Some(Item::Property(key, None));
					}
					s = &s[i + 1..];
					key
				};
				let value = {
					let i = parse::find_nl(s);
					let value = from_utf8(&s[..i]);
					let value = if self.auto_trim { trim(value) } else { value };
					self.skip_ln(&s[i..]);
					value
				};
				Some(Item::Property(key, Some(value)))
			},
		}
	}
}

impl<'a> core::iter::FusedIterator for Parser<'a> {}

impl<'a> Parser<'a> {
	#[inline]
	fn skip_ln(&mut self, mut s: &'a [u8]) {
		if !s.is_empty() {
			if s[0] == b'\r' {
				s = &s[1..];
			}
			if !s.is_empty() && s[0] == b'\n' {
   					s = &s[1..];
   				}
			self.line += 1;
		}
		self.state = s;
	}
}

#[cfg(test)]
mod tests;
