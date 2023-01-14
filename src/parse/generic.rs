
#[inline]
pub fn find_nl(s: &[u8]) -> usize {
	let mut i = 0;
	while i < s.len() {
		if s[i] == b'\n' || s[i] == b'\r' {
			break;
		}
		i += 1;
	}
	unsafe_assert!(i <= s.len());
	return i;
}

#[inline]
pub fn find_nl_chr(s: &[u8], chr: u8) -> usize {
	let mut i = 0;
	while i < s.len() {
		if s[i] == b'\n' || s[i] == b'\r' || s[i] == chr {
			break;
		}
		i += 1;
	}
	unsafe_assert!(i <= s.len());
	return i;
}
