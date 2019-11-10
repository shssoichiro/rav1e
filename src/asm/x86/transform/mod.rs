pub mod inverse;

use core::arch::x86_64::*;

const NEW_SQRT2_BITS: i32 = 12;
const NEW_INV_SQRT2: i32 = 2896;

#[inline(always)]
pub(crate) unsafe fn round_shift_32_avx2(vec: __m256i, bit: i32) -> __m256i {
  let round = _mm256_set1_epi32(1 << (bit - 1));
  let tmp = _mm256_add_epi32(vec, round);
  _mm256_srai_epi32(tmp, bit)
}

#[inline(always)]
pub(crate) unsafe fn round_shift_array_32_avx2(
  input: *const __m256i, output: *mut __m256i, size: usize, bit: i32,
) {
  if bit > 0 {
    for i in 0..size {
      output.add(i).write(round_shift_32_avx2(input.add(i).read(), bit));
    }
  } else {
    for i in 0..size {
      output.add(i).write(_mm256_slli_epi32(input.add(i).read(), -bit));
    }
  }
}

#[inline(always)]
pub(crate) unsafe fn round_shift_rect_array_32_avx2(
  input: *const __m256i, output: *mut __m256i, size: usize, bit: i32, val: i32,
) {
  let sqrt2 = _mm256_set1_epi32(val);
  if bit > 0 {
    for i in 0..size {
      let r0 = round_shift_32_avx2(input.add(i).read(), bit);
      let r1 = _mm256_mullo_epi32(sqrt2, r0);
      output.add(i).write(round_shift_32_avx2(r1, NEW_SQRT2_BITS));
    }
  } else {
    for i in 0..size {
      let r0 = _mm256_slli_epi32(input.add(i).read(), -bit);
      let r1 = _mm256_mullo_epi32(sqrt2, r0);
      output.add(i).write(round_shift_32_avx2(r1, NEW_SQRT2_BITS));
    }
  }
}
