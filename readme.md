Streaming INI parser
====================

Compatible with `no_std`.

Examples
--------

```rust
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

License
-------

Licensed under [MIT License](https://opensource.org/licenses/MIT), see [license.txt](license.txt).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
