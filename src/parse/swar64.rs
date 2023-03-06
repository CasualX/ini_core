
#[inline]
pub fn find_nl(s: &[u8]) -> usize {
	let mut offset = 0;

	let n_lit = b'\n' as u64 * 0x0101010101010101u64;
	let r_lit = b'\r' as u64 * 0x0101010101010101u64;
	while offset + 8 <= s.len() {
		let word = unsafe { (s.as_ptr().add(offset) as *const u64).read_unaligned() };
		let mask = cmpeq(n_lit, word) | cmpeq(r_lit, word);
		if mask != 0 {
			return offset + (mask.trailing_zeros() >> 3) as usize;
		}

		offset += 8;
	}

	unsafe_assert!(offset <= s.len());
	offset += super::generic::find_nl(&s[offset..]);
	unsafe_assert!(offset <= s.len());
	offset
}

#[inline]
pub fn find_nl_chr(s: &[u8], chr: u8) -> usize {
	let mut offset = 0;

	let n_lit = b'\n' as u64 * 0x0101010101010101u64;
	let r_lit = b'\r' as u64 * 0x0101010101010101u64;
	let c_lit = chr as u64 * 0x0101010101010101u64;
	while offset + 8 <= s.len() {
		let word = unsafe { (s.as_ptr().add(offset) as *const u64).read_unaligned() };
		let mask = cmpeq(n_lit, word) | cmpeq(r_lit, word) | cmpeq(c_lit, word);
		if mask != 0 {
			return offset + (mask.trailing_zeros() >> 3) as usize;
		}

		offset += 8;
	}

	unsafe_assert!(offset <= s.len());
	offset += super::generic::find_nl_chr(&s[offset..], chr);
	unsafe_assert!(offset <= s.len());
	offset
}

#[inline]
fn cmpeq(needle: u64, haystack: u64) -> u64 {
	let neq = !(needle ^ haystack);
	let t0 = (neq & 0x7f7f7f7f7f7f7f7f) + 0x0101010101010101;
	let t1 = neq & 0x8080808080808080;
	t0 & t1
}
