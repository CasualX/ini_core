
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[inline]
pub fn find_nl(s: &[u8]) -> usize {
	let mut offset = 0;

	unsafe {
		let n_lit = _mm_set1_epi8(b'\n' as i8);
		let r_lit = _mm_set1_epi8(b'\r' as i8);

		while offset + 16 <= s.len() {
			let block = _mm_loadu_si128(s.as_ptr().add(offset) as *const _);

			let n_eq = _mm_cmpeq_epi8(n_lit, block);
			let r_eq = _mm_cmpeq_epi8(r_lit, block);

			let mask = _mm_movemask_epi8(_mm_or_si128(n_eq, r_eq));

			if mask != 0 {
				return offset + mask.trailing_zeros() as usize;
			}

			offset += 16;
		}
	}

	unsafe_assert!(offset <= s.len());
	offset + super::generic::find_nl(&s[offset..])
}

#[inline]
pub fn find_nl_chr(s: &[u8], chr: u8) -> usize {
	let mut offset = 0;

	unsafe {
		let n_lit = _mm_set1_epi8(b'\n' as i8);
		let r_lit = _mm_set1_epi8(b'\r' as i8);
		let c_lit = _mm_set1_epi8(chr as i8);

		while offset + 16 <= s.len() {
			let block = _mm_loadu_si128(s.as_ptr().add(offset) as *const _);

			let n_eq = _mm_cmpeq_epi8(n_lit, block);
			let r_eq = _mm_cmpeq_epi8(r_lit, block);
			let c_eq = _mm_cmpeq_epi8(c_lit, block);

			let mask = _mm_movemask_epi8(_mm_or_si128(_mm_or_si128(n_eq, r_eq), c_eq));

			if mask != 0 {
				return offset + mask.trailing_zeros() as usize;
			}

			offset += 16;
		}
	}

	unsafe_assert!(offset <= s.len());
	offset + super::generic::find_nl_chr(&s[offset..], chr)
}
