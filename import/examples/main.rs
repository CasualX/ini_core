fn main() {
	let ini = ini_import::import!("/hello.ini");
	for &(section, props) in ini {
		if let Some(section) = section {
			println!("[{}]", section);
		}
		for &(key, value) in props {
			println!("{}={}", key, value);
		}
	}
}
