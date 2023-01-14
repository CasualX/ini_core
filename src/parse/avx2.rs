
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[inline]
pub fn find_nl(s: &[u8]) -> usize {
	let mut offset = 0;

	unsafe {
		let n_lit = _mm256_set1_epi8(b'\n' as i8);
		let r_lit = _mm256_set1_epi8(b'\r' as i8);

		while offset + 32 <= s.len() {
			let block = _mm256_lddqu_si256(s.as_ptr().add(offset) as *const _);

			let n_eq = _mm256_cmpeq_epi8(n_lit, block);
			let r_eq = _mm256_cmpeq_epi8(r_lit, block);

			let mask = _mm256_movemask_epi8(_mm256_or_si256(n_eq, r_eq));

			if mask != 0 {
				return offset + mask.trailing_zeros() as usize;
			}

			offset += 32;
		}
	}

	unsafe_assert!(offset <= s.len());
	offset += super::generic::find_nl(&s[offset..]);
	unsafe_assert!(offset <= s.len());
	return offset;
}

#[inline]
pub fn find_nl_chr(s: &[u8], chr: u8) -> usize {
	let mut offset = 0;

	unsafe {
		let n_lit = _mm256_set1_epi8(b'\n' as i8);
		let r_lit = _mm256_set1_epi8(b'\r' as i8);
		let c_lit = _mm256_set1_epi8(chr as i8);

		while offset + 32 <= s.len() {
			let block = _mm256_lddqu_si256(s.as_ptr().add(offset) as *const _);

			let n_eq = _mm256_cmpeq_epi8(n_lit, block);
			let r_eq = _mm256_cmpeq_epi8(r_lit, block);
			let c_eq = _mm256_cmpeq_epi8(c_lit, block);

			let mask = _mm256_movemask_epi8(_mm256_or_si256(_mm256_or_si256(n_eq, r_eq), c_eq));

			if mask != 0 {
				return offset + mask.trailing_zeros() as usize;
			}

			offset += 32;
		}
	}

	unsafe_assert!(offset <= s.len());
	offset += super::generic::find_nl_chr(&s[offset..], chr);
	unsafe_assert!(offset <= s.len());
	return offset;
}
