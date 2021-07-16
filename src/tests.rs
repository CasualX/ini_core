use crate::*;

#[track_caller]
fn check(s: &str, expected: &[Item]) {
	let value: Vec<_> = Parser::new(s).collect();
	assert_eq!(value, expected);
}

#[track_caller]
fn check_err(s: &str, line: usize) {
	for (index, item) in Parser::new(s).enumerate() {
		if index == line {
			assert!(matches!(item, Item::Error(_)));
		}
	}
}

#[test]
fn test_eos() {
	check("\r\n[SECTION]", &[Item::Blank, Item::Section("SECTION")]);
	check("\r\n[SECTION]\n", &[Item::Blank, Item::Section("SECTION")]);
	check("\r\n;comment", &[Item::Blank, Item::Comment("comment")]);
	check("\r\n;comment\n", &[Item::Blank, Item::Comment("comment")]);
	check("\r\nKey=Value", &[Item::Blank, Item::Property("Key", "Value")]);
	check("\r\nKey=Value\n", &[Item::Blank, Item::Property("Key", "Value")]);
	check("\r\nKey=Value\r", &[Item::Blank, Item::Property("Key", "Value")]);
	check("\r\nKey=Value\r\n", &[Item::Blank, Item::Property("Key", "Value")]);
	check("\r\nAction", &[Item::Blank, Item::Action("Action")]);
	check("\r\nAction\n", &[Item::Blank, Item::Action("Action")]);
	check("\r\nAction\r", &[Item::Blank, Item::Action("Action")]);
	check("\r\nAction\r\n", &[Item::Blank, Item::Action("Action")]);
}

#[test]
fn test_empty_strings() {
	check("[]\n=\r\n = \n;\n \r= \r\n =\n=", &[
		Item::Section(""),
		Item::Property("", ""),
		Item::Property(" ", " "),
		Item::Comment(""),
		Item::Action(" "),
		Item::Property("", " "),
		Item::Property(" ", ""),
		Item::Property("", ""),
	]);
}

#[test]
fn test_syntax_errors() {
	check_err("[foo] ", 0);
	check_err("[foo] \r", 0);
	check_err("[foo] \n", 0);
	check_err("[foo", 0);
	check_err("[foo\r", 0);
	check_err("[foo\n", 0);
	check_err("[", 0);
	check_err("[\r", 0);
	check_err("[\n", 0);
	check_err("[foo]\n[", 1);
}

#[test]
fn test_blank_lines() {
	check("\n\r\n\r", &[Item::Blank; 3]);
	check("\r\r\n\r", &[Item::Blank; 3]);
	check("\r\r\r\n", &[Item::Blank; 3]);
}

#[test]
fn test_terminates() {
	// Ensure syntax errors advance the internal parser state
	check("[\n[] \r\n", &[
		Item::Error("["),
		Item::Error("[] "),
	]);
	for _ in Parser::new("[") {}
	for _ in Parser::new("[] ") {}
}
