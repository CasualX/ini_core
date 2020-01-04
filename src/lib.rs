/*!
Streaming INI parser
====================

Compatible with `no_std`.

Examples
--------

```
use core_ini::{Parser, Item};

let document = "
[SECTION]
;this is a comment
Key=Value";

let elements = [
	Item::Section("SECTION"),
	Item::Comment("this is a comment"),
	Item::Property("Key", "Value"),
];

for (index, elem) in Parser::new(document).enumerate() {
	assert_eq!(elements[index], elem);
}
```

 */

use core::str;

/// Ini component.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Item<'a> {
	/// Ini section element.
	///
	/// Eg. `[SECTION]` results in `Item::Section("SECTION")`.
	Section(&'a str),
	/// Ini property element.
	///
	/// Eg. `KEY=VALUE` results in `Item::Property("KEY", "VALUE")`.
	Property(&'a str, &'a str),
	/// Ini comment.
	///
	/// Eg. `;comment` results in `Item::Comment("comment")`.
	Comment(&'a str),
}

/// Gets a value in the given section with key.
pub fn get<'a>(config: &'a str, section: Option<&str>, key: &str) -> Option<&'a str> {
	let mut current = section.is_none();
	for item in Parser::new(config) {
		match item {
			Item::Section(name) => {
				current = Some(name) == section;
			},
			Item::Property(k, v) => {
				if current && k == key {
					return Some(v);
				}
			},
			_ => (),
		}
	}
	None
}
pub fn set(config: &mut String, section: Option<&str>, key: &str, value: &str) {
	unimplemented!()
}

/// Streaming Ini parser.
#[derive(Clone, Debug)]
pub struct Parser<'a>(&'a str);

impl<'a> Parser<'a> {
	pub fn new(s: &'a str) -> Parser<'a> {
		Parser(s)
	}
}

impl<'a> Iterator for Parser<'a> {
	type Item = Item<'a>;
	fn next(&mut self) -> Option<Item<'a>> {
		// Strip off empty lines
		loop {
			if self.0.len() == 0 {
				return None;
			}
			if self.0.as_bytes()[0] != b'\n' {
				break;
			}
			self.0 = &self.0[1..];
		}
		let s = self.0.as_bytes();
		// Line is a comment
		if s[0] == b';' {
			let mut i = 0;
			let comment_end;
			loop {
				i += 1;
				if i >= s.len() {
					comment_end = i;
					break;
				}
				if s[i] == b'\n' {
					comment_end = i;
					i += 1;
					break;
				}
			}
			let comment = &self.0[1..comment_end];
			self.0 = &self.0[i..];
			Some(Item::Comment(comment))
		}
		// Line is a section
		else if s[0] == b'[' {
			let mut i = 0;
			let mut end = 0;
			loop {
				i += 1;
				if i >= s.len() {
					if end == 0 {
						end = i;
					}
					break;
				}
				if s[i] == b']' {
					end = i;
				}
				if s[i] == b'\n' {
					if end == 0 {
						end = i;
					}
					i += 1;
					break;
				}
			}
			let name = &self.0[1..end];
			self.0 = &self.0[i..];
			Some(Item::Section(name))
		}
		// Line is a property
		else {
			let mut i = 0;
			let mut key_end = 0;
			let mut value_start = 0;
			let mut value_end = 0;
			loop {
				i += 1;
				if i >= s.len() {
					if value_start > value_end {
						value_end = i;
					}
					break;
				}
				if s[i] == b'=' {
					if value_start == 0 {
						key_end = i;
						value_start = i + 1;
					}
				}
				else if s[i] == b'\n' {
					value_end = i;
					i += 1;
					break;
				}
			}
			let key = &self.0[0..key_end];
			let value = &self.0[value_start..value_end];
			self.0 = &self.0[i..];
			Some(Item::Property(key, value))
		}
	}
}

#[test]
fn test_eos() {
	assert_eq!(Parser::new("[SECTION]").collect::<Vec<_>>(), vec![Item::Section("SECTION")]);
	assert_eq!(Parser::new("[SECTION]\n").collect::<Vec<_>>(), vec![Item::Section("SECTION")]);
	assert_eq!(Parser::new(";comment").collect::<Vec<_>>(), vec![Item::Comment("comment")]);
	assert_eq!(Parser::new(";comment\n").collect::<Vec<_>>(), vec![Item::Comment("comment")]);
	assert_eq!(Parser::new("Key=Value").collect::<Vec<_>>(), vec![Item::Property("Key", "Value")]);
	assert_eq!(Parser::new("Key=Value\n").collect::<Vec<_>>(), vec![Item::Property("Key", "Value")]);
}
