#![feature(test)]

extern crate test;

use std::hint::black_box;
use test::Bencher;

const BIG_INI: &str = include_str!("big.ini");
const SMALL_INI: &str = include_str!("small.ini");

#[bench]
fn big_ini_core(b: &mut Bencher) {
	ini_core(b, black_box(BIG_INI));
}
#[bench]
fn small_ini_core(b: &mut Bencher) {
	ini_core(b, black_box(SMALL_INI));
}
fn ini_core(b: &mut Bencher, s: &str) {
	b.iter(|| {
		let mut count = 0;
		for item in ini_core::Parser::new(s) {
			// Try really hard to avoid LLVM optimizations
			match item {
				ini_core::Item::Section(s) |
				ini_core::Item::Error(s) |
				ini_core::Item::Comment(s) |
				ini_core::Item::Property(s, None) => {
					count += s.len();
				},
				ini_core::Item::SectionEnd => {
					count += 1;
				},
				ini_core::Item::Property(key, Some(value)) => {
					count += key.len() + value.len();
				},
				ini_core::Item::Blank => {
					count += 1;
				},
			}
		}
		count
	});
	b.bytes = s.len() as u64;
}

#[bench]
fn big_configparser(b: &mut Bencher) {
	configparser(b, black_box(BIG_INI));
}
#[bench]
fn small_configparser(b: &mut Bencher) {
	configparser(b, black_box(SMALL_INI));
}
fn configparser(b: &mut Bencher, s: &str) {
	let mut config = configparser::ini::Ini::new();
	b.iter(|| {
		config.read(String::from(s))
	});
	b.bytes = s.len() as u64;
}

#[bench]
fn big_simpleini(b: &mut Bencher) {
	simpleini(b, black_box(BIG_INI));
}
#[bench]
fn small_simpleini(b: &mut Bencher) {
	simpleini(b, black_box(SMALL_INI));
}
fn simpleini(b: &mut Bencher, s: &str) {
	b.iter(|| {
		simpleini::Ini::deserialize(s)
	});
	b.bytes = s.len() as u64;
}

#[bench]
fn big_tini(b: &mut Bencher) {
	tini(b, black_box(BIG_INI));
}
#[bench]
fn small_tini(b: &mut Bencher) {
	tini(b, black_box(SMALL_INI));
}
fn tini(b: &mut Bencher, s: &str) {
	b.iter(|| {
		tini::Ini::from_string(s)
	});
	b.bytes = s.len() as u64;
}

#[bench]
fn big_light_ini(b: &mut Bencher) {
	light_ini(b, black_box(BIG_INI))
}
#[bench]
fn small_light_ini(b: &mut Bencher) {
	light_ini(b, black_box(SMALL_INI))
}
fn light_ini(b: &mut Bencher, s: &str) {

	struct Handler {
		count: i32,
	}
	impl light_ini::IniHandler for Handler {
		type Error = light_ini::IniHandlerError;

		fn section(&mut self, _name: &str) -> Result<(), Self::Error> {
			self.count += 1;
			Ok(())
		}

		fn option(&mut self, _key: &str, _value: &str) -> Result<(), Self::Error> {
			self.count += 1;
			Ok(())
		}

		fn comment(&mut self, _comment: &str) -> Result<(), Self::Error> {
			self.count += 1;
			Ok(())
		}
	}

	b.iter(|| {
		let mut handler = Handler { count: 0 };
		let _ = light_ini::IniParser::new(&mut handler).parse(std::io::Cursor::new(s));
		handler
	});
	b.bytes = s.len() as u64;
}
