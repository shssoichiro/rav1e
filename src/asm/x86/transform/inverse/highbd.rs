/*
 * Copyright (c) 2016, Alliance for Open Media. All rights reserved
 *
 * This source code is subject to the terms of the BSD 2 Clause License and
 * the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
 * was not distributed with this source code in the LICENSE file, you can
 * obtain it at www.aomedia.org/license/software. If the Alliance for Open
 * Media Patent License 1.0 was not distributed with this source code in the
 * PATENTS file, you can obtain it at www.aomedia.org/license/patent.
 */

use super::super::*;
use crate::tiling::PlaneRegionMut;
use crate::transform::*;
use crate::util::AlignedArray;
use crate::util::Pixel;
use core::arch::x86_64::*;
use std::cmp;

#[target_feature(enable = "avx2")]
pub unsafe fn highbd_inv_txfm_add_avx2<T: Pixel>(
  input: &[i32], output: &mut PlaneRegionMut<'_, T>, tx_type: TxType,
  tx_size: TxSize, bd: usize,
) {
  // 64x only uses 32 coeffs
  let coeff_w = tx_size.width().min(32);
  let coeff_h = tx_size.height().min(32);
  let mut coeff32: AlignedArray<[i32; 32 * 32]> =
    AlignedArray::uninitialized();

  // Transpose the input.
  // TODO: should be possible to remove changing how coeffs are written
  assert!(input.len() >= coeff_w * coeff_h);
  for j in 0..coeff_h {
    for i in 0..coeff_w {
      coeff32.array[i * coeff_h + j] = input[j * coeff_w + i] as i32;
    }
  }

  let eob = (coeff_w * coeff_h) as i32;
  let stride = output.plane_cfg.stride as isize;

  match tx_size {
    TxSize::TX_4X8 => {
      highbd_inv_txfm_add_4x8_sse4_1(input, output, stride, tx_type, eob, bd);
    }
    TxSize::TX_8X4 => {
      highbd_inv_txfm_add_8x4_sse4_1(input, output, stride, tx_type, eob, bd);
    }
    TxSize::TX_4X4 => {
      highbd_inv_txfm_add_4x4_sse4_1(input, output, stride, tx_type, eob, bd);
    }
    TxSize::TX_16X4 => {
      highbd_inv_txfm_add_16x4_sse4_1(input, output, stride, tx_type, eob, bd);
    }
    TxSize::TX_4X16 => {
      highbd_inv_txfm_add_4x16_sse4_1(input, output, stride, tx_type, eob, bd);
    }
    _ => {
      highbd_inv_txfm2d_add_universe_avx2(
        input, output, stride, tx_type, tx_size, eob, bd,
      );
    }
  }

  // TODO: Dest into output
}

#[target_feature(enable = "sse4.1")]
pub unsafe fn highbd_inv_txfm_add_sse4_1<T: Pixel>(
  input: &[i32], output: &mut PlaneRegionMut<'_, T>, tx_type: TxType,
  tx_size: TxSize, bd: usize,
) {
  // 64x only uses 32 coeffs
  let coeff_w = tx_size.width().min(32);
  let coeff_h = tx_size.height().min(32);
  let mut coeff32: AlignedArray<[i32; 32 * 32]> =
    AlignedArray::uninitialized();

  // Transpose the input.
  // TODO: should be possible to remove changing how coeffs are written
  assert!(input.len() >= coeff_w * coeff_h);
  for j in 0..coeff_h {
    for i in 0..coeff_w {
      coeff32.array[i * coeff_h + j] = input[j * coeff_w + i] as i32;
    }
  }

  let eob = (coeff_w * coeff_h) as i32;
  let stride = output.plane_cfg.stride as isize;

  match tx_size {
    TxSize::TX_8X8 => {
      highbd_inv_txfm_add_8x8_sse4_1(input, output, stride, tx_type, bd);
    }
    TxSize::TX_4X8 => {
      highbd_inv_txfm_add_4x8_sse4_1(input, output, stride, tx_type, bd);
    }
    TxSize::TX_8X4 => {
      highbd_inv_txfm_add_8x4_sse4_1(input, output, stride, tx_type, bd);
    }
    TxSize::TX_4X4 => {
      highbd_inv_txfm_add_4x4_sse4_1(input, output, stride, tx_type, bd);
    }
    TxSize::TX_16X4 => {
      highbd_inv_txfm_add_16x4_sse4_1(input, output, stride, tx_type, bd);
    }
    TxSize::TX_4X16 => {
      highbd_inv_txfm_add_4x16_sse4_1(input, output, stride, tx_type, bd);
    }
    _ => {
      highbd_inv_txfm2d_add_universe_sse4_1(
        input, output, stride, tx_type, tx_size, bd,
      );
    }
  }

  // TODO: Dest into output
}

#[target_feature(enable = "avx2")]
unsafe fn highbd_inv_txfm2d_add_universe_avx2<T: Pixel>(
  input: &[i32], output: &mut PlaneRegionMut<'_, T>, stride: isize,
  tx_type: TxType, tx_size: TxSize, eob: i32, bd: usize,
) {
  match tx_type {
    TxType::DCT_DCT
    | TxType::ADST_DCT
    | TxType::DCT_ADST
    | TxType::ADST_ADST
    | TxType::FLIPADST_DCT
    | TxType::DCT_FLIPADST
    | TxType::FLIPADST_FLIPADST
    | TxType::ADST_FLIPADST
    | TxType::FLIPADST_ADST => {
      highbd_inv_txfm2d_add_no_identity_avx2(
        input.as_ptr(),
        output.data_ptr_mut() as *mut _,
        stride,
        tx_type,
        tx_size,
        eob,
        bd,
      );
    }
    TxType::IDTX
    | TxType::H_DCT
    | TxType::H_ADST
    | TxType::H_FLIPADST
    | TxType::V_DCT
    | TxType::V_ADST
    | TxType::V_FLIPADST => {
      highbd_inv_txfm2d_add_universe_sse4_1(
        input, output, stride, tx_type, tx_size, eob, bd,
      );
    }
  }
}

const INV_SHIFT_4X4: [i8; 2] = [0, -4];
const INV_SHIFT_8X8: [i8; 2] = [-1, -4];
const INV_SHIFT_16X16: [i8; 2] = [-2, -4];
const INV_SHIFT_32X32: [i8; 2] = [-2, -4];
const INV_SHIFT_64X64: [i8; 2] = [-2, -4];
const INV_SHIFT_4X8: [i8; 2] = [0, -4];
const INV_SHIFT_8X4: [i8; 2] = [0, -4];
const INV_SHIFT_8X16: [i8; 2] = [-1, -4];
const INV_SHIFT_16X8: [i8; 2] = [-1, -4];
const INV_SHIFT_16X32: [i8; 2] = [-1, -4];
const INV_SHIFT_32X16: [i8; 2] = [-1, -4];
const INV_SHIFT_32X64: [i8; 2] = [-1, -4];
const INV_SHIFT_64X32: [i8; 2] = [-1, -4];
const INV_SHIFT_4X16: [i8; 2] = [-1, -4];
const INV_SHIFT_16X4: [i8; 2] = [-1, -4];
const INV_SHIFT_8X32: [i8; 2] = [-2, -4];
const INV_SHIFT_32X8: [i8; 2] = [-2, -4];
const INV_SHIFT_16X64: [i8; 2] = [-2, -4];
const INV_SHIFT_64X16: [i8; 2] = [-2, -4];

const INV_TXFM_SHIFT_LS: [[i8; 2]; TxSize::TX_SIZES_ALL] = [
  INV_SHIFT_4X4,
  INV_SHIFT_8X8,
  INV_SHIFT_16X16,
  INV_SHIFT_32X32,
  INV_SHIFT_64X64,
  INV_SHIFT_4X8,
  INV_SHIFT_8X4,
  INV_SHIFT_8X16,
  INV_SHIFT_16X8,
  INV_SHIFT_16X32,
  INV_SHIFT_32X16,
  INV_SHIFT_32X64,
  INV_SHIFT_64X32,
  INV_SHIFT_4X16,
  INV_SHIFT_16X4,
  INV_SHIFT_8X32,
  INV_SHIFT_32X8,
  INV_SHIFT_16X64,
  INV_SHIFT_64X16,
];

const EOB_TO_EOBXY_8X8_DEFAULT: [i16; 8] =
  [0x0707, 0x0707, 0x0707, 0x0707, 0x0707, 0x0707, 0x0707, 0x0707];

const EOB_TO_EOBXY_16X16_DEFAULT: [i16; 16] = [
  0x0707, 0x0707, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f,
  0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f,
];

const EOB_TO_EOBXY_32X32_DEFAULT: [i16; 32] = [
  0x0707, 0x0f0f, 0x0f0f, 0x0f0f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f,
  0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f,
  0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f,
  0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f, 0x1f1f,
];

const EOB_TO_EOBXY_8X16_DEFAULT: [i16; 16] = [
  0x0707, 0x0707, 0x0707, 0x0707, 0x0707, 0x0f07, 0x0f07, 0x0f07, 0x0f07,
  0x0f07, 0x0f07, 0x0f07, 0x0f07, 0x0f07, 0x0f07, 0x0f07,
];

const EOB_TO_EOBXY_16X8_DEFAULT: [i16; 8] =
  [0x0707, 0x0707, 0x070f, 0x070f, 0x070f, 0x070f, 0x070f, 0x070f];

const EOB_TO_EOBXY_16X32_DEFAULT: [i16; 32] = [
  0x0707, 0x0707, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f0f,
  0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f,
  0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f,
  0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f, 0x1f0f,
];

const EOB_TO_EOBXY_32X16_DEFAULT: [i16; 16] = [
  0x0707, 0x0f0f, 0x0f0f, 0x0f0f, 0x0f1f, 0x0f1f, 0x0f1f, 0x0f1f, 0x0f1f,
  0x0f1f, 0x0f1f, 0x0f1f, 0x0f1f, 0x0f1f, 0x0f1f, 0x0f1f,
];

const EOB_TO_EOBXY_8X32_DEFAULT: [i16; 32] = [
  0x0707, 0x0707, 0x0707, 0x0707, 0x0707, 0x0f07, 0x0f07, 0x0f07, 0x0f07,
  0x0f07, 0x0f07, 0x0f07, 0x0f07, 0x1f07, 0x1f07, 0x1f07, 0x1f07, 0x1f07,
  0x1f07, 0x1f07, 0x1f07, 0x1f07, 0x1f07, 0x1f07, 0x1f07, 0x1f07, 0x1f07,
  0x1f07, 0x1f07, 0x1f07, 0x1f07, 0x1f07,
];

const EOB_TO_EOBXY_32X8_DEFAULT: [i16; 8] =
  [0x0707, 0x070f, 0x070f, 0x071f, 0x071f, 0x071f, 0x071f, 0x071f];

const EOB_TO_EOBXY_DEFAULT: [&[i16]; TxSize::TX_SIZES_ALL] = [
  &[],
  &EOB_TO_EOBXY_8X8_DEFAULT,
  &EOB_TO_EOBXY_16X16_DEFAULT,
  &EOB_TO_EOBXY_32X32_DEFAULT,
  &EOB_TO_EOBXY_32X32_DEFAULT,
  &[],
  &[],
  &EOB_TO_EOBXY_8X16_DEFAULT,
  &EOB_TO_EOBXY_16X8_DEFAULT,
  &EOB_TO_EOBXY_16X32_DEFAULT,
  &EOB_TO_EOBXY_32X16_DEFAULT,
  &EOB_TO_EOBXY_32X32_DEFAULT,
  &EOB_TO_EOBXY_32X32_DEFAULT,
  &[],
  &[],
  &EOB_TO_EOBXY_8X32_DEFAULT,
  &EOB_TO_EOBXY_32X8_DEFAULT,
  &EOB_TO_EOBXY_16X32_DEFAULT,
  &EOB_TO_EOBXY_32X16_DEFAULT,
];

fn get_eobx_eoby_scan_default(tx_size: TxSize, eob: i32) -> (i16, i16) {
  if eob == 1 {
    return (0, 0);
  }

  // (size of 64 map to 32)
  let tx_w_log2 = cmp::min(5, tx_size.width_log2());
  let eob_row = (eob - 1) >> tx_w_log2;
  let eobxy = EOB_TO_EOBXY_DEFAULT[tx_size as usize][eob_row as usize];
  (eobxy & 0xFF, eobxy >> 8)
}

const LOWBD_TXFM_ALL_1D_ZEROS_IDX: [usize; 32] = [
  0, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3,
  3, 3, 3, 3, 3, 3, 3,
];

const COSPI_ARR_DATA: [[i32; 64]; 7] = [
  [
    1024, 1024, 1023, 1021, 1019, 1016, 1013, 1009, 1004, 999, 993, 987, 980,
    972, 964, 955, 946, 936, 926, 915, 903, 891, 878, 865, 851, 837, 822, 807,
    792, 775, 759, 742, 724, 706, 688, 669, 650, 630, 610, 590, 569, 548, 526,
    505, 483, 460, 438, 415, 392, 369, 345, 321, 297, 273, 249, 224, 200, 175,
    150, 125, 100, 75, 50, 25,
  ],
  [
    2048, 2047, 2046, 2042, 2038, 2033, 2026, 2018, 2009, 1998, 1987, 1974,
    1960, 1945, 1928, 1911, 1892, 1872, 1851, 1829, 1806, 1782, 1757, 1730,
    1703, 1674, 1645, 1615, 1583, 1551, 1517, 1483, 1448, 1412, 1375, 1338,
    1299, 1260, 1220, 1179, 1138, 1096, 1053, 1009, 965, 921, 876, 830, 784,
    737, 690, 642, 595, 546, 498, 449, 400, 350, 301, 251, 201, 151, 100, 50,
  ],
  [
    4096, 4095, 4091, 4085, 4076, 4065, 4052, 4036, 4017, 3996, 3973, 3948,
    3920, 3889, 3857, 3822, 3784, 3745, 3703, 3659, 3612, 3564, 3513, 3461,
    3406, 3349, 3290, 3229, 3166, 3102, 3035, 2967, 2896, 2824, 2751, 2675,
    2598, 2520, 2440, 2359, 2276, 2191, 2106, 2019, 1931, 1842, 1751, 1660,
    1567, 1474, 1380, 1285, 1189, 1092, 995, 897, 799, 700, 601, 501, 401,
    301, 201, 101,
  ],
  [
    8192, 8190, 8182, 8170, 8153, 8130, 8103, 8071, 8035, 7993, 7946, 7895,
    7839, 7779, 7713, 7643, 7568, 7489, 7405, 7317, 7225, 7128, 7027, 6921,
    6811, 6698, 6580, 6458, 6333, 6203, 6070, 5933, 5793, 5649, 5501, 5351,
    5197, 5040, 4880, 4717, 4551, 4383, 4212, 4038, 3862, 3683, 3503, 3320,
    3135, 2948, 2760, 2570, 2378, 2185, 1990, 1795, 1598, 1401, 1202, 1003,
    803, 603, 402, 201,
  ],
  [
    16384, 16379, 16364, 16340, 16305, 16261, 16207, 16143, 16069, 15986,
    15893, 15791, 15679, 15557, 15426, 15286, 15137, 14978, 14811, 14635,
    14449, 14256, 14053, 13842, 13623, 13395, 13160, 12916, 12665, 12406,
    12140, 11866, 11585, 11297, 11003, 10702, 10394, 10080, 9760, 9434, 9102,
    8765, 8423, 8076, 7723, 7366, 7005, 6639, 6270, 5897, 5520, 5139, 4756,
    4370, 3981, 3590, 3196, 2801, 2404, 2006, 1606, 1205, 804, 402,
  ],
  [
    32768, 32758, 32729, 32679, 32610, 32522, 32413, 32286, 32138, 31972,
    31786, 31581, 31357, 31114, 30853, 30572, 30274, 29957, 29622, 29269,
    28899, 28511, 28106, 27684, 27246, 26791, 26320, 25833, 25330, 24812,
    24279, 23732, 23170, 22595, 22006, 21403, 20788, 20160, 19520, 18868,
    18205, 17531, 16846, 16151, 15447, 14733, 14010, 13279, 12540, 11793,
    11039, 10279, 9512, 8740, 7962, 7180, 6393, 5602, 4808, 4011, 3212, 2411,
    1608, 804,
  ],
  [
    65536, 65516, 65457, 65358, 65220, 65043, 64827, 64571, 64277, 63944,
    63572, 63162, 62714, 62228, 61705, 61145, 60547, 59914, 59244, 58538,
    57798, 57022, 56212, 55368, 54491, 53581, 52639, 51665, 50660, 49624,
    48559, 47464, 46341, 45190, 44011, 42806, 41576, 40320, 39040, 37736,
    36410, 35062, 33692, 32303, 30893, 29466, 28020, 26558, 25080, 23586,
    22078, 20557, 19024, 17479, 15924, 14359, 12785, 11204, 9616, 8022, 6424,
    4821, 3216, 1608,
  ],
];

const COS_BIT_MIN: usize = 10;
const INV_COS_BIT: i8 = 12;

const INV_COS_BIT_COL: [[i8; 5]; 5] = [
  [INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, 0, 0],
  [INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, 0],
  [INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT],
  [0, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT],
  [0, 0, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT],
];

const INV_COS_BIT_ROW: [[i8; 5]; 5] = [
  [INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, 0, 0],
  [INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, 0],
  [INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT],
  [0, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT],
  [0, 0, INV_COS_BIT, INV_COS_BIT, INV_COS_BIT],
];

#[inline(always)]
fn cospi_arr(n: usize) -> &'static [i32; 64] {
  &COSPI_ARR_DATA[n - COS_BIT_MIN]
}

unsafe fn load_buffer_32x32(
  coeff: *const i32, input: *mut __m256i, input_stride: usize, size: usize,
) {
  for i in 0..size {
    input.add(i).write(_mm256_loadu_si256(
      coeff.add(i * input_stride) as *const __m256i
    ));
  }
}

#[target_feature(enable = "avx2")]
unsafe fn transpose_8x8_avx2(input: *const __m256i, output: *mut __m256i) {
  let mut x0;
  let mut x1;

  let u0 = _mm256_unpacklo_epi32(input.add(0).read(), input.add(1).read());
  let u1 = _mm256_unpackhi_epi32(input.add(0).read(), input.add(1).read());

  let u2 = _mm256_unpacklo_epi32(input.add(2).read(), input.add(3).read());
  let u3 = _mm256_unpackhi_epi32(input.add(2).read(), input.add(3).read());

  let u4 = _mm256_unpacklo_epi32(input.add(4).read(), input.add(5).read());
  let u5 = _mm256_unpackhi_epi32(input.add(4).read(), input.add(5).read());

  let u6 = _mm256_unpacklo_epi32(input.add(6).read(), input.add(7).read());
  let u7 = _mm256_unpackhi_epi32(input.add(6).read(), input.add(7).read());

  x0 = _mm256_unpacklo_epi64(u0, u2);
  x1 = _mm256_unpacklo_epi64(u4, u6);
  output.add(0).write(_mm256_permute2f128_si256(x0, x1, 0x20));
  output.add(4).write(_mm256_permute2f128_si256(x0, x1, 0x31));

  x0 = _mm256_unpackhi_epi64(u0, u2);
  x1 = _mm256_unpackhi_epi64(u4, u6);
  output.add(1).write(_mm256_permute2f128_si256(x0, x1, 0x20));
  output.add(5).write(_mm256_permute2f128_si256(x0, x1, 0x31));

  x0 = _mm256_unpacklo_epi64(u1, u3);
  x1 = _mm256_unpacklo_epi64(u5, u7);
  output.add(2).write(_mm256_permute2f128_si256(x0, x1, 0x20));
  output.add(6).write(_mm256_permute2f128_si256(x0, x1, 0x31));

  x0 = _mm256_unpackhi_epi64(u1, u3);
  x1 = _mm256_unpackhi_epi64(u5, u7);
  output.add(3).write(_mm256_permute2f128_si256(x0, x1, 0x20));
  output.add(7).write(_mm256_permute2f128_si256(x0, x1, 0x31));
}

#[target_feature(enable = "avx2")]
unsafe fn transpose_8x8_flip_avx2(
  input: *const __m256i, output: *mut __m256i,
) {
  let mut x0;
  let mut x1;

  let u0 = _mm256_unpacklo_epi32(input.add(7).read(), input.add(6).read());
  let u1 = _mm256_unpackhi_epi32(input.add(7).read(), input.add(6).read());

  let u2 = _mm256_unpacklo_epi32(input.add(5).read(), input.add(4).read());
  let u3 = _mm256_unpackhi_epi32(input.add(5).read(), input.add(4).read());

  let u4 = _mm256_unpacklo_epi32(input.add(3).read(), input.add(2).read());
  let u5 = _mm256_unpackhi_epi32(input.add(3).read(), input.add(2).read());

  let u6 = _mm256_unpacklo_epi32(input.add(1).read(), input.add(0).read());
  let u7 = _mm256_unpackhi_epi32(input.add(1).read(), input.add(0).read());

  x0 = _mm256_unpacklo_epi64(u0, u2);
  x1 = _mm256_unpacklo_epi64(u4, u6);
  output.add(0).write(_mm256_permute2f128_si256(x0, x1, 0x20));
  output.add(4).write(_mm256_permute2f128_si256(x0, x1, 0x31));

  x0 = _mm256_unpackhi_epi64(u0, u2);
  x1 = _mm256_unpackhi_epi64(u4, u6);
  output.add(1).write(_mm256_permute2f128_si256(x0, x1, 0x20));
  output.add(5).write(_mm256_permute2f128_si256(x0, x1, 0x31));

  x0 = _mm256_unpacklo_epi64(u1, u3);
  x1 = _mm256_unpacklo_epi64(u5, u7);
  output.add(2).write(_mm256_permute2f128_si256(x0, x1, 0x20));
  output.add(6).write(_mm256_permute2f128_si256(x0, x1, 0x31));

  x0 = _mm256_unpackhi_epi64(u1, u3);
  x1 = _mm256_unpackhi_epi64(u5, u7);
  output.add(3).write(_mm256_permute2f128_si256(x0, x1, 0x20));
  output.add(7).write(_mm256_permute2f128_si256(x0, x1, 0x31));
}

#[inline(always)]
unsafe fn round_shift_4x4_avx2(input: *mut __m256i, shift: i32) {
  if shift != 0 {
    let rnding = _mm256_set1_epi32(1 << (shift - 1));
    input.add(0).write(_mm256_add_epi32(input.add(0).read(), rnding));
    input.add(1).write(_mm256_add_epi32(input.add(1).read(), rnding));
    input.add(2).write(_mm256_add_epi32(input.add(2).read(), rnding));
    input.add(3).write(_mm256_add_epi32(input.add(3).read(), rnding));

    input.add(0).write(_mm256_srai_epi32(input.add(0).read(), shift));
    input.add(1).write(_mm256_srai_epi32(input.add(1).read(), shift));
    input.add(2).write(_mm256_srai_epi32(input.add(2).read(), shift));
    input.add(3).write(_mm256_srai_epi32(input.add(3).read(), shift));
  }
}

#[inline(always)]
unsafe fn round_shift_8x8_avx2(input: *mut __m256i, shift: i32) {
  round_shift_4x4_avx2(input, shift);
  round_shift_4x4_avx2(input.add(4), shift);
  round_shift_4x4_avx2(input.add(8), shift);
  round_shift_4x4_avx2(input.add(12), shift);
}

#[target_feature(enable = "avx2")]
unsafe fn addsub_avx2(
  in0: __m256i, in1: __m256i, out0: *mut __m256i, out1: *mut __m256i,
  clamp_lo: __m256i, clamp_hi: __m256i,
) {
  let mut a0 = _mm256_add_epi32(in0, in1);
  let mut a1 = _mm256_sub_epi32(in0, in1);

  a0 = _mm256_max_epi32(a0, clamp_lo);
  a0 = _mm256_min_epi32(a0, clamp_hi);
  a1 = _mm256_max_epi32(a1, clamp_lo);
  a1 = _mm256_min_epi32(a1, clamp_hi);

  *out0 = a0;
  *out1 = a1;
}

#[target_feature(enable = "avx2")]
unsafe fn neg_shift_avx2(
  in0: __m256i, in1: __m256i, out0: *mut __m256i, out1: *mut __m256i,
  clamp_lo: __m256i, clamp_hi: __m256i, shift: i32,
) {
  let offset = _mm256_set1_epi32((1 << shift) >> 1);
  let mut a0 = _mm256_add_epi32(offset, in0);
  let mut a1 = _mm256_sub_epi32(offset, in1);

  a0 = _mm256_sra_epi32(a0, _mm_cvtsi32_si128(shift));
  a1 = _mm256_sra_epi32(a1, _mm_cvtsi32_si128(shift));

  a0 = _mm256_max_epi32(a0, clamp_lo);
  a0 = _mm256_min_epi32(a0, clamp_hi);
  a1 = _mm256_max_epi32(a1, clamp_lo);
  a1 = _mm256_min_epi32(a1, clamp_hi);

  *out0 = a0;
  *out1 = a1;
}

#[inline(always)]
unsafe fn half_btf_0_avx2(
  w0: __m256i, n0: __m256i, rounding: __m256i, bit: i32,
) -> __m256i {
  let mut x = _mm256_mullo_epi32(w0, n0);
  x = _mm256_add_epi32(x, rounding);
  x = _mm256_srai_epi32(x, bit);
  x
}

#[inline(always)]
unsafe fn half_btf_avx2(
  w0: __m256i, n0: __m256i, w1: __m256i, n1: __m256i, rounding: __m256i,
  bit: i32,
) -> __m256i {
  let mut x = _mm256_mullo_epi32(w0, n0);
  let y = _mm256_mullo_epi32(w1, n1);
  x = _mm256_add_epi32(x, y);
  x = _mm256_add_epi32(x, rounding);
  x = _mm256_srai_epi32(x, bit);
  x
}

#[inline(always)]
unsafe fn highbd_clamp_epi16_avx2(u: __m256i, bd: usize) -> __m256i {
  let zero = _mm256_setzero_si256();
  let one = _mm256_set1_epi16(1);
  let max = _mm256_sub_epi16(_mm256_slli_epi16(one, bd as i32), one);

  let mut mask = _mm256_cmpgt_epi16(u, max);
  let mut clamped = _mm256_andnot_si256(mask, u);
  mask = _mm256_and_si256(mask, max);
  clamped = _mm256_or_si256(mask, clamped);
  mask = _mm256_cmpgt_epi16(clamped, zero);
  clamped = _mm256_and_si256(clamped, mask);

  clamped
}

#[inline(always)]
unsafe fn highbd_clamp_epi32_avx2(
  input: *const __m256i, output: *mut __m256i, clamp_lo: __m256i,
  clamp_hi: __m256i, size: usize,
) {
  let mut a0;
  let mut a1;

  for i in (0..size).step_by(4) {
    a0 = _mm256_max_epi32(input.add(i).read(), clamp_lo);
    output.add(i).write(_mm256_min_epi32(a0, clamp_hi));

    a1 = _mm256_max_epi32(input.add(i + 1).read(), clamp_lo);
    output.add(i + 1).write(_mm256_min_epi32(a1, clamp_hi));

    a0 = _mm256_max_epi32(input.add(i + 2).read(), clamp_lo);
    output.add(i + 2).write(_mm256_min_epi32(a0, clamp_hi));

    a1 = _mm256_max_epi32(input.add(i + 3).read(), clamp_lo);
    output.add(i + 3).write(_mm256_min_epi32(a1, clamp_hi));
  }
}

#[inline(always)]
unsafe fn highbd_get_recon_16x8_avx2(
  pred: __m256i, res0: __m256i, res1: __m256i, bd: usize,
) -> __m256i {
  let mut x0 = _mm256_cvtepi16_epi32(_mm256_castsi256_si128(pred));
  let mut x1 = _mm256_cvtepi16_epi32(_mm256_extractf128_si256(pred, 1));

  x0 = _mm256_add_epi32(res0, x0);
  x1 = _mm256_add_epi32(res1, x1);
  x0 = _mm256_packus_epi32(x0, x1);
  x0 = _mm256_permute4x64_epi64(x0, 0xd8);
  x0 = highbd_clamp_epi16_avx2(x0, bd);
  x0
}

#[inline(always)]
unsafe fn highbd_write_buffer_16xn_avx2(
  input: *const __m256i, output: *mut u16, stride: usize, flipud: bool,
  height: usize, bd: usize,
) {
  let mut j = if flipud { height as isize - 1 } else { 0 };
  let step = if flipud { -1 } else { 1 };
  for i in 0..height {
    let v = _mm256_loadu_si256(output.add(i * stride) as *const __m256i);
    let u = highbd_get_recon_16x8_avx2(
      v,
      input.offset(j).read(),
      input.offset(j + height as isize).read(),
      bd,
    );

    _mm256_storeu_si256(output.add(i * stride) as *mut __m256i, u);
    j += step;
  }
}

#[inline(always)]
unsafe fn highbd_get_recon_8x8_avx2(
  pred: __m256i, res: __m256i, bd: usize,
) -> __m256i {
  let mut x0 = pred;
  x0 = _mm256_add_epi32(res, x0);
  x0 = _mm256_packus_epi32(x0, x0);
  x0 = _mm256_permute4x64_epi64(x0, 0xd8);
  x0 = highbd_clamp_epi16_avx2(x0, bd);
  x0
}

#[inline(always)]
unsafe fn highbd_write_buffer_8xn_avx2(
  input: *const __m256i, output: *mut u16, stride: usize, flipud: bool,
  height: usize, bd: usize,
) {
  let mut j = if flipud { height as isize - 1 } else { 0 };
  let step = if flipud { -1 } else { 1 };

  let mut temp;
  for i in 0..height {
    temp = _mm_loadu_si128(output.add(i * stride) as *const __m128i);
    let v = _mm256_cvtepi16_epi32(temp);
    let u = highbd_get_recon_8x8_avx2(v, input.offset(j).read(), bd);
    let u1 = _mm256_castsi256_si128(u);
    _mm_storeu_si128(output.add(i * stride) as *mut __m128i, u1);
    j += step;
  }
}

type HighBdInvTxfmFunc =
  unsafe fn(*mut __m256i, *mut __m256i, i32, bool, usize, i32);

fn get_highbd_avx2_func(
  tx_size: usize, tx_type: TxType1D, lowbd_idx: usize,
) -> HighBdInvTxfmFunc {
  match tx_type {
    TxType1D::DCT => match tx_size {
      8 => match lowbd_idx {
        0 => idct8x8_low1_avx2,
        1 => idct8x8_avx2,
        _ => unreachable!(),
      },
      16 => match lowbd_idx {
        0 => idct16_low1_avx2,
        1 => idct16_low8_avx2,
        2 => idct16_avx2,
        _ => unreachable!(),
      },
      32 => match lowbd_idx {
        0 => idct32_low1_avx2,
        1 => idct32_low8_avx2,
        2 => idct32_low16_avx2,
        3 => idct32_avx2,
        _ => unreachable!(),
      },
      64 => match lowbd_idx {
        0 => idct64_low1_avx2,
        1 => idct64_low8_avx2,
        2 => idct64_low16_avx2,
        3 => idct64_avx2,
        _ => unreachable!(),
      },
      _ => unreachable!(),
    },
    TxType1D::ADST => match tx_size {
      8 => match lowbd_idx {
        0 => iadst8x8_low1_avx2,
        1 => iadst8x8_avx2,
        _ => unreachable!(),
      },
      16 => match lowbd_idx {
        0 => iadst16_low1_avx2,
        1 => iadst16_low8_avx2,
        2 => iadst16_avx2,
        _ => unreachable!(),
      },
      _ => unreachable!(),
    },
    _ => unreachable!(),
  }
}

#[target_feature(enable = "avx2")]
unsafe fn highbd_inv_txfm2d_add_no_identity_avx2(
  input: *const i32, output: *mut u16, stride: isize, tx_type: TxType,
  tx_size: TxSize, eob: i32, bd: usize,
) {
  let mut buf1 = [_mm256_setzero_si256(); 64 * 8];
  let (eobx, eoby) = get_eobx_eoby_scan_default(tx_size, eob);
  let shift = INV_TXFM_SHIFT_LS[tx_size as usize];
  let txw_idx = tx_size.width_index();
  let txh_idx = tx_size.height_index();
  let txfm_size_col = tx_size.width();
  let txfm_size_row = tx_size.height();
  let buf_size_w_div8 = txfm_size_col >> 3;
  let buf_size_nonzero_w_div8 = (eobx as usize + 8) >> 3;
  let buf_size_nonzero_h_div8 = (eoby as usize + 8) >> 3;
  let input_stride = cmp::min(32, txfm_size_col);
  let rect_type = get_rect_tx_log_ratio(txfm_size_col, txfm_size_row);
  let fun_idx_x = LOWBD_TXFM_ALL_1D_ZEROS_IDX[eobx as usize];
  let fun_idx_y = LOWBD_TXFM_ALL_1D_ZEROS_IDX[eoby as usize];
  let (tx_type_x, tx_type_y) = get_1d_tx_types(tx_type);
  let row_txfm = get_highbd_avx2_func(txfm_size_col, tx_type_x, fun_idx_x);
  let col_txfm = get_highbd_avx2_func(txfm_size_row, tx_type_y, fun_idx_y);

  let (ud_flip, lr_flip) = Txfm2DFlipCfg::get_flip_cfg(tx_type);

  // 1st stage: column transform
  for i in 0..buf_size_nonzero_h_div8 {
    let mut buf0 = [_mm256_setzero_si256(); 64];
    let input_row = input.add(i * input_stride * 8);
    for j in 0..buf_size_nonzero_w_div8 {
      let buf0_cur = &mut buf0[(j * 8)..];
      load_buffer_32x32(
        input_row.add(j * 8),
        buf0_cur.as_mut_ptr(),
        input_stride,
        8,
      );
      transpose_8x8_avx2(buf0_cur.as_mut_ptr(), buf0_cur.as_mut_ptr());
    }
    if rect_type == 1 || rect_type == -1 {
      round_shift_rect_array_32_avx2(
        buf0.as_ptr(),
        buf0.as_mut_ptr(),
        buf_size_nonzero_w_div8 << 3,
        0,
        NEW_INV_SQRT2,
      );
    }
    row_txfm(
      buf0.as_mut_ptr(),
      buf0.as_mut_ptr(),
      INV_COS_BIT_ROW[txw_idx][txh_idx] as i32,
      false,
      bd,
      (-shift[0]) as i32,
    );

    let buf1_slice = &mut buf1[(i * 8)..];
    if lr_flip {
      for j in 0..buf_size_w_div8 {
        transpose_8x8_flip_avx2(
          buf0.as_mut_ptr().add(j * 8),
          buf1_slice
            .as_mut_ptr()
            .add((buf_size_w_div8 - 1 - j) * txfm_size_row),
        );
      }
    } else {
      for j in 0..buf_size_w_div8 {
        transpose_8x8_avx2(
          buf0.as_mut_ptr().add(j * 8),
          buf1_slice.as_mut_ptr().add(j * txfm_size_row),
        );
      }
    }
  }

  // 2nd stage: column transform
  for i in 0..buf_size_w_div8 {
    col_txfm(
      buf1.as_mut_ptr().add(i * txfm_size_row),
      buf1.as_mut_ptr().add(i * txfm_size_row),
      INV_COS_BIT_COL[txw_idx][txh_idx] as i32,
      true,
      bd,
      0,
    );

    round_shift_array_32_avx2(
      buf1.as_ptr().add(i * txfm_size_row),
      buf1.as_mut_ptr().add(i * txfm_size_row),
      txfm_size_row,
      (-shift[1]) as i32,
    );
  }

  // write to buffer
  if txfm_size_col >= 16 {
    for i in 0..(txfm_size_col >> 4) {
      highbd_write_buffer_16xn_avx2(
        buf1.as_ptr().add(i * txfm_size_row * 2),
        output.add(16 * i),
        stride as usize,
        ud_flip,
        txfm_size_row,
        bd,
      );
    }
  } else if txfm_size_col == 8 {
    highbd_write_buffer_8xn_avx2(
      buf1.as_ptr(),
      output,
      stride as usize,
      ud_flip,
      txfm_size_row,
      bd,
    );
  }
}

#[target_feature(enable = "avx2")]
unsafe fn idct8x8_low1_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let mut clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let mut clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  let mut x;

  // stage 0
  // stage 1
  // stage 2
  // stage 3
  x = _mm256_mullo_epi32(input.read(), cospi32);
  x = _mm256_add_epi32(x, rnding);
  x = _mm256_srai_epi32(x, bit);

  // stage 4
  // stage 5
  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    let offset = _mm256_set1_epi32((1 << out_shift) >> 1);
    clamp_lo = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    clamp_hi = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);
    x = _mm256_add_epi32(x, offset);
    x = _mm256_sra_epi32(x, _mm_cvtsi32_si128(out_shift));
  }
  x = _mm256_max_epi32(x, clamp_lo);
  x = _mm256_min_epi32(x, clamp_hi);

  output.add(0).write(x);
  output.add(1).write(x);
  output.add(2).write(x);
  output.add(3).write(x);
  output.add(4).write(x);
  output.add(5).write(x);
  output.add(6).write(x);
  output.add(7).write(x);
}

#[target_feature(enable = "avx2")]
unsafe fn idct8x8_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospim8 = _mm256_set1_epi32(-cospi[8]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospim40 = _mm256_set1_epi32(-cospi[40]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospim16 = _mm256_set1_epi32(-cospi[16]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);

  let mut u0;
  let mut u1;
  let mut u2;
  let mut u3;
  let mut u4;
  let mut u5;
  let mut u6;
  let mut u7;
  let mut v0;
  let mut v1;
  let mut v2;
  let mut v3;
  let mut v4 = _mm256_setzero_si256();
  let mut v5 = _mm256_setzero_si256();
  let mut v6 = _mm256_setzero_si256();
  let mut v7 = _mm256_setzero_si256();
  let mut x;
  let mut y;

  // stage 0
  // stage 1
  // stage 2
  u0 = input.add(0).read();
  u1 = input.add(4).read();
  u2 = input.add(2).read();
  u3 = input.add(6).read();

  x = _mm256_mullo_epi32(input.add(1).read(), cospi56);
  y = _mm256_mullo_epi32(input.add(7).read(), cospim8);
  u4 = _mm256_add_epi32(x, y);
  u4 = _mm256_add_epi32(u4, rnding);
  u4 = _mm256_srai_epi32(u4, bit);

  x = _mm256_mullo_epi32(input.add(1).read(), cospi8);
  y = _mm256_mullo_epi32(input.add(7).read(), cospi56);
  u7 = _mm256_add_epi32(x, y);
  u7 = _mm256_add_epi32(u7, rnding);
  u7 = _mm256_srai_epi32(u7, bit);

  x = _mm256_mullo_epi32(input.add(5).read(), cospi24);
  y = _mm256_mullo_epi32(input.add(3).read(), cospim40);
  u5 = _mm256_add_epi32(x, y);
  u5 = _mm256_add_epi32(u5, rnding);
  u5 = _mm256_srai_epi32(u5, bit);

  x = _mm256_mullo_epi32(input.add(5).read(), cospi40);
  y = _mm256_mullo_epi32(input.add(3).read(), cospi24);
  u6 = _mm256_add_epi32(x, y);
  u6 = _mm256_add_epi32(u6, rnding);
  u6 = _mm256_srai_epi32(u6, bit);

  // stage 3
  x = _mm256_mullo_epi32(u0, cospi32);
  y = _mm256_mullo_epi32(u1, cospi32);
  v0 = _mm256_add_epi32(x, y);
  v0 = _mm256_add_epi32(v0, rnding);
  v0 = _mm256_srai_epi32(v0, bit);

  v1 = _mm256_sub_epi32(x, y);
  v1 = _mm256_add_epi32(v1, rnding);
  v1 = _mm256_srai_epi32(v1, bit);

  x = _mm256_mullo_epi32(u2, cospi48);
  y = _mm256_mullo_epi32(u3, cospim16);
  v2 = _mm256_add_epi32(x, y);
  v2 = _mm256_add_epi32(v2, rnding);
  v2 = _mm256_srai_epi32(v2, bit);

  x = _mm256_mullo_epi32(u2, cospi16);
  y = _mm256_mullo_epi32(u3, cospi48);
  v3 = _mm256_add_epi32(x, y);
  v3 = _mm256_add_epi32(v3, rnding);
  v3 = _mm256_srai_epi32(v3, bit);

  addsub_avx2(u4, u5, &mut v4, &mut v5, clamp_lo, clamp_hi);
  addsub_avx2(u7, u6, &mut v7, &mut v6, clamp_lo, clamp_hi);

  // stage 4
  addsub_avx2(v0, v3, &mut u0, &mut u3, clamp_lo, clamp_hi);
  addsub_avx2(v1, v2, &mut u1, &mut u2, clamp_lo, clamp_hi);
  u4 = v4;
  u7 = v7;

  x = _mm256_mullo_epi32(v5, cospi32);
  y = _mm256_mullo_epi32(v6, cospi32);
  u6 = _mm256_add_epi32(y, x);
  u6 = _mm256_add_epi32(u6, rnding);
  u6 = _mm256_srai_epi32(u6, bit);

  u5 = _mm256_sub_epi32(y, x);
  u5 = _mm256_add_epi32(u5, rnding);
  u5 = _mm256_srai_epi32(u5, bit);

  addsub_avx2(
    u0,
    u7,
    output.add(0).as_mut().unwrap(),
    output.add(7).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u1,
    u6,
    output.add(1).as_mut().unwrap(),
    output.add(6).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u2,
    u5,
    output.add(2).as_mut().unwrap(),
    output.add(5).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u3,
    u4,
    output.add(3).as_mut().unwrap(),
    output.add(4).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  // stage 5
  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);

    round_shift_4x4_avx2(output, out_shift);
    round_shift_4x4_avx2(output.add(4), out_shift);
    highbd_clamp_epi32_avx2(output, output, clamp_lo_out, clamp_hi_out, 8);
  }
}

#[target_feature(enable = "avx2")]
unsafe fn idct16_low1_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let mut clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let mut clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);

  // stage 0
  // stage 1
  // stage 2
  // stage 3
  // stage 4
  input.write(_mm256_mullo_epi32(input.read(), cospi32));
  input.write(_mm256_add_epi32(input.read(), rnding));
  input.write(_mm256_srai_epi32(input.read(), bit));

  // stage 5
  // stage 6
  // stage 7
  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    clamp_lo = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    clamp_hi = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);
    let offset = _mm256_set1_epi32((1 << out_shift) >> 1);
    input.write(_mm256_add_epi32(input.read(), offset));
    input.write(_mm256_sra_epi32(input.read(), _mm_cvtsi32_si128(out_shift)));
  }
  input.write(_mm256_max_epi32(input.read(), clamp_lo));
  input.write(_mm256_min_epi32(input.read(), clamp_hi));
  output.add(0).write(input.read());
  output.add(1).write(input.read());
  output.add(2).write(input.read());
  output.add(3).write(input.read());
  output.add(4).write(input.read());
  output.add(5).write(input.read());
  output.add(6).write(input.read());
  output.add(7).write(input.read());
  output.add(8).write(input.read());
  output.add(9).write(input.read());
  output.add(10).write(input.read());
  output.add(11).write(input.read());
  output.add(12).write(input.read());
  output.add(13).write(input.read());
  output.add(14).write(input.read());
  output.add(15).write(input.read());
}

#[target_feature(enable = "avx2")]
unsafe fn idct16_low8_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospi28 = _mm256_set1_epi32(cospi[28]);
  let cospi44 = _mm256_set1_epi32(cospi[44]);
  let cospi20 = _mm256_set1_epi32(cospi[20]);
  let cospi12 = _mm256_set1_epi32(cospi[12]);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospim40 = _mm256_set1_epi32(-cospi[40]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospim16 = _mm256_set1_epi32(-cospi[16]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospim36 = _mm256_set1_epi32(-cospi[36]);
  let cospim52 = _mm256_set1_epi32(-cospi[52]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  let mut u = [_mm256_setzero_si256(); 16];
  let mut x;
  let mut y;

  // stage 0
  // stage 1
  u[0] = input.add(0).read();
  u[2] = input.add(4).read();
  u[4] = input.add(2).read();
  u[6] = input.add(6).read();
  u[8] = input.add(1).read();
  u[10] = input.add(5).read();
  u[12] = input.add(3).read();
  u[14] = input.add(7).read();

  // stage 2
  u[15] = half_btf_0_avx2(cospi4, u[8], rnding, bit);
  u[8] = half_btf_0_avx2(cospi60, u[8], rnding, bit);

  u[9] = half_btf_0_avx2(cospim36, u[14], rnding, bit);
  u[14] = half_btf_0_avx2(cospi28, u[14], rnding, bit);

  u[13] = half_btf_0_avx2(cospi20, u[10], rnding, bit);
  u[10] = half_btf_0_avx2(cospi44, u[10], rnding, bit);

  u[11] = half_btf_0_avx2(cospim52, u[12], rnding, bit);
  u[12] = half_btf_0_avx2(cospi12, u[12], rnding, bit);

  // stage 3
  u[7] = half_btf_0_avx2(cospi8, u[4], rnding, bit);
  u[4] = half_btf_0_avx2(cospi56, u[4], rnding, bit);
  u[5] = half_btf_0_avx2(cospim40, u[6], rnding, bit);
  u[6] = half_btf_0_avx2(cospi24, u[6], rnding, bit);

  addsub_avx2(u[8], u[9], &mut u[8], &mut u[9], clamp_lo, clamp_hi);
  addsub_avx2(u[11], u[10], &mut u[11], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(u[12], u[13], &mut u[12], &mut u[13], clamp_lo, clamp_hi);
  addsub_avx2(u[15], u[14], &mut u[15], &mut u[14], clamp_lo, clamp_hi);

  // stage 4
  x = _mm256_mullo_epi32(u[0], cospi32);
  u[0] = _mm256_add_epi32(x, rnding);
  u[0] = _mm256_srai_epi32(u[0], bit);
  u[1] = u[0];

  u[3] = half_btf_0_avx2(cospi16, u[2], rnding, bit);
  u[2] = half_btf_0_avx2(cospi48, u[2], rnding, bit);

  addsub_avx2(u[4], u[5], &mut u[4], &mut u[5], clamp_lo, clamp_hi);
  addsub_avx2(u[7], u[6], &mut u[7], &mut u[6], clamp_lo, clamp_hi);

  x = half_btf_avx2(cospim16, u[9], cospi48, u[14], rnding, bit);
  u[14] = half_btf_avx2(cospi48, u[9], cospi16, u[14], rnding, bit);
  u[9] = x;
  y = half_btf_avx2(cospim48, u[10], cospim16, u[13], rnding, bit);
  u[13] = half_btf_avx2(cospim16, u[10], cospi48, u[13], rnding, bit);
  u[10] = y;

  // stage 5
  addsub_avx2(u[0], u[3], &mut u[0], &mut u[3], clamp_lo, clamp_hi);
  addsub_avx2(u[1], u[2], &mut u[1], &mut u[2], clamp_lo, clamp_hi);

  x = _mm256_mullo_epi32(u[5], cospi32);
  y = _mm256_mullo_epi32(u[6], cospi32);
  u[5] = _mm256_sub_epi32(y, x);
  u[5] = _mm256_add_epi32(u[5], rnding);
  u[5] = _mm256_srai_epi32(u[5], bit);

  u[6] = _mm256_add_epi32(y, x);
  u[6] = _mm256_add_epi32(u[6], rnding);
  u[6] = _mm256_srai_epi32(u[6], bit);

  addsub_avx2(u[8], u[11], &mut u[8], &mut u[11], clamp_lo, clamp_hi);
  addsub_avx2(u[9], u[10], &mut u[9], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(u[15], u[12], &mut u[15], &mut u[12], clamp_lo, clamp_hi);
  addsub_avx2(u[14], u[13], &mut u[14], &mut u[13], clamp_lo, clamp_hi);

  // stage 6
  addsub_avx2(u[0], u[7], &mut u[0], &mut u[7], clamp_lo, clamp_hi);
  addsub_avx2(u[1], u[6], &mut u[1], &mut u[6], clamp_lo, clamp_hi);
  addsub_avx2(u[2], u[5], &mut u[2], &mut u[5], clamp_lo, clamp_hi);
  addsub_avx2(u[3], u[4], &mut u[3], &mut u[4], clamp_lo, clamp_hi);

  x = _mm256_mullo_epi32(u[10], cospi32);
  y = _mm256_mullo_epi32(u[13], cospi32);
  u[10] = _mm256_sub_epi32(y, x);
  u[10] = _mm256_add_epi32(u[10], rnding);
  u[10] = _mm256_srai_epi32(u[10], bit);

  u[13] = _mm256_add_epi32(x, y);
  u[13] = _mm256_add_epi32(u[13], rnding);
  u[13] = _mm256_srai_epi32(u[13], bit);

  x = _mm256_mullo_epi32(u[11], cospi32);
  y = _mm256_mullo_epi32(u[12], cospi32);
  u[11] = _mm256_sub_epi32(y, x);
  u[11] = _mm256_add_epi32(u[11], rnding);
  u[11] = _mm256_srai_epi32(u[11], bit);

  u[12] = _mm256_add_epi32(x, y);
  u[12] = _mm256_add_epi32(u[12], rnding);
  u[12] = _mm256_srai_epi32(u[12], bit);
  // stage 7
  addsub_avx2(
    u[0],
    u[15],
    output.add(0).as_mut().unwrap(),
    output.add(15).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u[1],
    u[14],
    output.add(1).as_mut().unwrap(),
    output.add(14).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u[2],
    u[13],
    output.add(2).as_mut().unwrap(),
    output.add(13).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u[3],
    u[12],
    output.add(3).as_mut().unwrap(),
    output.add(12).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u[4],
    u[11],
    output.add(4).as_mut().unwrap(),
    output.add(11).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u[5],
    u[10],
    output.add(5).as_mut().unwrap(),
    output.add(10).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u[6],
    u[9],
    output.add(6).as_mut().unwrap(),
    output.add(9).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    u[7],
    u[8],
    output.add(7).as_mut().unwrap(),
    output.add(8).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );

  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);
    round_shift_8x8_avx2(output, out_shift);
    highbd_clamp_epi32_avx2(output, output, clamp_lo_out, clamp_hi_out, 16);
  }
}

#[target_feature(enable = "avx2")]
unsafe fn idct16_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospim4 = _mm256_set1_epi32(-cospi[4]);
  let cospi28 = _mm256_set1_epi32(cospi[28]);
  let cospim36 = _mm256_set1_epi32(-cospi[36]);
  let cospi44 = _mm256_set1_epi32(cospi[44]);
  let cospi20 = _mm256_set1_epi32(cospi[20]);
  let cospim20 = _mm256_set1_epi32(-cospi[20]);
  let cospi12 = _mm256_set1_epi32(cospi[12]);
  let cospim52 = _mm256_set1_epi32(-cospi[52]);
  let cospi52 = _mm256_set1_epi32(cospi[52]);
  let cospi36 = _mm256_set1_epi32(cospi[36]);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospim8 = _mm256_set1_epi32(-cospi[8]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospim40 = _mm256_set1_epi32(-cospi[40]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospim16 = _mm256_set1_epi32(-cospi[16]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  let mut u = [_mm256_setzero_si256(); 16];
  let mut v = [_mm256_setzero_si256(); 16];
  let mut x;
  let mut y;

  // stage 0
  // stage 1
  u[0] = input.add(0).read();
  u[1] = input.add(8).read();
  u[2] = input.add(4).read();
  u[3] = input.add(12).read();
  u[4] = input.add(2).read();
  u[5] = input.add(10).read();
  u[6] = input.add(6).read();
  u[7] = input.add(14).read();
  u[8] = input.add(1).read();
  u[9] = input.add(9).read();
  u[10] = input.add(5).read();
  u[11] = input.add(13).read();
  u[12] = input.add(3).read();
  u[13] = input.add(11).read();
  u[14] = input.add(7).read();
  u[15] = input.add(15).read();

  // stage 2
  v[0] = u[0];
  v[1] = u[1];
  v[2] = u[2];
  v[3] = u[3];
  v[4] = u[4];
  v[5] = u[5];
  v[6] = u[6];
  v[7] = u[7];

  v[8] = half_btf_avx2(cospi60, u[8], cospim4, u[15], rnding, bit);
  v[9] = half_btf_avx2(cospi28, u[9], cospim36, u[14], rnding, bit);
  v[10] = half_btf_avx2(cospi44, u[10], cospim20, u[13], rnding, bit);
  v[11] = half_btf_avx2(cospi12, u[11], cospim52, u[12], rnding, bit);
  v[12] = half_btf_avx2(cospi52, u[11], cospi12, u[12], rnding, bit);
  v[13] = half_btf_avx2(cospi20, u[10], cospi44, u[13], rnding, bit);
  v[14] = half_btf_avx2(cospi36, u[9], cospi28, u[14], rnding, bit);
  v[15] = half_btf_avx2(cospi4, u[8], cospi60, u[15], rnding, bit);

  // stage 3
  u[0] = v[0];
  u[1] = v[1];
  u[2] = v[2];
  u[3] = v[3];
  u[4] = half_btf_avx2(cospi56, v[4], cospim8, v[7], rnding, bit);
  u[5] = half_btf_avx2(cospi24, v[5], cospim40, v[6], rnding, bit);
  u[6] = half_btf_avx2(cospi40, v[5], cospi24, v[6], rnding, bit);
  u[7] = half_btf_avx2(cospi8, v[4], cospi56, v[7], rnding, bit);
  addsub_avx2(v[8], v[9], &mut u[8], &mut u[9], clamp_lo, clamp_hi);
  addsub_avx2(v[11], v[10], &mut u[11], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(v[12], v[13], &mut u[12], &mut u[13], clamp_lo, clamp_hi);
  addsub_avx2(v[15], v[14], &mut u[15], &mut u[14], clamp_lo, clamp_hi);

  // stage 4
  x = _mm256_mullo_epi32(u[0], cospi32);
  y = _mm256_mullo_epi32(u[1], cospi32);
  v[0] = _mm256_add_epi32(x, y);
  v[0] = _mm256_add_epi32(v[0], rnding);
  v[0] = _mm256_srai_epi32(v[0], bit);

  v[1] = _mm256_sub_epi32(x, y);
  v[1] = _mm256_add_epi32(v[1], rnding);
  v[1] = _mm256_srai_epi32(v[1], bit);

  v[2] = half_btf_avx2(cospi48, u[2], cospim16, u[3], rnding, bit);
  v[3] = half_btf_avx2(cospi16, u[2], cospi48, u[3], rnding, bit);
  addsub_avx2(u[4], u[5], &mut v[4], &mut v[5], clamp_lo, clamp_hi);
  addsub_avx2(u[7], u[6], &mut v[7], &mut v[6], clamp_lo, clamp_hi);
  v[8] = u[8];
  v[9] = half_btf_avx2(cospim16, u[9], cospi48, u[14], rnding, bit);
  v[10] = half_btf_avx2(cospim48, u[10], cospim16, u[13], rnding, bit);
  v[11] = u[11];
  v[12] = u[12];
  v[13] = half_btf_avx2(cospim16, u[10], cospi48, u[13], rnding, bit);
  v[14] = half_btf_avx2(cospi48, u[9], cospi16, u[14], rnding, bit);
  v[15] = u[15];

  // stage 5
  addsub_avx2(v[0], v[3], &mut u[0], &mut u[3], clamp_lo, clamp_hi);
  addsub_avx2(v[1], v[2], &mut u[1], &mut u[2], clamp_lo, clamp_hi);
  u[4] = v[4];

  x = _mm256_mullo_epi32(v[5], cospi32);
  y = _mm256_mullo_epi32(v[6], cospi32);
  u[5] = _mm256_sub_epi32(y, x);
  u[5] = _mm256_add_epi32(u[5], rnding);
  u[5] = _mm256_srai_epi32(u[5], bit);

  u[6] = _mm256_add_epi32(y, x);
  u[6] = _mm256_add_epi32(u[6], rnding);
  u[6] = _mm256_srai_epi32(u[6], bit);

  u[7] = v[7];
  addsub_avx2(v[8], v[11], &mut u[8], &mut u[11], clamp_lo, clamp_hi);
  addsub_avx2(v[9], v[10], &mut u[9], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(v[15], v[12], &mut u[15], &mut u[12], clamp_lo, clamp_hi);
  addsub_avx2(v[14], v[13], &mut u[14], &mut u[13], clamp_lo, clamp_hi);

  // stage 6
  addsub_avx2(u[0], u[7], &mut v[0], &mut v[7], clamp_lo, clamp_hi);
  addsub_avx2(u[1], u[6], &mut v[1], &mut v[6], clamp_lo, clamp_hi);
  addsub_avx2(u[2], u[5], &mut v[2], &mut v[5], clamp_lo, clamp_hi);
  addsub_avx2(u[3], u[4], &mut v[3], &mut v[4], clamp_lo, clamp_hi);
  v[8] = u[8];
  v[9] = u[9];

  x = _mm256_mullo_epi32(u[10], cospi32);
  y = _mm256_mullo_epi32(u[13], cospi32);
  v[10] = _mm256_sub_epi32(y, x);
  v[10] = _mm256_add_epi32(v[10], rnding);
  v[10] = _mm256_srai_epi32(v[10], bit);

  v[13] = _mm256_add_epi32(x, y);
  v[13] = _mm256_add_epi32(v[13], rnding);
  v[13] = _mm256_srai_epi32(v[13], bit);

  x = _mm256_mullo_epi32(u[11], cospi32);
  y = _mm256_mullo_epi32(u[12], cospi32);
  v[11] = _mm256_sub_epi32(y, x);
  v[11] = _mm256_add_epi32(v[11], rnding);
  v[11] = _mm256_srai_epi32(v[11], bit);

  v[12] = _mm256_add_epi32(x, y);
  v[12] = _mm256_add_epi32(v[12], rnding);
  v[12] = _mm256_srai_epi32(v[12], bit);

  v[14] = u[14];
  v[15] = u[15];

  // stage 7
  addsub_avx2(
    v[0],
    v[15],
    output.add(0).as_mut().unwrap(),
    output.add(15).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    v[1],
    v[14],
    output.add(1).as_mut().unwrap(),
    output.add(14).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    v[2],
    v[13],
    output.add(2).as_mut().unwrap(),
    output.add(13).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    v[3],
    v[12],
    output.add(3).as_mut().unwrap(),
    output.add(12).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    v[4],
    v[11],
    output.add(4).as_mut().unwrap(),
    output.add(11).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    v[5],
    v[10],
    output.add(5).as_mut().unwrap(),
    output.add(10).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    v[6],
    v[9],
    output.add(6).as_mut().unwrap(),
    output.add(9).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    v[7],
    v[8],
    output.add(7).as_mut().unwrap(),
    output.add(8).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );

  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);
    round_shift_8x8_avx2(output, out_shift);
    highbd_clamp_epi32_avx2(output, output, clamp_lo_out, clamp_hi_out, 16);
  }
}

#[target_feature(enable = "avx2")]
unsafe fn idct32_low1_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let rounding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let mut clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let mut clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  // stage 0
  // stage 1
  // stage 2
  // stage 3
  // stage 4
  // stage 5
  let mut x = _mm256_mullo_epi32(input.read(), cospi32);
  x = _mm256_add_epi32(x, rounding);
  x = _mm256_srai_epi32(x, bit);

  // stage 6
  // stage 7
  // stage 8
  // stage 9
  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    let offset = _mm256_set1_epi32((1 << out_shift) >> 1);
    clamp_lo = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    clamp_hi = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);
    x = _mm256_add_epi32(offset, x);
    x = _mm256_sra_epi32(x, _mm_cvtsi32_si128(out_shift));
  }
  x = _mm256_max_epi32(x, clamp_lo);
  x = _mm256_min_epi32(x, clamp_hi);
  output.add(0).write(x);
  output.add(1).write(x);
  output.add(2).write(x);
  output.add(3).write(x);
  output.add(4).write(x);
  output.add(5).write(x);
  output.add(6).write(x);
  output.add(7).write(x);
  output.add(8).write(x);
  output.add(9).write(x);
  output.add(10).write(x);
  output.add(11).write(x);
  output.add(12).write(x);
  output.add(13).write(x);
  output.add(14).write(x);
  output.add(15).write(x);
  output.add(16).write(x);
  output.add(17).write(x);
  output.add(18).write(x);
  output.add(19).write(x);
  output.add(20).write(x);
  output.add(21).write(x);
  output.add(22).write(x);
  output.add(23).write(x);
  output.add(24).write(x);
  output.add(25).write(x);
  output.add(26).write(x);
  output.add(27).write(x);
  output.add(28).write(x);
  output.add(29).write(x);
  output.add(30).write(x);
  output.add(31).write(x);
}

#[target_feature(enable = "avx2")]
unsafe fn idct32_low8_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi62 = _mm256_set1_epi32(cospi[62]);
  let cospi14 = _mm256_set1_epi32(cospi[14]);
  let cospi54 = _mm256_set1_epi32(cospi[54]);
  let cospi6 = _mm256_set1_epi32(cospi[6]);
  let cospi10 = _mm256_set1_epi32(cospi[10]);
  let cospi2 = _mm256_set1_epi32(cospi[2]);
  let cospim58 = _mm256_set1_epi32(-cospi[58]);
  let cospim50 = _mm256_set1_epi32(-cospi[50]);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospi12 = _mm256_set1_epi32(cospi[12]);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospim52 = _mm256_set1_epi32(-cospi[52]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospim40 = _mm256_set1_epi32(-cospi[40]);
  let cospim8 = _mm256_set1_epi32(-cospi[8]);
  let cospim56 = _mm256_set1_epi32(-cospi[56]);
  let cospim24 = _mm256_set1_epi32(-cospi[24]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let cospim32 = _mm256_set1_epi32(-cospi[32]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospim16 = _mm256_set1_epi32(-cospi[16]);
  let rounding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  let mut bf1 = [_mm256_setzero_si256(); 32];

  // stage 0
  // stage 1
  bf1[0] = input.add(0).read();
  bf1[4] = input.add(4).read();
  bf1[8] = input.add(2).read();
  bf1[12] = input.add(6).read();
  bf1[16] = input.add(1).read();
  bf1[20] = input.add(5).read();
  bf1[24] = input.add(3).read();
  bf1[28] = input.add(7).read();

  // stage 2
  bf1[31] = half_btf_0_avx2(cospi2, bf1[16], rounding, bit);
  bf1[16] = half_btf_0_avx2(cospi62, bf1[16], rounding, bit);
  bf1[19] = half_btf_0_avx2(cospim50, bf1[28], rounding, bit);
  bf1[28] = half_btf_0_avx2(cospi14, bf1[28], rounding, bit);
  bf1[27] = half_btf_0_avx2(cospi10, bf1[20], rounding, bit);
  bf1[20] = half_btf_0_avx2(cospi54, bf1[20], rounding, bit);
  bf1[23] = half_btf_0_avx2(cospim58, bf1[24], rounding, bit);
  bf1[24] = half_btf_0_avx2(cospi6, bf1[24], rounding, bit);

  // stage 3
  bf1[15] = half_btf_0_avx2(cospi4, bf1[8], rounding, bit);
  bf1[8] = half_btf_0_avx2(cospi60, bf1[8], rounding, bit);

  bf1[11] = half_btf_0_avx2(cospim52, bf1[12], rounding, bit);
  bf1[12] = half_btf_0_avx2(cospi12, bf1[12], rounding, bit);
  bf1[17] = bf1[16];
  bf1[18] = bf1[19];
  bf1[21] = bf1[20];
  bf1[22] = bf1[23];
  bf1[25] = bf1[24];
  bf1[26] = bf1[27];
  bf1[29] = bf1[28];
  bf1[30] = bf1[31];

  // stage 4
  bf1[7] = half_btf_0_avx2(cospi8, bf1[4], rounding, bit);
  bf1[4] = half_btf_0_avx2(cospi56, bf1[4], rounding, bit);

  bf1[9] = bf1[8];
  bf1[10] = bf1[11];
  bf1[13] = bf1[12];
  bf1[14] = bf1[15];

  idct32_stage4_avx2(
    bf1.as_mut_ptr(),
    cospim8,
    cospi56,
    cospi8,
    cospim56,
    cospim40,
    cospi24,
    cospi40,
    cospim24,
    rounding,
    bit,
  );

  // stage 5
  bf1[0] = half_btf_0_avx2(cospi32, bf1[0], rounding, bit);
  bf1[1] = bf1[0];
  bf1[5] = bf1[4];
  bf1[6] = bf1[7];

  idct32_stage5_avx2(
    bf1.as_mut_ptr(),
    cospim16,
    cospi48,
    cospi16,
    cospim48,
    clamp_lo,
    clamp_hi,
    rounding,
    bit,
  );

  // stage 6
  bf1[3] = bf1[0];
  bf1[2] = bf1[1];

  idct32_stage6_avx2(
    bf1.as_mut_ptr(),
    cospim32,
    cospi32,
    cospim16,
    cospi48,
    cospi16,
    cospim48,
    clamp_lo,
    clamp_hi,
    rounding,
    bit,
  );

  // stage 7
  idct32_stage7_avx2(
    bf1.as_mut_ptr(),
    cospim32,
    cospi32,
    clamp_lo,
    clamp_hi,
    rounding,
    bit,
  );

  // stage 8
  idct32_stage8_avx2(
    bf1.as_mut_ptr(),
    cospim32,
    cospi32,
    clamp_lo,
    clamp_hi,
    rounding,
    bit,
  );

  // stage 9
  idct32_stage9_avx2(
    bf1.as_mut_ptr(),
    output,
    do_cols,
    bd,
    out_shift,
    clamp_lo,
    clamp_hi,
  );
}

#[target_feature(enable = "avx2")]
unsafe fn idct32_low16_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi62 = _mm256_set1_epi32(cospi[62]);
  let cospi30 = _mm256_set1_epi32(cospi[30]);
  let cospi46 = _mm256_set1_epi32(cospi[46]);
  let cospi14 = _mm256_set1_epi32(cospi[14]);
  let cospi54 = _mm256_set1_epi32(cospi[54]);
  let cospi22 = _mm256_set1_epi32(cospi[22]);
  let cospi38 = _mm256_set1_epi32(cospi[38]);
  let cospi6 = _mm256_set1_epi32(cospi[6]);
  let cospi26 = _mm256_set1_epi32(cospi[26]);
  let cospi10 = _mm256_set1_epi32(cospi[10]);
  let cospi18 = _mm256_set1_epi32(cospi[18]);
  let cospi2 = _mm256_set1_epi32(cospi[2]);
  let cospim58 = _mm256_set1_epi32(-cospi[58]);
  let cospim42 = _mm256_set1_epi32(-cospi[42]);
  let cospim50 = _mm256_set1_epi32(-cospi[50]);
  let cospim34 = _mm256_set1_epi32(-cospi[34]);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospi28 = _mm256_set1_epi32(cospi[28]);
  let cospi44 = _mm256_set1_epi32(cospi[44]);
  let cospi12 = _mm256_set1_epi32(cospi[12]);
  let cospi20 = _mm256_set1_epi32(cospi[20]);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospim52 = _mm256_set1_epi32(-cospi[52]);
  let cospim36 = _mm256_set1_epi32(-cospi[36]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospim40 = _mm256_set1_epi32(-cospi[40]);
  let cospim8 = _mm256_set1_epi32(-cospi[8]);
  let cospim56 = _mm256_set1_epi32(-cospi[56]);
  let cospim24 = _mm256_set1_epi32(-cospi[24]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let cospim32 = _mm256_set1_epi32(-cospi[32]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospim16 = _mm256_set1_epi32(-cospi[16]);
  let rounding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  let mut bf1 = [_mm256_setzero_si256(); 32];

  // stage 0
  // stage 1
  bf1[0] = input.add(0).read();
  bf1[2] = input.add(8).read();
  bf1[4] = input.add(4).read();
  bf1[6] = input.add(12).read();
  bf1[8] = input.add(2).read();
  bf1[10] = input.add(10).read();
  bf1[12] = input.add(6).read();
  bf1[14] = input.add(14).read();
  bf1[16] = input.add(1).read();
  bf1[18] = input.add(9).read();
  bf1[20] = input.add(5).read();
  bf1[22] = input.add(13).read();
  bf1[24] = input.add(3).read();
  bf1[26] = input.add(11).read();
  bf1[28] = input.add(7).read();
  bf1[30] = input.add(15).read();

  // stage 2
  bf1[31] = half_btf_0_avx2(cospi2, bf1[16], rounding, bit);
  bf1[16] = half_btf_0_avx2(cospi62, bf1[16], rounding, bit);
  bf1[17] = half_btf_0_avx2(cospim34, bf1[30], rounding, bit);
  bf1[30] = half_btf_0_avx2(cospi30, bf1[30], rounding, bit);
  bf1[29] = half_btf_0_avx2(cospi18, bf1[18], rounding, bit);
  bf1[18] = half_btf_0_avx2(cospi46, bf1[18], rounding, bit);
  bf1[19] = half_btf_0_avx2(cospim50, bf1[28], rounding, bit);
  bf1[28] = half_btf_0_avx2(cospi14, bf1[28], rounding, bit);
  bf1[27] = half_btf_0_avx2(cospi10, bf1[20], rounding, bit);
  bf1[20] = half_btf_0_avx2(cospi54, bf1[20], rounding, bit);
  bf1[21] = half_btf_0_avx2(cospim42, bf1[26], rounding, bit);
  bf1[26] = half_btf_0_avx2(cospi22, bf1[26], rounding, bit);
  bf1[25] = half_btf_0_avx2(cospi26, bf1[22], rounding, bit);
  bf1[22] = half_btf_0_avx2(cospi38, bf1[22], rounding, bit);
  bf1[23] = half_btf_0_avx2(cospim58, bf1[24], rounding, bit);
  bf1[24] = half_btf_0_avx2(cospi6, bf1[24], rounding, bit);

  // stage 3
  bf1[15] = half_btf_0_avx2(cospi4, bf1[8], rounding, bit);
  bf1[8] = half_btf_0_avx2(cospi60, bf1[8], rounding, bit);
  bf1[9] = half_btf_0_avx2(cospim36, bf1[14], rounding, bit);
  bf1[14] = half_btf_0_avx2(cospi28, bf1[14], rounding, bit);
  bf1[13] = half_btf_0_avx2(cospi20, bf1[10], rounding, bit);
  bf1[10] = half_btf_0_avx2(cospi44, bf1[10], rounding, bit);
  bf1[11] = half_btf_0_avx2(cospim52, bf1[12], rounding, bit);
  bf1[12] = half_btf_0_avx2(cospi12, bf1[12], rounding, bit);

  addsub_avx2(
    bf1[16],
    bf1[17],
    &mut bf1[16],
    &mut bf1[17],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[19],
    bf1[18],
    &mut bf1[19],
    &mut bf1[18],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[20],
    bf1[21],
    &mut bf1[20],
    &mut bf1[21],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[23],
    bf1[22],
    &mut bf1[23],
    &mut bf1[22],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[24],
    bf1[25],
    &mut bf1[24],
    &mut bf1[25],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[27],
    bf1[26],
    &mut bf1[27],
    &mut bf1[26],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[28],
    bf1[29],
    &mut bf1[28],
    &mut bf1[29],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[31],
    bf1[30],
    &mut bf1[31],
    &mut bf1[30],
    clamp_lo,
    clamp_hi,
  );

  // stage 4
  bf1[7] = half_btf_0_avx2(cospi8, bf1[4], rounding, bit);
  bf1[4] = half_btf_0_avx2(cospi56, bf1[4], rounding, bit);
  bf1[5] = half_btf_0_avx2(cospim40, bf1[6], rounding, bit);
  bf1[6] = half_btf_0_avx2(cospi24, bf1[6], rounding, bit);

  addsub_avx2(bf1[8], bf1[9], &mut bf1[8], &mut bf1[9], clamp_lo, clamp_hi);
  addsub_avx2(
    bf1[11],
    bf1[10],
    &mut bf1[11],
    &mut bf1[10],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[12],
    bf1[13],
    &mut bf1[12],
    &mut bf1[13],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[15],
    bf1[14],
    &mut bf1[15],
    &mut bf1[14],
    clamp_lo,
    clamp_hi,
  );

  idct32_stage4_avx2(
    bf1.as_mut_ptr(),
    cospim8,
    cospi56,
    cospi8,
    cospim56,
    cospim40,
    cospi24,
    cospi40,
    cospim24,
    rounding,
    bit,
  );

  // stage 5
  bf1[0] = half_btf_0_avx2(cospi32, bf1[0], rounding, bit);
  bf1[1] = bf1[0];
  bf1[3] = half_btf_0_avx2(cospi16, bf1[2], rounding, bit);
  bf1[2] = half_btf_0_avx2(cospi48, bf1[2], rounding, bit);

  addsub_avx2(bf1[4], bf1[5], &mut bf1[4], &mut bf1[5], clamp_lo, clamp_hi);
  addsub_avx2(bf1[7], bf1[6], &mut bf1[7], &mut bf1[6], clamp_lo, clamp_hi);

  idct32_stage5_avx2(
    bf1.as_mut_ptr(),
    cospim16,
    cospi48,
    cospi16,
    cospim48,
    clamp_lo,
    clamp_hi,
    rounding,
    bit,
  );

  // stage 6
  addsub_avx2(bf1[0], bf1[3], &mut bf1[0], &mut bf1[3], clamp_lo, clamp_hi);
  addsub_avx2(bf1[1], bf1[2], &mut bf1[1], &mut bf1[2], clamp_lo, clamp_hi);

  idct32_stage6_avx2(
    bf1.as_mut_ptr(),
    cospim32,
    cospi32,
    cospim16,
    cospi48,
    cospi16,
    cospim48,
    clamp_lo,
    clamp_hi,
    rounding,
    bit,
  );

  // stage 7
  idct32_stage7_avx2(
    bf1.as_mut_ptr(),
    cospim32,
    cospi32,
    clamp_lo,
    clamp_hi,
    rounding,
    bit,
  );

  // stage 8
  idct32_stage8_avx2(
    bf1.as_mut_ptr(),
    cospim32,
    cospi32,
    clamp_lo,
    clamp_hi,
    rounding,
    bit,
  );

  // stage 9
  idct32_stage9_avx2(
    bf1.as_mut_ptr(),
    output,
    do_cols,
    bd,
    out_shift,
    clamp_lo,
    clamp_hi,
  );
}

#[target_feature(enable = "avx2")]
unsafe fn idct32_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi62 = _mm256_set1_epi32(cospi[62]);
  let cospi30 = _mm256_set1_epi32(cospi[30]);
  let cospi46 = _mm256_set1_epi32(cospi[46]);
  let cospi14 = _mm256_set1_epi32(cospi[14]);
  let cospi54 = _mm256_set1_epi32(cospi[54]);
  let cospi22 = _mm256_set1_epi32(cospi[22]);
  let cospi38 = _mm256_set1_epi32(cospi[38]);
  let cospi6 = _mm256_set1_epi32(cospi[6]);
  let cospi58 = _mm256_set1_epi32(cospi[58]);
  let cospi26 = _mm256_set1_epi32(cospi[26]);
  let cospi42 = _mm256_set1_epi32(cospi[42]);
  let cospi10 = _mm256_set1_epi32(cospi[10]);
  let cospi50 = _mm256_set1_epi32(cospi[50]);
  let cospi18 = _mm256_set1_epi32(cospi[18]);
  let cospi34 = _mm256_set1_epi32(cospi[34]);
  let cospi2 = _mm256_set1_epi32(cospi[2]);
  let cospim58 = _mm256_set1_epi32(-cospi[58]);
  let cospim26 = _mm256_set1_epi32(-cospi[26]);
  let cospim42 = _mm256_set1_epi32(-cospi[42]);
  let cospim10 = _mm256_set1_epi32(-cospi[10]);
  let cospim50 = _mm256_set1_epi32(-cospi[50]);
  let cospim18 = _mm256_set1_epi32(-cospi[18]);
  let cospim34 = _mm256_set1_epi32(-cospi[34]);
  let cospim2 = _mm256_set1_epi32(-cospi[2]);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospi28 = _mm256_set1_epi32(cospi[28]);
  let cospi44 = _mm256_set1_epi32(cospi[44]);
  let cospi12 = _mm256_set1_epi32(cospi[12]);
  let cospi52 = _mm256_set1_epi32(cospi[52]);
  let cospi20 = _mm256_set1_epi32(cospi[20]);
  let cospi36 = _mm256_set1_epi32(cospi[36]);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospim52 = _mm256_set1_epi32(-cospi[52]);
  let cospim20 = _mm256_set1_epi32(-cospi[20]);
  let cospim36 = _mm256_set1_epi32(-cospi[36]);
  let cospim4 = _mm256_set1_epi32(-cospi[4]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospim40 = _mm256_set1_epi32(-cospi[40]);
  let cospim8 = _mm256_set1_epi32(-cospi[8]);
  let cospim56 = _mm256_set1_epi32(-cospi[56]);
  let cospim24 = _mm256_set1_epi32(-cospi[24]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let cospim32 = _mm256_set1_epi32(-cospi[32]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospim16 = _mm256_set1_epi32(-cospi[16]);
  let rounding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  let mut bf1 = [_mm256_setzero_si256(); 32];
  let mut bf0 = [_mm256_setzero_si256(); 32];

  // stage 0
  // stage 1
  bf1[0] = input.add(0).read();
  bf1[1] = input.add(16).read();
  bf1[2] = input.add(8).read();
  bf1[3] = input.add(24).read();
  bf1[4] = input.add(4).read();
  bf1[5] = input.add(20).read();
  bf1[6] = input.add(12).read();
  bf1[7] = input.add(28).read();
  bf1[8] = input.add(2).read();
  bf1[9] = input.add(18).read();
  bf1[10] = input.add(10).read();
  bf1[11] = input.add(26).read();
  bf1[12] = input.add(6).read();
  bf1[13] = input.add(22).read();
  bf1[14] = input.add(14).read();
  bf1[15] = input.add(30).read();
  bf1[16] = input.add(1).read();
  bf1[17] = input.add(17).read();
  bf1[18] = input.add(9).read();
  bf1[19] = input.add(25).read();
  bf1[20] = input.add(5).read();
  bf1[21] = input.add(21).read();
  bf1[22] = input.add(13).read();
  bf1[23] = input.add(29).read();
  bf1[24] = input.add(3).read();
  bf1[25] = input.add(19).read();
  bf1[26] = input.add(11).read();
  bf1[27] = input.add(27).read();
  bf1[28] = input.add(7).read();
  bf1[29] = input.add(23).read();
  bf1[30] = input.add(15).read();
  bf1[31] = input.add(31).read();

  // stage 2
  bf0[0] = bf1[0];
  bf0[1] = bf1[1];
  bf0[2] = bf1[2];
  bf0[3] = bf1[3];
  bf0[4] = bf1[4];
  bf0[5] = bf1[5];
  bf0[6] = bf1[6];
  bf0[7] = bf1[7];
  bf0[8] = bf1[8];
  bf0[9] = bf1[9];
  bf0[10] = bf1[10];
  bf0[11] = bf1[11];
  bf0[12] = bf1[12];
  bf0[13] = bf1[13];
  bf0[14] = bf1[14];
  bf0[15] = bf1[15];
  bf0[16] = half_btf_avx2(cospi62, bf1[16], cospim2, bf1[31], rounding, bit);
  bf0[17] = half_btf_avx2(cospi30, bf1[17], cospim34, bf1[30], rounding, bit);
  bf0[18] = half_btf_avx2(cospi46, bf1[18], cospim18, bf1[29], rounding, bit);
  bf0[19] = half_btf_avx2(cospi14, bf1[19], cospim50, bf1[28], rounding, bit);
  bf0[20] = half_btf_avx2(cospi54, bf1[20], cospim10, bf1[27], rounding, bit);
  bf0[21] = half_btf_avx2(cospi22, bf1[21], cospim42, bf1[26], rounding, bit);
  bf0[22] = half_btf_avx2(cospi38, bf1[22], cospim26, bf1[25], rounding, bit);
  bf0[23] = half_btf_avx2(cospi6, bf1[23], cospim58, bf1[24], rounding, bit);
  bf0[24] = half_btf_avx2(cospi58, bf1[23], cospi6, bf1[24], rounding, bit);
  bf0[25] = half_btf_avx2(cospi26, bf1[22], cospi38, bf1[25], rounding, bit);
  bf0[26] = half_btf_avx2(cospi42, bf1[21], cospi22, bf1[26], rounding, bit);
  bf0[27] = half_btf_avx2(cospi10, bf1[20], cospi54, bf1[27], rounding, bit);
  bf0[28] = half_btf_avx2(cospi50, bf1[19], cospi14, bf1[28], rounding, bit);
  bf0[29] = half_btf_avx2(cospi18, bf1[18], cospi46, bf1[29], rounding, bit);
  bf0[30] = half_btf_avx2(cospi34, bf1[17], cospi30, bf1[30], rounding, bit);
  bf0[31] = half_btf_avx2(cospi2, bf1[16], cospi62, bf1[31], rounding, bit);

  // stage 3
  bf1[0] = bf0[0];
  bf1[1] = bf0[1];
  bf1[2] = bf0[2];
  bf1[3] = bf0[3];
  bf1[4] = bf0[4];
  bf1[5] = bf0[5];
  bf1[6] = bf0[6];
  bf1[7] = bf0[7];
  bf1[8] = half_btf_avx2(cospi60, bf0[8], cospim4, bf0[15], rounding, bit);
  bf1[9] = half_btf_avx2(cospi28, bf0[9], cospim36, bf0[14], rounding, bit);
  bf1[10] = half_btf_avx2(cospi44, bf0[10], cospim20, bf0[13], rounding, bit);
  bf1[11] = half_btf_avx2(cospi12, bf0[11], cospim52, bf0[12], rounding, bit);
  bf1[12] = half_btf_avx2(cospi52, bf0[11], cospi12, bf0[12], rounding, bit);
  bf1[13] = half_btf_avx2(cospi20, bf0[10], cospi44, bf0[13], rounding, bit);
  bf1[14] = half_btf_avx2(cospi36, bf0[9], cospi28, bf0[14], rounding, bit);
  bf1[15] = half_btf_avx2(cospi4, bf0[8], cospi60, bf0[15], rounding, bit);

  addsub_avx2(
    bf0[16],
    bf0[17],
    &mut bf1[16],
    &mut bf1[17],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[19],
    bf0[18],
    &mut bf1[19],
    &mut bf1[18],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[20],
    bf0[21],
    &mut bf1[20],
    &mut bf1[21],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[23],
    bf0[22],
    &mut bf1[23],
    &mut bf1[22],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[24],
    bf0[25],
    &mut bf1[24],
    &mut bf1[25],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[27],
    bf0[26],
    &mut bf1[27],
    &mut bf1[26],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[28],
    bf0[29],
    &mut bf1[28],
    &mut bf1[29],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[31],
    bf0[30],
    &mut bf1[31],
    &mut bf1[30],
    clamp_lo,
    clamp_hi,
  );

  // stage 4
  bf0[0] = bf1[0];
  bf0[1] = bf1[1];
  bf0[2] = bf1[2];
  bf0[3] = bf1[3];
  bf0[4] = half_btf_avx2(cospi56, bf1[4], cospim8, bf1[7], rounding, bit);
  bf0[5] = half_btf_avx2(cospi24, bf1[5], cospim40, bf1[6], rounding, bit);
  bf0[6] = half_btf_avx2(cospi40, bf1[5], cospi24, bf1[6], rounding, bit);
  bf0[7] = half_btf_avx2(cospi8, bf1[4], cospi56, bf1[7], rounding, bit);

  addsub_avx2(bf1[8], bf1[9], &mut bf0[8], &mut bf0[9], clamp_lo, clamp_hi);
  addsub_avx2(
    bf1[11],
    bf1[10],
    &mut bf0[11],
    &mut bf0[10],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[12],
    bf1[13],
    &mut bf0[12],
    &mut bf0[13],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[15],
    bf1[14],
    &mut bf0[15],
    &mut bf0[14],
    clamp_lo,
    clamp_hi,
  );

  bf0[16] = bf1[16];
  bf0[17] = half_btf_avx2(cospim8, bf1[17], cospi56, bf1[30], rounding, bit);
  bf0[18] = half_btf_avx2(cospim56, bf1[18], cospim8, bf1[29], rounding, bit);
  bf0[19] = bf1[19];
  bf0[20] = bf1[20];
  bf0[21] = half_btf_avx2(cospim40, bf1[21], cospi24, bf1[26], rounding, bit);
  bf0[22] = half_btf_avx2(cospim24, bf1[22], cospim40, bf1[25], rounding, bit);
  bf0[23] = bf1[23];
  bf0[24] = bf1[24];
  bf0[25] = half_btf_avx2(cospim40, bf1[22], cospi24, bf1[25], rounding, bit);
  bf0[26] = half_btf_avx2(cospi24, bf1[21], cospi40, bf1[26], rounding, bit);
  bf0[27] = bf1[27];
  bf0[28] = bf1[28];
  bf0[29] = half_btf_avx2(cospim8, bf1[18], cospi56, bf1[29], rounding, bit);
  bf0[30] = half_btf_avx2(cospi56, bf1[17], cospi8, bf1[30], rounding, bit);
  bf0[31] = bf1[31];

  // stage 5
  bf1[0] = half_btf_avx2(cospi32, bf0[0], cospi32, bf0[1], rounding, bit);
  bf1[1] = half_btf_avx2(cospi32, bf0[0], cospim32, bf0[1], rounding, bit);
  bf1[2] = half_btf_avx2(cospi48, bf0[2], cospim16, bf0[3], rounding, bit);
  bf1[3] = half_btf_avx2(cospi16, bf0[2], cospi48, bf0[3], rounding, bit);
  addsub_avx2(bf0[4], bf0[5], &mut bf1[4], &mut bf1[5], clamp_lo, clamp_hi);
  addsub_avx2(bf0[7], bf0[6], &mut bf1[7], &mut bf1[6], clamp_lo, clamp_hi);
  bf1[8] = bf0[8];
  bf1[9] = half_btf_avx2(cospim16, bf0[9], cospi48, bf0[14], rounding, bit);
  bf1[10] = half_btf_avx2(cospim48, bf0[10], cospim16, bf0[13], rounding, bit);
  bf1[11] = bf0[11];
  bf1[12] = bf0[12];
  bf1[13] = half_btf_avx2(cospim16, bf0[10], cospi48, bf0[13], rounding, bit);
  bf1[14] = half_btf_avx2(cospi48, bf0[9], cospi16, bf0[14], rounding, bit);
  bf1[15] = bf0[15];
  addsub_avx2(
    bf0[16],
    bf0[19],
    &mut bf1[16],
    &mut bf1[19],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[17],
    bf0[18],
    &mut bf1[17],
    &mut bf1[18],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[23],
    bf0[20],
    &mut bf1[23],
    &mut bf1[20],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[22],
    bf0[21],
    &mut bf1[22],
    &mut bf1[21],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[24],
    bf0[27],
    &mut bf1[24],
    &mut bf1[27],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[25],
    bf0[26],
    &mut bf1[25],
    &mut bf1[26],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[31],
    bf0[28],
    &mut bf1[31],
    &mut bf1[28],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[30],
    bf0[29],
    &mut bf1[30],
    &mut bf1[29],
    clamp_lo,
    clamp_hi,
  );

  // stage 6
  addsub_avx2(bf1[0], bf1[3], &mut bf0[0], &mut bf0[3], clamp_lo, clamp_hi);
  addsub_avx2(bf1[1], bf1[2], &mut bf0[1], &mut bf0[2], clamp_lo, clamp_hi);
  bf0[4] = bf1[4];
  bf0[5] = half_btf_avx2(cospim32, bf1[5], cospi32, bf1[6], rounding, bit);
  bf0[6] = half_btf_avx2(cospi32, bf1[5], cospi32, bf1[6], rounding, bit);
  bf0[7] = bf1[7];
  addsub_avx2(bf1[8], bf1[11], &mut bf0[8], &mut bf0[11], clamp_lo, clamp_hi);
  addsub_avx2(bf1[9], bf1[10], &mut bf0[9], &mut bf0[10], clamp_lo, clamp_hi);
  addsub_avx2(
    bf1[15],
    bf1[12],
    &mut bf0[15],
    &mut bf0[12],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1[14],
    bf1[13],
    &mut bf0[14],
    &mut bf0[13],
    clamp_lo,
    clamp_hi,
  );
  bf0[16] = bf1[16];
  bf0[17] = bf1[17];
  bf0[18] = half_btf_avx2(cospim16, bf1[18], cospi48, bf1[29], rounding, bit);
  bf0[19] = half_btf_avx2(cospim16, bf1[19], cospi48, bf1[28], rounding, bit);
  bf0[20] = half_btf_avx2(cospim48, bf1[20], cospim16, bf1[27], rounding, bit);
  bf0[21] = half_btf_avx2(cospim48, bf1[21], cospim16, bf1[26], rounding, bit);
  bf0[22] = bf1[22];
  bf0[23] = bf1[23];
  bf0[24] = bf1[24];
  bf0[25] = bf1[25];
  bf0[26] = half_btf_avx2(cospim16, bf1[21], cospi48, bf1[26], rounding, bit);
  bf0[27] = half_btf_avx2(cospim16, bf1[20], cospi48, bf1[27], rounding, bit);
  bf0[28] = half_btf_avx2(cospi48, bf1[19], cospi16, bf1[28], rounding, bit);
  bf0[29] = half_btf_avx2(cospi48, bf1[18], cospi16, bf1[29], rounding, bit);
  bf0[30] = bf1[30];
  bf0[31] = bf1[31];

  // stage 7
  addsub_avx2(bf0[0], bf0[7], &mut bf1[0], &mut bf1[7], clamp_lo, clamp_hi);
  addsub_avx2(bf0[1], bf0[6], &mut bf1[1], &mut bf1[6], clamp_lo, clamp_hi);
  addsub_avx2(bf0[2], bf0[5], &mut bf1[2], &mut bf1[5], clamp_lo, clamp_hi);
  addsub_avx2(bf0[3], bf0[4], &mut bf1[3], &mut bf1[4], clamp_lo, clamp_hi);
  bf1[8] = bf0[8];
  bf1[9] = bf0[9];
  bf1[10] = half_btf_avx2(cospim32, bf0[10], cospi32, bf0[13], rounding, bit);
  bf1[11] = half_btf_avx2(cospim32, bf0[11], cospi32, bf0[12], rounding, bit);
  bf1[12] = half_btf_avx2(cospi32, bf0[11], cospi32, bf0[12], rounding, bit);
  bf1[13] = half_btf_avx2(cospi32, bf0[10], cospi32, bf0[13], rounding, bit);
  bf1[14] = bf0[14];
  bf1[15] = bf0[15];
  addsub_avx2(
    bf0[16],
    bf0[23],
    &mut bf1[16],
    &mut bf1[23],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[17],
    bf0[22],
    &mut bf1[17],
    &mut bf1[22],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[18],
    bf0[21],
    &mut bf1[18],
    &mut bf1[21],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[19],
    bf0[20],
    &mut bf1[19],
    &mut bf1[20],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[31],
    bf0[24],
    &mut bf1[31],
    &mut bf1[24],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[30],
    bf0[25],
    &mut bf1[30],
    &mut bf1[25],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[29],
    bf0[26],
    &mut bf1[29],
    &mut bf1[26],
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[28],
    bf0[27],
    &mut bf1[28],
    &mut bf1[27],
    clamp_lo,
    clamp_hi,
  );

  // stage 8
  addsub_avx2(bf1[0], bf1[15], &mut bf0[0], &mut bf0[15], clamp_lo, clamp_hi);
  addsub_avx2(bf1[1], bf1[14], &mut bf0[1], &mut bf0[14], clamp_lo, clamp_hi);
  addsub_avx2(bf1[2], bf1[13], &mut bf0[2], &mut bf0[13], clamp_lo, clamp_hi);
  addsub_avx2(bf1[3], bf1[12], &mut bf0[3], &mut bf0[12], clamp_lo, clamp_hi);
  addsub_avx2(bf1[4], bf1[11], &mut bf0[4], &mut bf0[11], clamp_lo, clamp_hi);
  addsub_avx2(bf1[5], bf1[10], &mut bf0[5], &mut bf0[10], clamp_lo, clamp_hi);
  addsub_avx2(bf1[6], bf1[9], &mut bf0[6], &mut bf0[9], clamp_lo, clamp_hi);
  addsub_avx2(bf1[7], bf1[8], &mut bf0[7], &mut bf0[8], clamp_lo, clamp_hi);
  bf0[16] = bf1[16];
  bf0[17] = bf1[17];
  bf0[18] = bf1[18];
  bf0[19] = bf1[19];
  bf0[20] = half_btf_avx2(cospim32, bf1[20], cospi32, bf1[27], rounding, bit);
  bf0[21] = half_btf_avx2(cospim32, bf1[21], cospi32, bf1[26], rounding, bit);
  bf0[22] = half_btf_avx2(cospim32, bf1[22], cospi32, bf1[25], rounding, bit);
  bf0[23] = half_btf_avx2(cospim32, bf1[23], cospi32, bf1[24], rounding, bit);
  bf0[24] = half_btf_avx2(cospi32, bf1[23], cospi32, bf1[24], rounding, bit);
  bf0[25] = half_btf_avx2(cospi32, bf1[22], cospi32, bf1[25], rounding, bit);
  bf0[26] = half_btf_avx2(cospi32, bf1[21], cospi32, bf1[26], rounding, bit);
  bf0[27] = half_btf_avx2(cospi32, bf1[20], cospi32, bf1[27], rounding, bit);
  bf0[28] = bf1[28];
  bf0[29] = bf1[29];
  bf0[30] = bf1[30];
  bf0[31] = bf1[31];

  // stage 9
  addsub_avx2(
    bf0[0],
    bf0[31],
    output.add(0).as_mut().unwrap(),
    output.add(31).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[1],
    bf0[30],
    output.add(1).as_mut().unwrap(),
    output.add(30).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[2],
    bf0[29],
    output.add(2).as_mut().unwrap(),
    output.add(29).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[3],
    bf0[28],
    output.add(3).as_mut().unwrap(),
    output.add(28).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[4],
    bf0[27],
    output.add(4).as_mut().unwrap(),
    output.add(27).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[5],
    bf0[26],
    output.add(5).as_mut().unwrap(),
    output.add(26).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[6],
    bf0[25],
    output.add(6).as_mut().unwrap(),
    output.add(25).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[7],
    bf0[24],
    output.add(7).as_mut().unwrap(),
    output.add(24).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[8],
    bf0[23],
    output.add(8).as_mut().unwrap(),
    output.add(23).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[9],
    bf0[22],
    output.add(9).as_mut().unwrap(),
    output.add(22).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[10],
    bf0[21],
    output.add(10).as_mut().unwrap(),
    output.add(21).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[11],
    bf0[20],
    output.add(11).as_mut().unwrap(),
    output.add(20).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[12],
    bf0[19],
    output.add(12).as_mut().unwrap(),
    output.add(19).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[13],
    bf0[18],
    output.add(13).as_mut().unwrap(),
    output.add(18).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[14],
    bf0[17],
    output.add(14).as_mut().unwrap(),
    output.add(17).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf0[15],
    bf0[16],
    output.add(15).as_mut().unwrap(),
    output.add(16).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);
    round_shift_8x8_avx2(output, out_shift);
    round_shift_8x8_avx2(output.add(16), out_shift);
    highbd_clamp_epi32_avx2(output, output, clamp_lo_out, clamp_hi_out, 32);
  }
}

#[inline(always)]
unsafe fn idct32_stage4_avx2(
  bf1: *mut __m256i, cospim8: __m256i, cospi56: __m256i, cospi8: __m256i,
  cospim56: __m256i, cospim40: __m256i, cospi24: __m256i, cospi40: __m256i,
  cospim24: __m256i, rounding: __m256i, bit: i32,
) {
  let mut temp1;
  let mut temp2;
  temp1 = half_btf_avx2(
    cospim8,
    bf1.add(17).read(),
    cospi56,
    bf1.add(30).read(),
    rounding,
    bit,
  );
  bf1.add(30).write(half_btf_avx2(
    cospi56,
    bf1.add(17).read(),
    cospi8,
    bf1.add(30).read(),
    rounding,
    bit,
  ));
  bf1.add(17).write(temp1);

  temp2 = half_btf_avx2(
    cospim56,
    bf1.add(18).read(),
    cospim8,
    bf1.add(29).read(),
    rounding,
    bit,
  );
  bf1.add(29).write(half_btf_avx2(
    cospim8,
    bf1.add(18).read(),
    cospi56,
    bf1.add(29).read(),
    rounding,
    bit,
  ));
  bf1.add(18).write(temp2);

  temp1 = half_btf_avx2(
    cospim40,
    bf1.add(21).read(),
    cospi24,
    bf1.add(26).read(),
    rounding,
    bit,
  );
  bf1.add(26).write(half_btf_avx2(
    cospi24,
    bf1.add(21).read(),
    cospi40,
    bf1.add(26).read(),
    rounding,
    bit,
  ));
  bf1.add(21).write(temp1);

  temp2 = half_btf_avx2(
    cospim24,
    bf1.add(22).read(),
    cospim40,
    bf1.add(25).read(),
    rounding,
    bit,
  );
  bf1.add(25).write(half_btf_avx2(
    cospim40,
    bf1.add(22).read(),
    cospi24,
    bf1.add(25).read(),
    rounding,
    bit,
  ));
  bf1.add(22).write(temp2);
}

#[inline(always)]
unsafe fn idct32_stage5_avx2(
  bf1: *mut __m256i, cospim16: __m256i, cospi48: __m256i, cospi16: __m256i,
  cospim48: __m256i, clamp_lo: __m256i, clamp_hi: __m256i, rounding: __m256i,
  bit: i32,
) {
  let temp1 = half_btf_avx2(
    cospim16,
    bf1.add(9).read(),
    cospi48,
    bf1.add(14).read(),
    rounding,
    bit,
  );
  bf1.add(14).write(half_btf_avx2(
    cospi48,
    bf1.add(9).read(),
    cospi16,
    bf1.add(14).read(),
    rounding,
    bit,
  ));
  bf1.add(9).write(temp1);

  let temp2 = half_btf_avx2(
    cospim48,
    bf1.add(10).read(),
    cospim16,
    bf1.add(13).read(),
    rounding,
    bit,
  );
  bf1.add(13).write(half_btf_avx2(
    cospim16,
    bf1.add(10).read(),
    cospi48,
    bf1.add(13).read(),
    rounding,
    bit,
  ));
  bf1.add(10).write(temp2);

  addsub_avx2(
    bf1.add(16).read(),
    bf1.add(19).read(),
    bf1.add(16).as_mut().unwrap(),
    bf1.add(19).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(17).read(),
    bf1.add(18).read(),
    bf1.add(17).as_mut().unwrap(),
    bf1.add(18).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(23).read(),
    bf1.add(20).read(),
    bf1.add(23).as_mut().unwrap(),
    bf1.add(20).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(22).read(),
    bf1.add(21).read(),
    bf1.add(22).as_mut().unwrap(),
    bf1.add(21).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(24).read(),
    bf1.add(27).read(),
    bf1.add(24).as_mut().unwrap(),
    bf1.add(27).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(25).read(),
    bf1.add(26).read(),
    bf1.add(25).as_mut().unwrap(),
    bf1.add(26).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(31).read(),
    bf1.add(28).read(),
    bf1.add(31).as_mut().unwrap(),
    bf1.add(28).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(30).read(),
    bf1.add(29).read(),
    bf1.add(30).as_mut().unwrap(),
    bf1.add(29).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
}

#[inline(always)]
unsafe fn idct32_stage6_avx2(
  bf1: *mut __m256i, cospim32: __m256i, cospi32: __m256i, cospim16: __m256i,
  cospi48: __m256i, cospi16: __m256i, cospim48: __m256i, clamp_lo: __m256i,
  clamp_hi: __m256i, rounding: __m256i, bit: i32,
) {
  let mut temp1;
  let mut temp2;
  temp1 = half_btf_avx2(
    cospim32,
    bf1.add(5).read(),
    cospi32,
    bf1.add(6).read(),
    rounding,
    bit,
  );
  bf1.add(6).write(half_btf_avx2(
    cospi32,
    bf1.add(5).read(),
    cospi32,
    bf1.add(6).read(),
    rounding,
    bit,
  ));
  bf1.add(5).write(temp1);

  addsub_avx2(
    bf1.add(8).read(),
    bf1.add(11).read(),
    bf1.add(8).as_mut().unwrap(),
    bf1.add(11).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(9).read(),
    bf1.add(10).read(),
    bf1.add(9).as_mut().unwrap(),
    bf1.add(10).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(15).read(),
    bf1.add(12).read(),
    bf1.add(15).as_mut().unwrap(),
    bf1.add(12).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(14).read(),
    bf1.add(13).read(),
    bf1.add(14).as_mut().unwrap(),
    bf1.add(13).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );

  temp1 = half_btf_avx2(
    cospim16,
    bf1.add(18).read(),
    cospi48,
    bf1.add(29).read(),
    rounding,
    bit,
  );
  bf1.add(29).write(half_btf_avx2(
    cospi48,
    bf1.add(18).read(),
    cospi16,
    bf1.add(29).read(),
    rounding,
    bit,
  ));
  bf1.add(18).write(temp1);
  temp2 = half_btf_avx2(
    cospim16,
    bf1.add(19).read(),
    cospi48,
    bf1.add(28).read(),
    rounding,
    bit,
  );
  bf1.add(28).write(half_btf_avx2(
    cospi48,
    bf1.add(19).read(),
    cospi16,
    bf1.add(28).read(),
    rounding,
    bit,
  ));
  bf1.add(19).write(temp2);
  temp1 = half_btf_avx2(
    cospim48,
    bf1.add(20).read(),
    cospim16,
    bf1.add(27).read(),
    rounding,
    bit,
  );
  bf1.add(27).write(half_btf_avx2(
    cospim16,
    bf1.add(20).read(),
    cospi48,
    bf1.add(27).read(),
    rounding,
    bit,
  ));
  bf1.add(20).write(temp1);
  temp2 = half_btf_avx2(
    cospim48,
    bf1.add(21).read(),
    cospim16,
    bf1.add(26).read(),
    rounding,
    bit,
  );
  bf1.add(26).write(half_btf_avx2(
    cospim16,
    bf1.add(21).read(),
    cospi48,
    bf1.add(26).read(),
    rounding,
    bit,
  ));
  bf1.add(21).write(temp2);
}

#[inline(always)]
unsafe fn idct32_stage7_avx2(
  bf1: *mut __m256i, cospim32: __m256i, cospi32: __m256i, clamp_lo: __m256i,
  clamp_hi: __m256i, rounding: __m256i, bit: i32,
) {
  addsub_avx2(
    bf1.add(0).read(),
    bf1.add(7).read(),
    bf1.add(0).as_mut().unwrap(),
    bf1.add(7).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(1).read(),
    bf1.add(6).read(),
    bf1.add(1).as_mut().unwrap(),
    bf1.add(6).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(2).read(),
    bf1.add(5).read(),
    bf1.add(2).as_mut().unwrap(),
    bf1.add(5).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(3).read(),
    bf1.add(4).read(),
    bf1.add(3).as_mut().unwrap(),
    bf1.add(4).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );

  let temp1 = half_btf_avx2(
    cospim32,
    bf1.add(10).read(),
    cospi32,
    bf1.add(13).read(),
    rounding,
    bit,
  );
  bf1.add(13).write(half_btf_avx2(
    cospi32,
    bf1.add(10).read(),
    cospi32,
    bf1.add(13).read(),
    rounding,
    bit,
  ));
  bf1.add(10).write(temp1);
  let temp2 = half_btf_avx2(
    cospim32,
    bf1.add(11).read(),
    cospi32,
    bf1.add(12).read(),
    rounding,
    bit,
  );
  bf1.add(12).write(half_btf_avx2(
    cospi32,
    bf1.add(11).read(),
    cospi32,
    bf1.add(12).read(),
    rounding,
    bit,
  ));
  bf1.add(11).write(temp2);

  addsub_avx2(
    bf1.add(16).read(),
    bf1.add(23).read(),
    bf1.add(16).as_mut().unwrap(),
    bf1.add(23).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(17).read(),
    bf1.add(22).read(),
    bf1.add(17).as_mut().unwrap(),
    bf1.add(22).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(18).read(),
    bf1.add(21).read(),
    bf1.add(18).as_mut().unwrap(),
    bf1.add(21).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(19).read(),
    bf1.add(20).read(),
    bf1.add(19).as_mut().unwrap(),
    bf1.add(20).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(31).read(),
    bf1.add(24).read(),
    bf1.add(31).as_mut().unwrap(),
    bf1.add(24).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(30).read(),
    bf1.add(25).read(),
    bf1.add(30).as_mut().unwrap(),
    bf1.add(25).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(29).read(),
    bf1.add(26).read(),
    bf1.add(29).as_mut().unwrap(),
    bf1.add(26).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(28).read(),
    bf1.add(27).read(),
    bf1.add(28).as_mut().unwrap(),
    bf1.add(27).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
}

#[inline(always)]
unsafe fn idct32_stage8_avx2(
  bf1: *mut __m256i, cospim32: __m256i, cospi32: __m256i, clamp_lo: __m256i,
  clamp_hi: __m256i, rounding: __m256i, bit: i32,
) {
  let mut temp1;
  let mut temp2;
  addsub_avx2(
    bf1.add(0).read(),
    bf1.add(15).read(),
    bf1.add(0).as_mut().unwrap(),
    bf1.add(15).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(1).read(),
    bf1.add(14).read(),
    bf1.add(1).as_mut().unwrap(),
    bf1.add(14).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(2).read(),
    bf1.add(13).read(),
    bf1.add(2).as_mut().unwrap(),
    bf1.add(13).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(3).read(),
    bf1.add(12).read(),
    bf1.add(3).as_mut().unwrap(),
    bf1.add(12).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(4).read(),
    bf1.add(11).read(),
    bf1.add(4).as_mut().unwrap(),
    bf1.add(11).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(5).read(),
    bf1.add(10).read(),
    bf1.add(5).as_mut().unwrap(),
    bf1.add(10).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(6).read(),
    bf1.add(9).read(),
    bf1.add(6).as_mut().unwrap(),
    bf1.add(9).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(7).read(),
    bf1.add(8).read(),
    bf1.add(7).as_mut().unwrap(),
    bf1.add(8).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );

  temp1 = half_btf_avx2(
    cospim32,
    bf1.add(20).read(),
    cospi32,
    bf1.add(27).read(),
    rounding,
    bit,
  );
  bf1.add(27).write(half_btf_avx2(
    cospi32,
    bf1.add(20).read(),
    cospi32,
    bf1.add(27).read(),
    rounding,
    bit,
  ));
  bf1.add(20).write(temp1);
  temp2 = half_btf_avx2(
    cospim32,
    bf1.add(21).read(),
    cospi32,
    bf1.add(26).read(),
    rounding,
    bit,
  );
  bf1.add(26).write(half_btf_avx2(
    cospi32,
    bf1.add(21).read(),
    cospi32,
    bf1.add(26).read(),
    rounding,
    bit,
  ));
  bf1.add(21).write(temp2);
  temp1 = half_btf_avx2(
    cospim32,
    bf1.add(22).read(),
    cospi32,
    bf1.add(25).read(),
    rounding,
    bit,
  );
  bf1.add(25).write(half_btf_avx2(
    cospi32,
    bf1.add(22).read(),
    cospi32,
    bf1.add(25).read(),
    rounding,
    bit,
  ));
  bf1.add(22).write(temp1);
  temp2 = half_btf_avx2(
    cospim32,
    bf1.add(23).read(),
    cospi32,
    bf1.add(24).read(),
    rounding,
    bit,
  );
  bf1.add(24).write(half_btf_avx2(
    cospi32,
    bf1.add(23).read(),
    cospi32,
    bf1.add(24).read(),
    rounding,
    bit,
  ));
  bf1.add(23).write(temp2);
}

#[inline(always)]
unsafe fn idct32_stage9_avx2(
  bf1: *mut __m256i, out: *mut __m256i, do_cols: bool, bd: usize,
  out_shift: i32, clamp_lo: __m256i, clamp_hi: __m256i,
) {
  addsub_avx2(
    bf1.add(0).read(),
    bf1.add(31).read(),
    out.add(0).as_mut().unwrap(),
    out.add(31).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(1).read(),
    bf1.add(30).read(),
    out.add(1).as_mut().unwrap(),
    out.add(30).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(2).read(),
    bf1.add(29).read(),
    out.add(2).as_mut().unwrap(),
    out.add(29).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(3).read(),
    bf1.add(28).read(),
    out.add(3).as_mut().unwrap(),
    out.add(28).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(4).read(),
    bf1.add(27).read(),
    out.add(4).as_mut().unwrap(),
    out.add(27).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(5).read(),
    bf1.add(26).read(),
    out.add(5).as_mut().unwrap(),
    out.add(26).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(6).read(),
    bf1.add(25).read(),
    out.add(6).as_mut().unwrap(),
    out.add(25).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(7).read(),
    bf1.add(24).read(),
    out.add(7).as_mut().unwrap(),
    out.add(24).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(8).read(),
    bf1.add(23).read(),
    out.add(8).as_mut().unwrap(),
    out.add(23).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(9).read(),
    bf1.add(22).read(),
    out.add(9).as_mut().unwrap(),
    out.add(22).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(10).read(),
    bf1.add(21).read(),
    out.add(10).as_mut().unwrap(),
    out.add(21).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(11).read(),
    bf1.add(20).read(),
    out.add(11).as_mut().unwrap(),
    out.add(20).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(12).read(),
    bf1.add(19).read(),
    out.add(12).as_mut().unwrap(),
    out.add(19).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(13).read(),
    bf1.add(18).read(),
    out.add(13).as_mut().unwrap(),
    out.add(18).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(14).read(),
    bf1.add(17).read(),
    out.add(14).as_mut().unwrap(),
    out.add(17).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  addsub_avx2(
    bf1.add(15).read(),
    bf1.add(16).read(),
    out.add(15).as_mut().unwrap(),
    out.add(16).as_mut().unwrap(),
    clamp_lo,
    clamp_hi,
  );
  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);
    round_shift_8x8_avx2(out, out_shift);
    round_shift_8x8_avx2(out.add(16), out_shift);
    highbd_clamp_epi32_avx2(out, out, clamp_lo_out, clamp_hi_out, 32);
  }
}

#[target_feature(enable = "avx2")]
unsafe fn idct64_low1_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let mut clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let mut clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);

  let cospi32 = _mm256_set1_epi32(cospi[32]);

  // stage 1
  // stage 2
  // stage 3
  // stage 4
  // stage 5
  // stage 6
  let mut x = half_btf_0_avx2(cospi32, input.read(), rnding, bit);

  // stage 8
  // stage 9
  // stage 10
  // stage 11
  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    clamp_lo = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    clamp_hi = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);
    if out_shift != 0 {
      let offset = _mm256_set1_epi32((1 << out_shift) >> 1);
      x = _mm256_add_epi32(x, offset);
      x = _mm256_sra_epi32(x, _mm_cvtsi32_si128(out_shift));
    }
  }
  x = _mm256_max_epi32(x, clamp_lo);
  x = _mm256_min_epi32(x, clamp_hi);
  output.add(0).write(x);
  output.add(1).write(x);
  output.add(2).write(x);
  output.add(3).write(x);
  output.add(4).write(x);
  output.add(5).write(x);
  output.add(6).write(x);
  output.add(7).write(x);
  output.add(8).write(x);
  output.add(9).write(x);
  output.add(10).write(x);
  output.add(11).write(x);
  output.add(12).write(x);
  output.add(13).write(x);
  output.add(14).write(x);
  output.add(15).write(x);
  output.add(16).write(x);
  output.add(17).write(x);
  output.add(18).write(x);
  output.add(19).write(x);
  output.add(20).write(x);
  output.add(21).write(x);
  output.add(22).write(x);
  output.add(23).write(x);
  output.add(24).write(x);
  output.add(25).write(x);
  output.add(26).write(x);
  output.add(27).write(x);
  output.add(28).write(x);
  output.add(29).write(x);
  output.add(30).write(x);
  output.add(31).write(x);
  output.add(32).write(x);
  output.add(33).write(x);
  output.add(34).write(x);
  output.add(35).write(x);
  output.add(36).write(x);
  output.add(37).write(x);
  output.add(38).write(x);
  output.add(39).write(x);
  output.add(40).write(x);
  output.add(41).write(x);
  output.add(42).write(x);
  output.add(43).write(x);
  output.add(44).write(x);
  output.add(45).write(x);
  output.add(46).write(x);
  output.add(47).write(x);
  output.add(48).write(x);
  output.add(49).write(x);
  output.add(50).write(x);
  output.add(51).write(x);
  output.add(52).write(x);
  output.add(53).write(x);
  output.add(54).write(x);
  output.add(55).write(x);
  output.add(56).write(x);
  output.add(57).write(x);
  output.add(58).write(x);
  output.add(59).write(x);
  output.add(60).write(x);
  output.add(61).write(x);
  output.add(62).write(x);
  output.add(63).write(x);
}

#[target_feature(enable = "avx2")]
unsafe fn idct64_low8_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);

  let cospi1 = _mm256_set1_epi32(cospi[1]);
  let cospi2 = _mm256_set1_epi32(cospi[2]);
  let cospi3 = _mm256_set1_epi32(cospi[3]);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospi6 = _mm256_set1_epi32(cospi[6]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospi12 = _mm256_set1_epi32(cospi[12]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospi20 = _mm256_set1_epi32(cospi[20]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospi28 = _mm256_set1_epi32(cospi[28]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi44 = _mm256_set1_epi32(cospi[44]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospim4 = _mm256_set1_epi32(-cospi[4]);
  let cospim8 = _mm256_set1_epi32(-cospi[8]);
  let cospim12 = _mm256_set1_epi32(-cospi[12]);
  let cospim16 = _mm256_set1_epi32(-cospi[16]);
  let cospim20 = _mm256_set1_epi32(-cospi[20]);
  let cospim24 = _mm256_set1_epi32(-cospi[24]);
  let cospim28 = _mm256_set1_epi32(-cospi[28]);
  let cospim32 = _mm256_set1_epi32(-cospi[32]);
  let cospim36 = _mm256_set1_epi32(-cospi[36]);
  let cospim40 = _mm256_set1_epi32(-cospi[40]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospim52 = _mm256_set1_epi32(-cospi[52]);
  let cospim56 = _mm256_set1_epi32(-cospi[56]);
  let cospi63 = _mm256_set1_epi32(cospi[63]);
  let cospim57 = _mm256_set1_epi32(-cospi[57]);
  let cospi7 = _mm256_set1_epi32(cospi[7]);
  let cospi5 = _mm256_set1_epi32(cospi[5]);
  let cospi59 = _mm256_set1_epi32(cospi[59]);
  let cospim61 = _mm256_set1_epi32(-cospi[61]);
  let cospim58 = _mm256_set1_epi32(-cospi[58]);
  let cospi62 = _mm256_set1_epi32(cospi[62]);

  let mut u = [_mm256_setzero_si256(); 64];

  // stage 1
  u[0] = input.add(0).read();
  u[8] = input.add(4).read();
  u[16] = input.add(2).read();
  u[24] = input.add(6).read();
  u[32] = input.add(1).read();
  u[40] = input.add(5).read();
  u[48] = input.add(3).read();
  u[56] = input.add(7).read();

  // stage 2
  u[63] = half_btf_0_avx2(cospi1, u[32], rnding, bit);
  u[32] = half_btf_0_avx2(cospi63, u[32], rnding, bit);
  u[39] = half_btf_0_avx2(cospim57, u[56], rnding, bit);
  u[56] = half_btf_0_avx2(cospi7, u[56], rnding, bit);
  u[55] = half_btf_0_avx2(cospi5, u[40], rnding, bit);
  u[40] = half_btf_0_avx2(cospi59, u[40], rnding, bit);
  u[47] = half_btf_0_avx2(cospim61, u[48], rnding, bit);
  u[48] = half_btf_0_avx2(cospi3, u[48], rnding, bit);

  // stage 3
  u[31] = half_btf_0_avx2(cospi2, u[16], rnding, bit);
  u[16] = half_btf_0_avx2(cospi62, u[16], rnding, bit);
  u[23] = half_btf_0_avx2(cospim58, u[24], rnding, bit);
  u[24] = half_btf_0_avx2(cospi6, u[24], rnding, bit);
  u[33] = u[32];
  u[38] = u[39];
  u[41] = u[40];
  u[46] = u[47];
  u[49] = u[48];
  u[54] = u[55];
  u[57] = u[56];
  u[62] = u[63];

  // stage 4
  let mut temp1;
  let mut temp2;
  u[15] = half_btf_0_avx2(cospi4, u[8], rnding, bit);
  u[8] = half_btf_0_avx2(cospi60, u[8], rnding, bit);
  u[17] = u[16];
  u[22] = u[23];
  u[25] = u[24];
  u[30] = u[31];

  temp1 = half_btf_avx2(cospim4, u[33], cospi60, u[62], rnding, bit);
  u[62] = half_btf_avx2(cospi60, u[33], cospi4, u[62], rnding, bit);
  u[33] = temp1;

  temp2 = half_btf_avx2(cospim36, u[38], cospi28, u[57], rnding, bit);
  u[38] = half_btf_avx2(cospim28, u[38], cospim36, u[57], rnding, bit);
  u[57] = temp2;

  temp1 = half_btf_avx2(cospim20, u[41], cospi44, u[54], rnding, bit);
  u[54] = half_btf_avx2(cospi44, u[41], cospi20, u[54], rnding, bit);
  u[41] = temp1;

  temp2 = half_btf_avx2(cospim12, u[46], cospim52, u[49], rnding, bit);
  u[49] = half_btf_avx2(cospim52, u[46], cospi12, u[49], rnding, bit);
  u[46] = temp2;

  // stage 5
  u[9] = u[8];
  u[14] = u[15];

  temp1 = half_btf_avx2(cospim8, u[17], cospi56, u[30], rnding, bit);
  u[30] = half_btf_avx2(cospi56, u[17], cospi8, u[30], rnding, bit);
  u[17] = temp1;

  temp2 = half_btf_avx2(cospim24, u[22], cospim40, u[25], rnding, bit);
  u[25] = half_btf_avx2(cospim40, u[22], cospi24, u[25], rnding, bit);
  u[22] = temp2;

  u[35] = u[32];
  u[34] = u[33];
  u[36] = u[39];
  u[37] = u[38];
  u[43] = u[40];
  u[42] = u[41];
  u[44] = u[47];
  u[45] = u[46];
  u[51] = u[48];
  u[50] = u[49];
  u[52] = u[55];
  u[53] = u[54];
  u[59] = u[56];
  u[58] = u[57];
  u[60] = u[63];
  u[61] = u[62];

  // stage 6
  temp1 = half_btf_0_avx2(cospi32, u[0], rnding, bit);
  u[1] = half_btf_0_avx2(cospi32, u[0], rnding, bit);
  u[0] = temp1;

  temp2 = half_btf_avx2(cospim16, u[9], cospi48, u[14], rnding, bit);
  u[14] = half_btf_avx2(cospi48, u[9], cospi16, u[14], rnding, bit);
  u[9] = temp2;
  u[19] = u[16];
  u[18] = u[17];
  u[20] = u[23];
  u[21] = u[22];
  u[27] = u[24];
  u[26] = u[25];
  u[28] = u[31];
  u[29] = u[30];

  temp1 = half_btf_avx2(cospim8, u[34], cospi56, u[61], rnding, bit);
  u[61] = half_btf_avx2(cospi56, u[34], cospi8, u[61], rnding, bit);
  u[34] = temp1;
  temp2 = half_btf_avx2(cospim8, u[35], cospi56, u[60], rnding, bit);
  u[60] = half_btf_avx2(cospi56, u[35], cospi8, u[60], rnding, bit);
  u[35] = temp2;
  temp1 = half_btf_avx2(cospim56, u[36], cospim8, u[59], rnding, bit);
  u[59] = half_btf_avx2(cospim8, u[36], cospi56, u[59], rnding, bit);
  u[36] = temp1;
  temp2 = half_btf_avx2(cospim56, u[37], cospim8, u[58], rnding, bit);
  u[58] = half_btf_avx2(cospim8, u[37], cospi56, u[58], rnding, bit);
  u[37] = temp2;
  temp1 = half_btf_avx2(cospim40, u[42], cospi24, u[53], rnding, bit);
  u[53] = half_btf_avx2(cospi24, u[42], cospi40, u[53], rnding, bit);
  u[42] = temp1;
  temp2 = half_btf_avx2(cospim40, u[43], cospi24, u[52], rnding, bit);
  u[52] = half_btf_avx2(cospi24, u[43], cospi40, u[52], rnding, bit);
  u[43] = temp2;
  temp1 = half_btf_avx2(cospim24, u[44], cospim40, u[51], rnding, bit);
  u[51] = half_btf_avx2(cospim40, u[44], cospi24, u[51], rnding, bit);
  u[44] = temp1;
  temp2 = half_btf_avx2(cospim24, u[45], cospim40, u[50], rnding, bit);
  u[50] = half_btf_avx2(cospim40, u[45], cospi24, u[50], rnding, bit);
  u[45] = temp2;

  // stage 7
  u[3] = u[0];
  u[2] = u[1];
  u[11] = u[8];
  u[10] = u[9];
  u[12] = u[15];
  u[13] = u[14];

  temp1 = half_btf_avx2(cospim16, u[18], cospi48, u[29], rnding, bit);
  u[29] = half_btf_avx2(cospi48, u[18], cospi16, u[29], rnding, bit);
  u[18] = temp1;
  temp2 = half_btf_avx2(cospim16, u[19], cospi48, u[28], rnding, bit);
  u[28] = half_btf_avx2(cospi48, u[19], cospi16, u[28], rnding, bit);
  u[19] = temp2;
  temp1 = half_btf_avx2(cospim48, u[20], cospim16, u[27], rnding, bit);
  u[27] = half_btf_avx2(cospim16, u[20], cospi48, u[27], rnding, bit);
  u[20] = temp1;
  temp2 = half_btf_avx2(cospim48, u[21], cospim16, u[26], rnding, bit);
  u[26] = half_btf_avx2(cospim16, u[21], cospi48, u[26], rnding, bit);
  u[21] = temp2;
  for i in (32..64).step_by(16) {
    for j in i..(i + 4) {
      addsub_avx2(
        u[j],
        u[j ^ 7],
        &mut u[j],
        &mut u[j ^ 7],
        clamp_lo,
        clamp_hi,
      );
      addsub_avx2(
        u[j ^ 15],
        u[j ^ 8],
        &mut u[j ^ 15],
        &mut u[j ^ 8],
        clamp_lo,
        clamp_hi,
      );
    }
  }

  // stage 8
  u[7] = u[0];
  u[6] = u[1];
  u[5] = u[2];
  u[4] = u[3];
  u[9] = u[9];

  idct64_stage8_avx2(
    u.as_mut_ptr(),
    cospim32,
    cospi32,
    cospim16,
    cospi48,
    cospi16,
    cospim48,
    clamp_lo,
    clamp_hi,
    rnding,
    bit,
  );

  // stage 9
  idct64_stage9_avx2(
    u.as_mut_ptr(),
    cospim32,
    cospi32,
    clamp_lo,
    clamp_hi,
    rnding,
    bit,
  );

  // stage 10
  idct64_stage10_avx2(
    u.as_mut_ptr(),
    cospim32,
    cospi32,
    clamp_lo,
    clamp_hi,
    rnding,
    bit,
  );

  // stage 11
  idct64_stage11_avx2(
    u.as_mut_ptr(),
    output,
    do_cols,
    bd,
    out_shift,
    clamp_lo,
    clamp_hi,
  );
}

#[target_feature(enable = "avx2")]
unsafe fn idct64_low16_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);

  let cospi1 = _mm256_set1_epi32(cospi[1]);
  let cospi2 = _mm256_set1_epi32(cospi[2]);
  let cospi3 = _mm256_set1_epi32(cospi[3]);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospi5 = _mm256_set1_epi32(cospi[5]);
  let cospi6 = _mm256_set1_epi32(cospi[6]);
  let cospi7 = _mm256_set1_epi32(cospi[7]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospi9 = _mm256_set1_epi32(cospi[9]);
  let cospi10 = _mm256_set1_epi32(cospi[10]);
  let cospi11 = _mm256_set1_epi32(cospi[11]);
  let cospi12 = _mm256_set1_epi32(cospi[12]);
  let cospi13 = _mm256_set1_epi32(cospi[13]);
  let cospi14 = _mm256_set1_epi32(cospi[14]);
  let cospi15 = _mm256_set1_epi32(cospi[15]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospi20 = _mm256_set1_epi32(cospi[20]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospi28 = _mm256_set1_epi32(cospi[28]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let cospi36 = _mm256_set1_epi32(cospi[36]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi44 = _mm256_set1_epi32(cospi[44]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospi51 = _mm256_set1_epi32(cospi[51]);
  let cospi52 = _mm256_set1_epi32(cospi[52]);
  let cospi54 = _mm256_set1_epi32(cospi[54]);
  let cospi55 = _mm256_set1_epi32(cospi[55]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi59 = _mm256_set1_epi32(cospi[59]);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospi62 = _mm256_set1_epi32(cospi[62]);
  let cospi63 = _mm256_set1_epi32(cospi[63]);

  let cospim4 = _mm256_set1_epi32(-cospi[4]);
  let cospim8 = _mm256_set1_epi32(-cospi[8]);
  let cospim12 = _mm256_set1_epi32(-cospi[12]);
  let cospim16 = _mm256_set1_epi32(-cospi[16]);
  let cospim20 = _mm256_set1_epi32(-cospi[20]);
  let cospim24 = _mm256_set1_epi32(-cospi[24]);
  let cospim28 = _mm256_set1_epi32(-cospi[28]);
  let cospim32 = _mm256_set1_epi32(-cospi[32]);
  let cospim36 = _mm256_set1_epi32(-cospi[36]);
  let cospim40 = _mm256_set1_epi32(-cospi[40]);
  let cospim44 = _mm256_set1_epi32(-cospi[44]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospim49 = _mm256_set1_epi32(-cospi[49]);
  let cospim50 = _mm256_set1_epi32(-cospi[50]);
  let cospim52 = _mm256_set1_epi32(-cospi[52]);
  let cospim53 = _mm256_set1_epi32(-cospi[53]);
  let cospim56 = _mm256_set1_epi32(-cospi[56]);
  let cospim57 = _mm256_set1_epi32(-cospi[57]);
  let cospim58 = _mm256_set1_epi32(-cospi[58]);
  let cospim60 = _mm256_set1_epi32(-cospi[60]);
  let cospim61 = _mm256_set1_epi32(-cospi[61]);

  let mut u = [_mm256_setzero_si256(); 64];
  let mut tmp1;
  let mut tmp2;
  let mut tmp3;
  let mut tmp4;

  // stage 1
  u[0] = input.add(0).read();
  u[32] = input.add(1).read();
  u[36] = input.add(9).read();
  u[40] = input.add(5).read();
  u[44] = input.add(13).read();
  u[48] = input.add(3).read();
  u[52] = input.add(11).read();
  u[56] = input.add(7).read();
  u[60] = input.add(15).read();
  u[16] = input.add(2).read();
  u[20] = input.add(10).read();
  u[24] = input.add(6).read();
  u[28] = input.add(14).read();
  u[4] = input.add(8).read();
  u[8] = input.add(4).read();
  u[12] = input.add(12).read();

  // stage 2
  u[63] = half_btf_0_avx2(cospi1, u[32], rnding, bit);
  u[32] = half_btf_0_avx2(cospi63, u[32], rnding, bit);
  u[35] = half_btf_0_avx2(cospim49, u[60], rnding, bit);
  u[60] = half_btf_0_avx2(cospi15, u[60], rnding, bit);
  u[59] = half_btf_0_avx2(cospi9, u[36], rnding, bit);
  u[36] = half_btf_0_avx2(cospi55, u[36], rnding, bit);
  u[39] = half_btf_0_avx2(cospim57, u[56], rnding, bit);
  u[56] = half_btf_0_avx2(cospi7, u[56], rnding, bit);
  u[55] = half_btf_0_avx2(cospi5, u[40], rnding, bit);
  u[40] = half_btf_0_avx2(cospi59, u[40], rnding, bit);
  u[43] = half_btf_0_avx2(cospim53, u[52], rnding, bit);
  u[52] = half_btf_0_avx2(cospi11, u[52], rnding, bit);
  u[47] = half_btf_0_avx2(cospim61, u[48], rnding, bit);
  u[48] = half_btf_0_avx2(cospi3, u[48], rnding, bit);
  u[51] = half_btf_0_avx2(cospi13, u[44], rnding, bit);
  u[44] = half_btf_0_avx2(cospi51, u[44], rnding, bit);

  // stage 3
  u[31] = half_btf_0_avx2(cospi2, u[16], rnding, bit);
  u[16] = half_btf_0_avx2(cospi62, u[16], rnding, bit);
  u[19] = half_btf_0_avx2(cospim50, u[28], rnding, bit);
  u[28] = half_btf_0_avx2(cospi14, u[28], rnding, bit);
  u[27] = half_btf_0_avx2(cospi10, u[20], rnding, bit);
  u[20] = half_btf_0_avx2(cospi54, u[20], rnding, bit);
  u[23] = half_btf_0_avx2(cospim58, u[24], rnding, bit);
  u[24] = half_btf_0_avx2(cospi6, u[24], rnding, bit);
  u[33] = u[32];
  u[34] = u[35];
  u[37] = u[36];
  u[38] = u[39];
  u[41] = u[40];
  u[42] = u[43];
  u[45] = u[44];
  u[46] = u[47];
  u[49] = u[48];
  u[50] = u[51];
  u[53] = u[52];
  u[54] = u[55];
  u[57] = u[56];
  u[58] = u[59];
  u[61] = u[60];
  u[62] = u[63];

  // stage 4
  u[15] = half_btf_0_avx2(cospi4, u[8], rnding, bit);
  u[8] = half_btf_0_avx2(cospi60, u[8], rnding, bit);
  u[11] = half_btf_0_avx2(cospim52, u[12], rnding, bit);
  u[12] = half_btf_0_avx2(cospi12, u[12], rnding, bit);

  u[17] = u[16];
  u[18] = u[19];
  u[21] = u[20];
  u[22] = u[23];
  u[25] = u[24];
  u[26] = u[27];
  u[29] = u[28];
  u[30] = u[31];

  tmp1 = half_btf_avx2(cospim4, u[33], cospi60, u[62], rnding, bit);
  tmp2 = half_btf_avx2(cospim60, u[34], cospim4, u[61], rnding, bit);
  tmp3 = half_btf_avx2(cospim36, u[37], cospi28, u[58], rnding, bit);
  tmp4 = half_btf_avx2(cospim28, u[38], cospim36, u[57], rnding, bit);
  u[57] = half_btf_avx2(cospim36, u[38], cospi28, u[57], rnding, bit);
  u[58] = half_btf_avx2(cospi28, u[37], cospi36, u[58], rnding, bit);
  u[61] = half_btf_avx2(cospim4, u[34], cospi60, u[61], rnding, bit);
  u[62] = half_btf_avx2(cospi60, u[33], cospi4, u[62], rnding, bit);
  u[33] = tmp1;
  u[34] = tmp2;
  u[37] = tmp3;
  u[38] = tmp4;

  tmp1 = half_btf_avx2(cospim20, u[41], cospi44, u[54], rnding, bit);
  tmp2 = half_btf_avx2(cospim44, u[42], cospim20, u[53], rnding, bit);
  tmp3 = half_btf_avx2(cospim52, u[45], cospi12, u[50], rnding, bit);
  tmp4 = half_btf_avx2(cospim12, u[46], cospim52, u[49], rnding, bit);
  u[49] = half_btf_avx2(cospim52, u[46], cospi12, u[49], rnding, bit);
  u[50] = half_btf_avx2(cospi12, u[45], cospi52, u[50], rnding, bit);
  u[53] = half_btf_avx2(cospim20, u[42], cospi44, u[53], rnding, bit);
  u[54] = half_btf_avx2(cospi44, u[41], cospi20, u[54], rnding, bit);
  u[41] = tmp1;
  u[42] = tmp2;
  u[45] = tmp3;
  u[46] = tmp4;

  // stage 5
  u[7] = half_btf_0_avx2(cospi8, u[4], rnding, bit);
  u[4] = half_btf_0_avx2(cospi56, u[4], rnding, bit);

  u[9] = u[8];
  u[10] = u[11];
  u[13] = u[12];
  u[14] = u[15];

  tmp1 = half_btf_avx2(cospim8, u[17], cospi56, u[30], rnding, bit);
  tmp2 = half_btf_avx2(cospim56, u[18], cospim8, u[29], rnding, bit);
  tmp3 = half_btf_avx2(cospim40, u[21], cospi24, u[26], rnding, bit);
  tmp4 = half_btf_avx2(cospim24, u[22], cospim40, u[25], rnding, bit);
  u[25] = half_btf_avx2(cospim40, u[22], cospi24, u[25], rnding, bit);
  u[26] = half_btf_avx2(cospi24, u[21], cospi40, u[26], rnding, bit);
  u[29] = half_btf_avx2(cospim8, u[18], cospi56, u[29], rnding, bit);
  u[30] = half_btf_avx2(cospi56, u[17], cospi8, u[30], rnding, bit);
  u[17] = tmp1;
  u[18] = tmp2;
  u[21] = tmp3;
  u[22] = tmp4;

  for i in (32..64).step_by(8) {
    addsub_avx2(u[i], u[i + 3], &mut u[i], &mut u[i + 3], clamp_lo, clamp_hi);
    addsub_avx2(
      u[i + 1],
      u[i + 2],
      &mut u[i + 1],
      &mut u[i + 2],
      clamp_lo,
      clamp_hi,
    );

    addsub_avx2(
      u[i + 7],
      u[i + 4],
      &mut u[i + 7],
      &mut u[i + 4],
      clamp_lo,
      clamp_hi,
    );
    addsub_avx2(
      u[i + 6],
      u[i + 5],
      &mut u[i + 6],
      &mut u[i + 5],
      clamp_lo,
      clamp_hi,
    );
  }

  // stage 6
  tmp1 = half_btf_0_avx2(cospi32, u[0], rnding, bit);
  u[1] = half_btf_0_avx2(cospi32, u[0], rnding, bit);
  u[0] = tmp1;
  u[5] = u[4];
  u[6] = u[7];

  tmp1 = half_btf_avx2(cospim16, u[9], cospi48, u[14], rnding, bit);
  u[14] = half_btf_avx2(cospi48, u[9], cospi16, u[14], rnding, bit);
  u[9] = tmp1;
  tmp2 = half_btf_avx2(cospim48, u[10], cospim16, u[13], rnding, bit);
  u[13] = half_btf_avx2(cospim16, u[10], cospi48, u[13], rnding, bit);
  u[10] = tmp2;

  for i in (16..32).step_by(8) {
    addsub_avx2(u[i], u[i + 3], &mut u[i], &mut u[i + 3], clamp_lo, clamp_hi);
    addsub_avx2(
      u[i + 1],
      u[i + 2],
      &mut u[i + 1],
      &mut u[i + 2],
      clamp_lo,
      clamp_hi,
    );

    addsub_avx2(
      u[i + 7],
      u[i + 4],
      &mut u[i + 7],
      &mut u[i + 4],
      clamp_lo,
      clamp_hi,
    );
    addsub_avx2(
      u[i + 6],
      u[i + 5],
      &mut u[i + 6],
      &mut u[i + 5],
      clamp_lo,
      clamp_hi,
    );
  }

  tmp1 = half_btf_avx2(cospim8, u[34], cospi56, u[61], rnding, bit);
  tmp2 = half_btf_avx2(cospim8, u[35], cospi56, u[60], rnding, bit);
  tmp3 = half_btf_avx2(cospim56, u[36], cospim8, u[59], rnding, bit);
  tmp4 = half_btf_avx2(cospim56, u[37], cospim8, u[58], rnding, bit);
  u[58] = half_btf_avx2(cospim8, u[37], cospi56, u[58], rnding, bit);
  u[59] = half_btf_avx2(cospim8, u[36], cospi56, u[59], rnding, bit);
  u[60] = half_btf_avx2(cospi56, u[35], cospi8, u[60], rnding, bit);
  u[61] = half_btf_avx2(cospi56, u[34], cospi8, u[61], rnding, bit);
  u[34] = tmp1;
  u[35] = tmp2;
  u[36] = tmp3;
  u[37] = tmp4;

  tmp1 = half_btf_avx2(cospim40, u[42], cospi24, u[53], rnding, bit);
  tmp2 = half_btf_avx2(cospim40, u[43], cospi24, u[52], rnding, bit);
  tmp3 = half_btf_avx2(cospim24, u[44], cospim40, u[51], rnding, bit);
  tmp4 = half_btf_avx2(cospim24, u[45], cospim40, u[50], rnding, bit);
  u[50] = half_btf_avx2(cospim40, u[45], cospi24, u[50], rnding, bit);
  u[51] = half_btf_avx2(cospim40, u[44], cospi24, u[51], rnding, bit);
  u[52] = half_btf_avx2(cospi24, u[43], cospi40, u[52], rnding, bit);
  u[53] = half_btf_avx2(cospi24, u[42], cospi40, u[53], rnding, bit);
  u[42] = tmp1;
  u[43] = tmp2;
  u[44] = tmp3;
  u[45] = tmp4;

  // stage 7
  u[3] = u[0];
  u[2] = u[1];
  tmp1 = half_btf_avx2(cospim32, u[5], cospi32, u[6], rnding, bit);
  u[6] = half_btf_avx2(cospi32, u[5], cospi32, u[6], rnding, bit);
  u[5] = tmp1;
  addsub_avx2(u[8], u[11], &mut u[8], &mut u[11], clamp_lo, clamp_hi);
  addsub_avx2(u[9], u[10], &mut u[9], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(u[15], u[12], &mut u[15], &mut u[12], clamp_lo, clamp_hi);
  addsub_avx2(u[14], u[13], &mut u[14], &mut u[13], clamp_lo, clamp_hi);

  tmp1 = half_btf_avx2(cospim16, u[18], cospi48, u[29], rnding, bit);
  tmp2 = half_btf_avx2(cospim16, u[19], cospi48, u[28], rnding, bit);
  tmp3 = half_btf_avx2(cospim48, u[20], cospim16, u[27], rnding, bit);
  tmp4 = half_btf_avx2(cospim48, u[21], cospim16, u[26], rnding, bit);
  u[26] = half_btf_avx2(cospim16, u[21], cospi48, u[26], rnding, bit);
  u[27] = half_btf_avx2(cospim16, u[20], cospi48, u[27], rnding, bit);
  u[28] = half_btf_avx2(cospi48, u[19], cospi16, u[28], rnding, bit);
  u[29] = half_btf_avx2(cospi48, u[18], cospi16, u[29], rnding, bit);
  u[18] = tmp1;
  u[19] = tmp2;
  u[20] = tmp3;
  u[21] = tmp4;

  for i in (32..64).step_by(16) {
    for j in i..(i + 4) {
      addsub_avx2(
        u[j],
        u[j ^ 7],
        &mut u[j],
        &mut u[j ^ 7],
        clamp_lo,
        clamp_hi,
      );
      addsub_avx2(
        u[j ^ 15],
        u[j ^ 8],
        &mut u[j ^ 15],
        &mut u[j ^ 8],
        clamp_lo,
        clamp_hi,
      );
    }
  }

  // stage 8
  for i in 0..4 {
    addsub_avx2(u[i], u[7 - i], &mut u[i], &mut u[7 - i], clamp_lo, clamp_hi);
  }

  idct64_stage8_avx2(
    u.as_mut_ptr(),
    cospim32,
    cospi32,
    cospim16,
    cospi48,
    cospi16,
    cospim48,
    clamp_lo,
    clamp_hi,
    rnding,
    bit,
  );

  // stage 9
  idct64_stage9_avx2(
    u.as_mut_ptr(),
    cospim32,
    cospi32,
    clamp_lo,
    clamp_hi,
    rnding,
    bit,
  );

  // stage 10
  idct64_stage10_avx2(
    u.as_mut_ptr(),
    cospim32,
    cospi32,
    clamp_lo,
    clamp_hi,
    rnding,
    bit,
  );

  // stage 11
  idct64_stage11_avx2(
    u.as_mut_ptr(),
    output,
    do_cols,
    bd,
    out_shift,
    clamp_lo,
    clamp_hi,
  );
}

#[target_feature(enable = "avx2")]
unsafe fn idct64_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);

  let cospi1 = _mm256_set1_epi32(cospi[1]);
  let cospi2 = _mm256_set1_epi32(cospi[2]);
  let cospi3 = _mm256_set1_epi32(cospi[3]);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospi5 = _mm256_set1_epi32(cospi[5]);
  let cospi6 = _mm256_set1_epi32(cospi[6]);
  let cospi7 = _mm256_set1_epi32(cospi[7]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospi9 = _mm256_set1_epi32(cospi[9]);
  let cospi10 = _mm256_set1_epi32(cospi[10]);
  let cospi11 = _mm256_set1_epi32(cospi[11]);
  let cospi12 = _mm256_set1_epi32(cospi[12]);
  let cospi13 = _mm256_set1_epi32(cospi[13]);
  let cospi14 = _mm256_set1_epi32(cospi[14]);
  let cospi15 = _mm256_set1_epi32(cospi[15]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospi17 = _mm256_set1_epi32(cospi[17]);
  let cospi18 = _mm256_set1_epi32(cospi[18]);
  let cospi19 = _mm256_set1_epi32(cospi[19]);
  let cospi20 = _mm256_set1_epi32(cospi[20]);
  let cospi21 = _mm256_set1_epi32(cospi[21]);
  let cospi22 = _mm256_set1_epi32(cospi[22]);
  let cospi23 = _mm256_set1_epi32(cospi[23]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospi25 = _mm256_set1_epi32(cospi[25]);
  let cospi26 = _mm256_set1_epi32(cospi[26]);
  let cospi27 = _mm256_set1_epi32(cospi[27]);
  let cospi28 = _mm256_set1_epi32(cospi[28]);
  let cospi29 = _mm256_set1_epi32(cospi[29]);
  let cospi30 = _mm256_set1_epi32(cospi[30]);
  let cospi31 = _mm256_set1_epi32(cospi[31]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let cospi35 = _mm256_set1_epi32(cospi[35]);
  let cospi36 = _mm256_set1_epi32(cospi[36]);
  let cospi38 = _mm256_set1_epi32(cospi[38]);
  let cospi39 = _mm256_set1_epi32(cospi[39]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi43 = _mm256_set1_epi32(cospi[43]);
  let cospi44 = _mm256_set1_epi32(cospi[44]);
  let cospi46 = _mm256_set1_epi32(cospi[46]);
  let cospi47 = _mm256_set1_epi32(cospi[47]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospi51 = _mm256_set1_epi32(cospi[51]);
  let cospi52 = _mm256_set1_epi32(cospi[52]);
  let cospi54 = _mm256_set1_epi32(cospi[54]);
  let cospi55 = _mm256_set1_epi32(cospi[55]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi59 = _mm256_set1_epi32(cospi[59]);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospi62 = _mm256_set1_epi32(cospi[62]);
  let cospi63 = _mm256_set1_epi32(cospi[63]);

  let cospim4 = _mm256_set1_epi32(-cospi[4]);
  let cospim8 = _mm256_set1_epi32(-cospi[8]);
  let cospim12 = _mm256_set1_epi32(-cospi[12]);
  let cospim16 = _mm256_set1_epi32(-cospi[16]);
  let cospim20 = _mm256_set1_epi32(-cospi[20]);
  let cospim24 = _mm256_set1_epi32(-cospi[24]);
  let cospim28 = _mm256_set1_epi32(-cospi[28]);
  let cospim32 = _mm256_set1_epi32(-cospi[32]);
  let cospim33 = _mm256_set1_epi32(-cospi[33]);
  let cospim34 = _mm256_set1_epi32(-cospi[34]);
  let cospim36 = _mm256_set1_epi32(-cospi[36]);
  let cospim37 = _mm256_set1_epi32(-cospi[37]);
  let cospim40 = _mm256_set1_epi32(-cospi[40]);
  let cospim41 = _mm256_set1_epi32(-cospi[41]);
  let cospim42 = _mm256_set1_epi32(-cospi[42]);
  let cospim44 = _mm256_set1_epi32(-cospi[44]);
  let cospim45 = _mm256_set1_epi32(-cospi[45]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospim49 = _mm256_set1_epi32(-cospi[49]);
  let cospim50 = _mm256_set1_epi32(-cospi[50]);
  let cospim52 = _mm256_set1_epi32(-cospi[52]);
  let cospim53 = _mm256_set1_epi32(-cospi[53]);
  let cospim56 = _mm256_set1_epi32(-cospi[56]);
  let cospim57 = _mm256_set1_epi32(-cospi[57]);
  let cospim58 = _mm256_set1_epi32(-cospi[58]);
  let cospim60 = _mm256_set1_epi32(-cospi[60]);
  let cospim61 = _mm256_set1_epi32(-cospi[61]);

  let mut u = [_mm256_setzero_si256(); 64];
  let mut v = [_mm256_setzero_si256(); 64];

  // stage 1
  u[32] = input.add(1).read();
  u[34] = input.add(17).read();
  u[36] = input.add(9).read();
  u[38] = input.add(25).read();
  u[40] = input.add(5).read();
  u[42] = input.add(21).read();
  u[44] = input.add(13).read();
  u[46] = input.add(29).read();
  u[48] = input.add(3).read();
  u[50] = input.add(19).read();
  u[52] = input.add(11).read();
  u[54] = input.add(27).read();
  u[56] = input.add(7).read();
  u[58] = input.add(23).read();
  u[60] = input.add(15).read();
  u[62] = input.add(31).read();

  v[16] = input.add(2).read();
  v[18] = input.add(18).read();
  v[20] = input.add(10).read();
  v[22] = input.add(26).read();
  v[24] = input.add(6).read();
  v[26] = input.add(22).read();
  v[28] = input.add(14).read();
  v[30] = input.add(30).read();

  u[8] = input.add(4).read();
  u[10] = input.add(20).read();
  u[12] = input.add(12).read();
  u[14] = input.add(28).read();

  v[4] = input.add(8).read();
  v[6] = input.add(24).read();

  u[0] = input.add(0).read();
  u[2] = input.add(16).read();

  // stage 2
  v[32] = half_btf_0_avx2(cospi63, u[32], rnding, bit);
  v[33] = half_btf_0_avx2(cospim33, u[62], rnding, bit);
  v[34] = half_btf_0_avx2(cospi47, u[34], rnding, bit);
  v[35] = half_btf_0_avx2(cospim49, u[60], rnding, bit);
  v[36] = half_btf_0_avx2(cospi55, u[36], rnding, bit);
  v[37] = half_btf_0_avx2(cospim41, u[58], rnding, bit);
  v[38] = half_btf_0_avx2(cospi39, u[38], rnding, bit);
  v[39] = half_btf_0_avx2(cospim57, u[56], rnding, bit);
  v[40] = half_btf_0_avx2(cospi59, u[40], rnding, bit);
  v[41] = half_btf_0_avx2(cospim37, u[54], rnding, bit);
  v[42] = half_btf_0_avx2(cospi43, u[42], rnding, bit);
  v[43] = half_btf_0_avx2(cospim53, u[52], rnding, bit);
  v[44] = half_btf_0_avx2(cospi51, u[44], rnding, bit);
  v[45] = half_btf_0_avx2(cospim45, u[50], rnding, bit);
  v[46] = half_btf_0_avx2(cospi35, u[46], rnding, bit);
  v[47] = half_btf_0_avx2(cospim61, u[48], rnding, bit);
  v[48] = half_btf_0_avx2(cospi3, u[48], rnding, bit);
  v[49] = half_btf_0_avx2(cospi29, u[46], rnding, bit);
  v[50] = half_btf_0_avx2(cospi19, u[50], rnding, bit);
  v[51] = half_btf_0_avx2(cospi13, u[44], rnding, bit);
  v[52] = half_btf_0_avx2(cospi11, u[52], rnding, bit);
  v[53] = half_btf_0_avx2(cospi21, u[42], rnding, bit);
  v[54] = half_btf_0_avx2(cospi27, u[54], rnding, bit);
  v[55] = half_btf_0_avx2(cospi5, u[40], rnding, bit);
  v[56] = half_btf_0_avx2(cospi7, u[56], rnding, bit);
  v[57] = half_btf_0_avx2(cospi25, u[38], rnding, bit);
  v[58] = half_btf_0_avx2(cospi23, u[58], rnding, bit);
  v[59] = half_btf_0_avx2(cospi9, u[36], rnding, bit);
  v[60] = half_btf_0_avx2(cospi15, u[60], rnding, bit);
  v[61] = half_btf_0_avx2(cospi17, u[34], rnding, bit);
  v[62] = half_btf_0_avx2(cospi31, u[62], rnding, bit);
  v[63] = half_btf_0_avx2(cospi1, u[32], rnding, bit);

  // stage 3
  u[16] = half_btf_0_avx2(cospi62, v[16], rnding, bit);
  u[17] = half_btf_0_avx2(cospim34, v[30], rnding, bit);
  u[18] = half_btf_0_avx2(cospi46, v[18], rnding, bit);
  u[19] = half_btf_0_avx2(cospim50, v[28], rnding, bit);
  u[20] = half_btf_0_avx2(cospi54, v[20], rnding, bit);
  u[21] = half_btf_0_avx2(cospim42, v[26], rnding, bit);
  u[22] = half_btf_0_avx2(cospi38, v[22], rnding, bit);
  u[23] = half_btf_0_avx2(cospim58, v[24], rnding, bit);
  u[24] = half_btf_0_avx2(cospi6, v[24], rnding, bit);
  u[25] = half_btf_0_avx2(cospi26, v[22], rnding, bit);
  u[26] = half_btf_0_avx2(cospi22, v[26], rnding, bit);
  u[27] = half_btf_0_avx2(cospi10, v[20], rnding, bit);
  u[28] = half_btf_0_avx2(cospi14, v[28], rnding, bit);
  u[29] = half_btf_0_avx2(cospi18, v[18], rnding, bit);
  u[30] = half_btf_0_avx2(cospi30, v[30], rnding, bit);
  u[31] = half_btf_0_avx2(cospi2, v[16], rnding, bit);

  for i in (32..64).step_by(4) {
    addsub_avx2(v[i], v[i + 1], &mut u[i], &mut u[i + 1], clamp_lo, clamp_hi);
    addsub_avx2(
      v[i + 3],
      v[i + 2],
      &mut u[i + 3],
      &mut u[i + 2],
      clamp_lo,
      clamp_hi,
    );
  }

  // stage 4
  v[8] = half_btf_0_avx2(cospi60, u[8], rnding, bit);
  v[9] = half_btf_0_avx2(cospim36, u[14], rnding, bit);
  v[10] = half_btf_0_avx2(cospi44, u[10], rnding, bit);
  v[11] = half_btf_0_avx2(cospim52, u[12], rnding, bit);
  v[12] = half_btf_0_avx2(cospi12, u[12], rnding, bit);
  v[13] = half_btf_0_avx2(cospi20, u[10], rnding, bit);
  v[14] = half_btf_0_avx2(cospi28, u[14], rnding, bit);
  v[15] = half_btf_0_avx2(cospi4, u[8], rnding, bit);

  for i in (16..32).step_by(4) {
    addsub_avx2(u[i], u[i + 1], &mut v[i], &mut v[i + 1], clamp_lo, clamp_hi);
    addsub_avx2(
      u[i + 3],
      u[i + 2],
      &mut v[i + 3],
      &mut v[i + 2],
      clamp_lo,
      clamp_hi,
    );
  }

  for i in (32..64).step_by(4) {
    v[i] = u[i];
    v[i + 3] = u[i + 3];
  }

  v[33] = half_btf_avx2(cospim4, u[33], cospi60, u[62], rnding, bit);
  v[34] = half_btf_avx2(cospim60, u[34], cospim4, u[61], rnding, bit);
  v[37] = half_btf_avx2(cospim36, u[37], cospi28, u[58], rnding, bit);
  v[38] = half_btf_avx2(cospim28, u[38], cospim36, u[57], rnding, bit);
  v[41] = half_btf_avx2(cospim20, u[41], cospi44, u[54], rnding, bit);
  v[42] = half_btf_avx2(cospim44, u[42], cospim20, u[53], rnding, bit);
  v[45] = half_btf_avx2(cospim52, u[45], cospi12, u[50], rnding, bit);
  v[46] = half_btf_avx2(cospim12, u[46], cospim52, u[49], rnding, bit);
  v[49] = half_btf_avx2(cospim52, u[46], cospi12, u[49], rnding, bit);
  v[50] = half_btf_avx2(cospi12, u[45], cospi52, u[50], rnding, bit);
  v[53] = half_btf_avx2(cospim20, u[42], cospi44, u[53], rnding, bit);
  v[54] = half_btf_avx2(cospi44, u[41], cospi20, u[54], rnding, bit);
  v[57] = half_btf_avx2(cospim36, u[38], cospi28, u[57], rnding, bit);
  v[58] = half_btf_avx2(cospi28, u[37], cospi36, u[58], rnding, bit);
  v[61] = half_btf_avx2(cospim4, u[34], cospi60, u[61], rnding, bit);
  v[62] = half_btf_avx2(cospi60, u[33], cospi4, u[62], rnding, bit);

  // stage 5
  u[4] = half_btf_0_avx2(cospi56, v[4], rnding, bit);
  u[5] = half_btf_0_avx2(cospim40, v[6], rnding, bit);
  u[6] = half_btf_0_avx2(cospi24, v[6], rnding, bit);
  u[7] = half_btf_0_avx2(cospi8, v[4], rnding, bit);

  for i in (8..16).step_by(4) {
    addsub_avx2(v[i], v[i + 1], &mut u[i], &mut u[i + 1], clamp_lo, clamp_hi);
    addsub_avx2(
      v[i + 3],
      v[i + 2],
      &mut u[i + 3],
      &mut u[i + 2],
      clamp_lo,
      clamp_hi,
    );
  }

  for i in (16..32).step_by(4) {
    u[i] = v[i];
    u[i + 3] = v[i + 3];
  }

  u[17] = half_btf_avx2(cospim8, v[17], cospi56, v[30], rnding, bit);
  u[18] = half_btf_avx2(cospim56, v[18], cospim8, v[29], rnding, bit);
  u[21] = half_btf_avx2(cospim40, v[21], cospi24, v[26], rnding, bit);
  u[22] = half_btf_avx2(cospim24, v[22], cospim40, v[25], rnding, bit);
  u[25] = half_btf_avx2(cospim40, v[22], cospi24, v[25], rnding, bit);
  u[26] = half_btf_avx2(cospi24, v[21], cospi40, v[26], rnding, bit);
  u[29] = half_btf_avx2(cospim8, v[18], cospi56, v[29], rnding, bit);
  u[30] = half_btf_avx2(cospi56, v[17], cospi8, v[30], rnding, bit);

  for i in (32..64).step_by(8) {
    addsub_avx2(v[i], v[i + 3], &mut u[i], &mut u[i + 3], clamp_lo, clamp_hi);
    addsub_avx2(
      v[i + 1],
      v[i + 2],
      &mut u[i + 1],
      &mut u[i + 2],
      clamp_lo,
      clamp_hi,
    );

    addsub_avx2(
      v[i + 7],
      v[i + 4],
      &mut u[i + 7],
      &mut u[i + 4],
      clamp_lo,
      clamp_hi,
    );
    addsub_avx2(
      v[i + 6],
      v[i + 5],
      &mut u[i + 6],
      &mut u[i + 5],
      clamp_lo,
      clamp_hi,
    );
  }

  // stage 6
  v[0] = half_btf_0_avx2(cospi32, u[0], rnding, bit);
  v[1] = half_btf_0_avx2(cospi32, u[0], rnding, bit);
  v[2] = half_btf_0_avx2(cospi48, u[2], rnding, bit);
  v[3] = half_btf_0_avx2(cospi16, u[2], rnding, bit);

  addsub_avx2(u[4], u[5], &mut v[4], &mut v[5], clamp_lo, clamp_hi);
  addsub_avx2(u[7], u[6], &mut v[7], &mut v[6], clamp_lo, clamp_hi);

  for i in (8..16).step_by(4) {
    v[i] = u[i];
    v[i + 3] = u[i + 3];
  }

  v[9] = half_btf_avx2(cospim16, u[9], cospi48, u[14], rnding, bit);
  v[10] = half_btf_avx2(cospim48, u[10], cospim16, u[13], rnding, bit);
  v[13] = half_btf_avx2(cospim16, u[10], cospi48, u[13], rnding, bit);
  v[14] = half_btf_avx2(cospi48, u[9], cospi16, u[14], rnding, bit);

  for i in (16..32).step_by(8) {
    addsub_avx2(u[i], u[i + 3], &mut v[i], &mut v[i + 3], clamp_lo, clamp_hi);
    addsub_avx2(
      u[i + 1],
      u[i + 2],
      &mut v[i + 1],
      &mut v[i + 2],
      clamp_lo,
      clamp_hi,
    );

    addsub_avx2(
      u[i + 7],
      u[i + 4],
      &mut v[i + 7],
      &mut v[i + 4],
      clamp_lo,
      clamp_hi,
    );
    addsub_avx2(
      u[i + 6],
      u[i + 5],
      &mut v[i + 6],
      &mut v[i + 5],
      clamp_lo,
      clamp_hi,
    );
  }

  for i in (32..64).step_by(8) {
    v[i] = u[i];
    v[i + 1] = u[i + 1];
    v[i + 6] = u[i + 6];
    v[i + 7] = u[i + 7];
  }

  v[34] = half_btf_avx2(cospim8, u[34], cospi56, u[61], rnding, bit);
  v[35] = half_btf_avx2(cospim8, u[35], cospi56, u[60], rnding, bit);
  v[36] = half_btf_avx2(cospim56, u[36], cospim8, u[59], rnding, bit);
  v[37] = half_btf_avx2(cospim56, u[37], cospim8, u[58], rnding, bit);
  v[42] = half_btf_avx2(cospim40, u[42], cospi24, u[53], rnding, bit);
  v[43] = half_btf_avx2(cospim40, u[43], cospi24, u[52], rnding, bit);
  v[44] = half_btf_avx2(cospim24, u[44], cospim40, u[51], rnding, bit);
  v[45] = half_btf_avx2(cospim24, u[45], cospim40, u[50], rnding, bit);
  v[50] = half_btf_avx2(cospim40, u[45], cospi24, u[50], rnding, bit);
  v[51] = half_btf_avx2(cospim40, u[44], cospi24, u[51], rnding, bit);
  v[52] = half_btf_avx2(cospi24, u[43], cospi40, u[52], rnding, bit);
  v[53] = half_btf_avx2(cospi24, u[42], cospi40, u[53], rnding, bit);
  v[58] = half_btf_avx2(cospim8, u[37], cospi56, u[58], rnding, bit);
  v[59] = half_btf_avx2(cospim8, u[36], cospi56, u[59], rnding, bit);
  v[60] = half_btf_avx2(cospi56, u[35], cospi8, u[60], rnding, bit);
  v[61] = half_btf_avx2(cospi56, u[34], cospi8, u[61], rnding, bit);

  // stage 7
  addsub_avx2(v[0], v[3], &mut u[0], &mut u[3], clamp_lo, clamp_hi);
  addsub_avx2(v[1], v[2], &mut u[1], &mut u[2], clamp_lo, clamp_hi);

  u[4] = v[4];
  u[7] = v[7];
  u[5] = half_btf_avx2(cospim32, v[5], cospi32, v[6], rnding, bit);
  u[6] = half_btf_avx2(cospi32, v[5], cospi32, v[6], rnding, bit);

  addsub_avx2(v[8], v[11], &mut u[8], &mut u[11], clamp_lo, clamp_hi);
  addsub_avx2(v[9], v[10], &mut u[9], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(v[15], v[12], &mut u[15], &mut u[12], clamp_lo, clamp_hi);
  addsub_avx2(v[14], v[13], &mut u[14], &mut u[13], clamp_lo, clamp_hi);

  for i in (16..32).step_by(8) {
    u[i] = v[i];
    u[i + 1] = v[i + 1];
    u[i + 6] = v[i + 6];
    u[i + 7] = v[i + 7];
  }

  u[18] = half_btf_avx2(cospim16, v[18], cospi48, v[29], rnding, bit);
  u[19] = half_btf_avx2(cospim16, v[19], cospi48, v[28], rnding, bit);
  u[20] = half_btf_avx2(cospim48, v[20], cospim16, v[27], rnding, bit);
  u[21] = half_btf_avx2(cospim48, v[21], cospim16, v[26], rnding, bit);
  u[26] = half_btf_avx2(cospim16, v[21], cospi48, v[26], rnding, bit);
  u[27] = half_btf_avx2(cospim16, v[20], cospi48, v[27], rnding, bit);
  u[28] = half_btf_avx2(cospi48, v[19], cospi16, v[28], rnding, bit);
  u[29] = half_btf_avx2(cospi48, v[18], cospi16, v[29], rnding, bit);

  for i in (32..64).step_by(16) {
    for j in i..(i + 4) {
      addsub_avx2(
        v[j],
        v[j ^ 7],
        &mut u[j],
        &mut u[j ^ 7],
        clamp_lo,
        clamp_hi,
      );
      addsub_avx2(
        v[j ^ 15],
        v[j ^ 8],
        &mut u[j ^ 15],
        &mut u[j ^ 8],
        clamp_lo,
        clamp_hi,
      );
    }
  }

  // stage 8
  for i in 0..4 {
    addsub_avx2(u[i], u[7 - i], &mut v[i], &mut v[7 - i], clamp_lo, clamp_hi);
  }

  v[8] = u[8];
  v[9] = u[9];
  v[14] = u[14];
  v[15] = u[15];

  v[10] = half_btf_avx2(cospim32, u[10], cospi32, u[13], rnding, bit);
  v[11] = half_btf_avx2(cospim32, u[11], cospi32, u[12], rnding, bit);
  v[12] = half_btf_avx2(cospi32, u[11], cospi32, u[12], rnding, bit);
  v[13] = half_btf_avx2(cospi32, u[10], cospi32, u[13], rnding, bit);

  for i in 16..20 {
    addsub_avx2(u[i], u[i ^ 7], &mut v[i], &mut v[i ^ 7], clamp_lo, clamp_hi);
    addsub_avx2(
      u[i ^ 15],
      u[i ^ 8],
      &mut v[i ^ 15],
      &mut v[i ^ 8],
      clamp_lo,
      clamp_hi,
    );
  }

  v[32..36].copy_from_slice(&u[32..36]);
  v[(32 + 12)..(36 + 12)].copy_from_slice(&u[(32 + 12)..(36 + 12)]);
  v[(32 + 16)..(36 + 16)].copy_from_slice(&u[(32 + 16)..(36 + 16)]);
  v[(32 + 28)..(36 + 28)].copy_from_slice(&u[(32 + 28)..(36 + 28)]);

  v[36] = half_btf_avx2(cospim16, u[36], cospi48, u[59], rnding, bit);
  v[37] = half_btf_avx2(cospim16, u[37], cospi48, u[58], rnding, bit);
  v[38] = half_btf_avx2(cospim16, u[38], cospi48, u[57], rnding, bit);
  v[39] = half_btf_avx2(cospim16, u[39], cospi48, u[56], rnding, bit);
  v[40] = half_btf_avx2(cospim48, u[40], cospim16, u[55], rnding, bit);
  v[41] = half_btf_avx2(cospim48, u[41], cospim16, u[54], rnding, bit);
  v[42] = half_btf_avx2(cospim48, u[42], cospim16, u[53], rnding, bit);
  v[43] = half_btf_avx2(cospim48, u[43], cospim16, u[52], rnding, bit);
  v[52] = half_btf_avx2(cospim16, u[43], cospi48, u[52], rnding, bit);
  v[53] = half_btf_avx2(cospim16, u[42], cospi48, u[53], rnding, bit);
  v[54] = half_btf_avx2(cospim16, u[41], cospi48, u[54], rnding, bit);
  v[55] = half_btf_avx2(cospim16, u[40], cospi48, u[55], rnding, bit);
  v[56] = half_btf_avx2(cospi48, u[39], cospi16, u[56], rnding, bit);
  v[57] = half_btf_avx2(cospi48, u[38], cospi16, u[57], rnding, bit);
  v[58] = half_btf_avx2(cospi48, u[37], cospi16, u[58], rnding, bit);
  v[59] = half_btf_avx2(cospi48, u[36], cospi16, u[59], rnding, bit);

  // stage 9
  for i in 0..8 {
    addsub_avx2(
      v[i],
      v[15 - i],
      &mut u[i],
      &mut u[15 - i],
      clamp_lo,
      clamp_hi,
    );
  }

  u[16..20].copy_from_slice(&v[16..20]);
  u[(16 + 12)..(20 + 12)].copy_from_slice(&v[(16 + 12)..(20 + 12)]);

  u[20] = half_btf_avx2(cospim32, v[20], cospi32, v[27], rnding, bit);
  u[21] = half_btf_avx2(cospim32, v[21], cospi32, v[26], rnding, bit);
  u[22] = half_btf_avx2(cospim32, v[22], cospi32, v[25], rnding, bit);
  u[23] = half_btf_avx2(cospim32, v[23], cospi32, v[24], rnding, bit);
  u[24] = half_btf_avx2(cospi32, v[23], cospi32, v[24], rnding, bit);
  u[25] = half_btf_avx2(cospi32, v[22], cospi32, v[25], rnding, bit);
  u[26] = half_btf_avx2(cospi32, v[21], cospi32, v[26], rnding, bit);
  u[27] = half_btf_avx2(cospi32, v[20], cospi32, v[27], rnding, bit);

  for i in 32..40 {
    addsub_avx2(
      v[i],
      v[i ^ 15],
      &mut u[i],
      &mut u[i ^ 15],
      clamp_lo,
      clamp_hi,
    );
  }

  for i in 48..56 {
    addsub_avx2(
      v[i ^ 15],
      v[i],
      &mut u[i ^ 15],
      &mut u[i],
      clamp_lo,
      clamp_hi,
    );
  }

  // stage 10
  for i in 0..16 {
    addsub_avx2(
      u[i],
      u[31 - i],
      &mut v[i],
      &mut v[31 - i],
      clamp_lo,
      clamp_hi,
    );
  }

  v[32..40].copy_from_slice(&u[32..40]);

  v[40] = half_btf_avx2(cospim32, u[40], cospi32, u[55], rnding, bit);
  v[41] = half_btf_avx2(cospim32, u[41], cospi32, u[54], rnding, bit);
  v[42] = half_btf_avx2(cospim32, u[42], cospi32, u[53], rnding, bit);
  v[43] = half_btf_avx2(cospim32, u[43], cospi32, u[52], rnding, bit);
  v[44] = half_btf_avx2(cospim32, u[44], cospi32, u[51], rnding, bit);
  v[45] = half_btf_avx2(cospim32, u[45], cospi32, u[50], rnding, bit);
  v[46] = half_btf_avx2(cospim32, u[46], cospi32, u[49], rnding, bit);
  v[47] = half_btf_avx2(cospim32, u[47], cospi32, u[48], rnding, bit);
  v[48] = half_btf_avx2(cospi32, u[47], cospi32, u[48], rnding, bit);
  v[49] = half_btf_avx2(cospi32, u[46], cospi32, u[49], rnding, bit);
  v[50] = half_btf_avx2(cospi32, u[45], cospi32, u[50], rnding, bit);
  v[51] = half_btf_avx2(cospi32, u[44], cospi32, u[51], rnding, bit);
  v[52] = half_btf_avx2(cospi32, u[43], cospi32, u[52], rnding, bit);
  v[53] = half_btf_avx2(cospi32, u[42], cospi32, u[53], rnding, bit);
  v[54] = half_btf_avx2(cospi32, u[41], cospi32, u[54], rnding, bit);
  v[55] = half_btf_avx2(cospi32, u[40], cospi32, u[55], rnding, bit);

  v[56..64].copy_from_slice(&u[56..64]);

  // stage 11
  for i in 0..32 {
    addsub_avx2(
      v[i],
      v[63 - i],
      output.add(i).as_mut().unwrap(),
      output.add(63 - i).as_mut().unwrap(),
      clamp_lo,
      clamp_hi,
    );
  }
  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);

    round_shift_8x8_avx2(output, out_shift);
    round_shift_8x8_avx2(output.add(16), out_shift);
    round_shift_8x8_avx2(output.add(32), out_shift);
    round_shift_8x8_avx2(output.add(48), out_shift);
    highbd_clamp_epi32_avx2(output, output, clamp_lo_out, clamp_hi_out, 64);
  }
}

#[inline(always)]
unsafe fn idct64_stage8_avx2(
  u: *mut __m256i, cospim32: __m256i, cospi32: __m256i, cospim16: __m256i,
  cospi48: __m256i, cospi16: __m256i, cospim48: __m256i, clamp_lo: __m256i,
  clamp_hi: __m256i, rnding: __m256i, bit: i32,
) {
  let mut temp1;
  let mut temp2;
  let mut temp3;
  let mut temp4;
  temp1 = half_btf_avx2(
    cospim32,
    u.add(10).read(),
    cospi32,
    u.add(13).read(),
    rnding,
    bit,
  );
  u.add(13).write(half_btf_avx2(
    cospi32,
    u.add(10).read(),
    cospi32,
    u.add(13).read(),
    rnding,
    bit,
  ));
  u.add(10).write(temp1);
  temp2 = half_btf_avx2(
    cospim32,
    u.add(11).read(),
    cospi32,
    u.add(12).read(),
    rnding,
    bit,
  );
  u.add(12).write(half_btf_avx2(
    cospi32,
    u.add(11).read(),
    cospi32,
    u.add(12).read(),
    rnding,
    bit,
  ));
  u.add(11).write(temp2);

  for i in 16..20 {
    addsub_avx2(
      u.add(i).read(),
      u.add(i ^ 7).read(),
      u.add(i).as_mut().unwrap(),
      u.add(i ^ 7).as_mut().unwrap(),
      clamp_lo,
      clamp_hi,
    );
    addsub_avx2(
      u.add(i ^ 15).read(),
      u.add(i ^ 8).read(),
      u.add(i ^ 15).as_mut().unwrap(),
      u.add(i ^ 8).as_mut().unwrap(),
      clamp_lo,
      clamp_hi,
    );
  }

  temp1 = half_btf_avx2(
    cospim16,
    u.add(36).read(),
    cospi48,
    u.add(59).read(),
    rnding,
    bit,
  );
  temp2 = half_btf_avx2(
    cospim16,
    u.add(37).read(),
    cospi48,
    u.add(58).read(),
    rnding,
    bit,
  );
  temp3 = half_btf_avx2(
    cospim16,
    u.add(38).read(),
    cospi48,
    u.add(57).read(),
    rnding,
    bit,
  );
  temp4 = half_btf_avx2(
    cospim16,
    u.add(39).read(),
    cospi48,
    u.add(56).read(),
    rnding,
    bit,
  );
  u.add(56).write(half_btf_avx2(
    cospi48,
    u.add(39).read(),
    cospi16,
    u.add(56).read(),
    rnding,
    bit,
  ));
  u.add(57).write(half_btf_avx2(
    cospi48,
    u.add(38).read(),
    cospi16,
    u.add(57).read(),
    rnding,
    bit,
  ));
  u.add(58).write(half_btf_avx2(
    cospi48,
    u.add(37).read(),
    cospi16,
    u.add(58).read(),
    rnding,
    bit,
  ));
  u.add(59).write(half_btf_avx2(
    cospi48,
    u.add(36).read(),
    cospi16,
    u.add(59).read(),
    rnding,
    bit,
  ));
  u.add(36).write(temp1);
  u.add(37).write(temp2);
  u.add(38).write(temp3);
  u.add(39).write(temp4);

  temp1 = half_btf_avx2(
    cospim48,
    u.add(40).read(),
    cospim16,
    u.add(55).read(),
    rnding,
    bit,
  );
  temp2 = half_btf_avx2(
    cospim48,
    u.add(41).read(),
    cospim16,
    u.add(54).read(),
    rnding,
    bit,
  );
  temp3 = half_btf_avx2(
    cospim48,
    u.add(42).read(),
    cospim16,
    u.add(53).read(),
    rnding,
    bit,
  );
  temp4 = half_btf_avx2(
    cospim48,
    u.add(43).read(),
    cospim16,
    u.add(52).read(),
    rnding,
    bit,
  );
  u.add(52).write(half_btf_avx2(
    cospim16,
    u.add(43).read(),
    cospi48,
    u.add(52).read(),
    rnding,
    bit,
  ));
  u.add(53).write(half_btf_avx2(
    cospim16,
    u.add(42).read(),
    cospi48,
    u.add(53).read(),
    rnding,
    bit,
  ));
  u.add(54).write(half_btf_avx2(
    cospim16,
    u.add(41).read(),
    cospi48,
    u.add(54).read(),
    rnding,
    bit,
  ));
  u.add(55).write(half_btf_avx2(
    cospim16,
    u.add(40).read(),
    cospi48,
    u.add(55).read(),
    rnding,
    bit,
  ));
  u.add(40).write(temp1);
  u.add(41).write(temp2);
  u.add(42).write(temp3);
  u.add(43).write(temp4);
}

#[inline(always)]
unsafe fn idct64_stage9_avx2(
  u: *mut __m256i, cospim32: __m256i, cospi32: __m256i, clamp_lo: __m256i,
  clamp_hi: __m256i, rnding: __m256i, bit: i32,
) {
  for i in 0..8 {
    addsub_avx2(
      u.add(i).read(),
      u.add(15 - i).read(),
      u.add(i).as_mut().unwrap(),
      u.add(15 - i).as_mut().unwrap(),
      clamp_lo,
      clamp_hi,
    );
  }

  let temp1 = half_btf_avx2(
    cospim32,
    u.add(20).read(),
    cospi32,
    u.add(27).read(),
    rnding,
    bit,
  );
  let temp2 = half_btf_avx2(
    cospim32,
    u.add(21).read(),
    cospi32,
    u.add(26).read(),
    rnding,
    bit,
  );
  let temp3 = half_btf_avx2(
    cospim32,
    u.add(22).read(),
    cospi32,
    u.add(25).read(),
    rnding,
    bit,
  );
  let temp4 = half_btf_avx2(
    cospim32,
    u.add(23).read(),
    cospi32,
    u.add(24).read(),
    rnding,
    bit,
  );
  u.add(24).write(half_btf_avx2(
    cospi32,
    u.add(23).read(),
    cospi32,
    u.add(24).read(),
    rnding,
    bit,
  ));
  u.add(25).write(half_btf_avx2(
    cospi32,
    u.add(22).read(),
    cospi32,
    u.add(25).read(),
    rnding,
    bit,
  ));
  u.add(26).write(half_btf_avx2(
    cospi32,
    u.add(21).read(),
    cospi32,
    u.add(26).read(),
    rnding,
    bit,
  ));
  u.add(27).write(half_btf_avx2(
    cospi32,
    u.add(20).read(),
    cospi32,
    u.add(27).read(),
    rnding,
    bit,
  ));
  u.add(20).write(temp1);
  u.add(21).write(temp2);
  u.add(22).write(temp3);
  u.add(23).write(temp4);
  for i in 32..40 {
    addsub_avx2(
      u.add(i).read(),
      u.add(i ^ 15).read(),
      u.add(i).as_mut().unwrap(),
      u.add(i ^ 15).as_mut().unwrap(),
      clamp_lo,
      clamp_hi,
    );
  }

  for i in 48..56 {
    addsub_avx2(
      u.add(i ^ 15).read(),
      u.add(i).read(),
      u.add(i ^ 15).as_mut().unwrap(),
      u.add(i).as_mut().unwrap(),
      clamp_lo,
      clamp_hi,
    );
  }
}

#[inline(always)]
unsafe fn idct64_stage10_avx2(
  u: *mut __m256i, cospim32: __m256i, cospi32: __m256i, clamp_lo: __m256i,
  clamp_hi: __m256i, rnding: __m256i, bit: i32,
) {
  let mut temp1;
  let mut temp2;
  let mut temp3;
  let mut temp4;
  for i in 0..16 {
    addsub_avx2(
      u.add(i).read(),
      u.add(31 - i).read(),
      u.add(i).as_mut().unwrap(),
      u.add(31 - i).as_mut().unwrap(),
      clamp_lo,
      clamp_hi,
    );
  }

  temp1 = half_btf_avx2(
    cospim32,
    u.add(40).read(),
    cospi32,
    u.add(55).read(),
    rnding,
    bit,
  );
  temp2 = half_btf_avx2(
    cospim32,
    u.add(41).read(),
    cospi32,
    u.add(54).read(),
    rnding,
    bit,
  );
  temp3 = half_btf_avx2(
    cospim32,
    u.add(42).read(),
    cospi32,
    u.add(53).read(),
    rnding,
    bit,
  );
  temp4 = half_btf_avx2(
    cospim32,
    u.add(43).read(),
    cospi32,
    u.add(52).read(),
    rnding,
    bit,
  );
  u.add(52).write(half_btf_avx2(
    cospi32,
    u.add(43).read(),
    cospi32,
    u.add(52).read(),
    rnding,
    bit,
  ));
  u.add(53).write(half_btf_avx2(
    cospi32,
    u.add(42).read(),
    cospi32,
    u.add(53).read(),
    rnding,
    bit,
  ));
  u.add(54).write(half_btf_avx2(
    cospi32,
    u.add(41).read(),
    cospi32,
    u.add(54).read(),
    rnding,
    bit,
  ));
  u.add(55).write(half_btf_avx2(
    cospi32,
    u.add(40).read(),
    cospi32,
    u.add(55).read(),
    rnding,
    bit,
  ));
  u.add(40).write(temp1);
  u.add(41).write(temp2);
  u.add(42).write(temp3);
  u.add(43).write(temp4);

  temp1 = half_btf_avx2(
    cospim32,
    u.add(44).read(),
    cospi32,
    u.add(51).read(),
    rnding,
    bit,
  );
  temp2 = half_btf_avx2(
    cospim32,
    u.add(45).read(),
    cospi32,
    u.add(50).read(),
    rnding,
    bit,
  );
  temp3 = half_btf_avx2(
    cospim32,
    u.add(46).read(),
    cospi32,
    u.add(49).read(),
    rnding,
    bit,
  );
  temp4 = half_btf_avx2(
    cospim32,
    u.add(47).read(),
    cospi32,
    u.add(48).read(),
    rnding,
    bit,
  );
  u.add(48).write(half_btf_avx2(
    cospi32,
    u.add(47).read(),
    cospi32,
    u.add(48).read(),
    rnding,
    bit,
  ));
  u.add(49).write(half_btf_avx2(
    cospi32,
    u.add(46).read(),
    cospi32,
    u.add(49).read(),
    rnding,
    bit,
  ));
  u.add(50).write(half_btf_avx2(
    cospi32,
    u.add(45).read(),
    cospi32,
    u.add(50).read(),
    rnding,
    bit,
  ));
  u.add(51).write(half_btf_avx2(
    cospi32,
    u.add(44).read(),
    cospi32,
    u.add(51).read(),
    rnding,
    bit,
  ));
  u.add(44).write(temp1);
  u.add(45).write(temp2);
  u.add(46).write(temp3);
  u.add(47).write(temp4);
}

#[inline(always)]
unsafe fn idct64_stage11_avx2(
  u: *mut __m256i, out: *mut __m256i, do_cols: bool, bd: usize,
  out_shift: i32, clamp_lo: __m256i, clamp_hi: __m256i,
) {
  for i in 0..32 {
    addsub_avx2(
      u.add(i).read(),
      u.add(63 - i).read(),
      out.add(i).as_mut().unwrap(),
      out.add(63 - i).as_mut().unwrap(),
      clamp_lo,
      clamp_hi,
    );
  }

  if !do_cols {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);

    round_shift_8x8_avx2(out, out_shift);
    round_shift_8x8_avx2(out.add(16), out_shift);
    round_shift_8x8_avx2(out.add(32), out_shift);
    round_shift_8x8_avx2(out.add(48), out_shift);
    highbd_clamp_epi32_avx2(out, out, clamp_lo_out, clamp_hi_out, 64);
  }
}

#[target_feature(enable = "avx2")]
unsafe fn iadst8x8_low1_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let k_zero = _mm256_setzero_si256();
  let mut u = [_mm256_setzero_si256(); 8];
  let mut x;

  // stage 0
  // stage 1
  // stage 2

  x = _mm256_mullo_epi32(input.read(), cospi60);
  u[0] = _mm256_add_epi32(x, rnding);
  u[0] = _mm256_srai_epi32(u[0], bit);

  x = _mm256_mullo_epi32(input.read(), cospi4);
  u[1] = _mm256_sub_epi32(k_zero, x);
  u[1] = _mm256_add_epi32(u[1], rnding);
  u[1] = _mm256_srai_epi32(u[1], bit);

  // stage 3
  // stage 4
  let mut temp1;
  temp1 = _mm256_mullo_epi32(u[0], cospi16);
  x = _mm256_mullo_epi32(u[1], cospi48);
  temp1 = _mm256_add_epi32(temp1, x);
  temp1 = _mm256_add_epi32(temp1, rnding);
  temp1 = _mm256_srai_epi32(temp1, bit);
  u[4] = temp1;

  let temp2 = _mm256_mullo_epi32(u[0], cospi48);
  x = _mm256_mullo_epi32(u[1], cospi16);
  u[5] = _mm256_sub_epi32(temp2, x);
  u[5] = _mm256_add_epi32(u[5], rnding);
  u[5] = _mm256_srai_epi32(u[5], bit);

  // stage 5
  // stage 6
  temp1 = _mm256_mullo_epi32(u[0], cospi32);
  x = _mm256_mullo_epi32(u[1], cospi32);
  u[2] = _mm256_add_epi32(temp1, x);
  u[2] = _mm256_add_epi32(u[2], rnding);
  u[2] = _mm256_srai_epi32(u[2], bit);

  u[3] = _mm256_sub_epi32(temp1, x);
  u[3] = _mm256_add_epi32(u[3], rnding);
  u[3] = _mm256_srai_epi32(u[3], bit);

  temp1 = _mm256_mullo_epi32(u[4], cospi32);
  x = _mm256_mullo_epi32(u[5], cospi32);
  u[6] = _mm256_add_epi32(temp1, x);
  u[6] = _mm256_add_epi32(u[6], rnding);
  u[6] = _mm256_srai_epi32(u[6], bit);

  u[7] = _mm256_sub_epi32(temp1, x);
  u[7] = _mm256_add_epi32(u[7], rnding);
  u[7] = _mm256_srai_epi32(u[7], bit);

  // stage 7
  if do_cols {
    output.add(0).write(u[0]);
    output.add(1).write(_mm256_sub_epi32(k_zero, u[4]));
    output.add(2).write(u[6]);
    output.add(3).write(_mm256_sub_epi32(k_zero, u[2]));
    output.add(4).write(u[3]);
    output.add(5).write(_mm256_sub_epi32(k_zero, u[7]));
    output.add(6).write(u[5]);
    output.add(7).write(_mm256_sub_epi32(k_zero, u[1]));
  } else {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);

    neg_shift_avx2(
      u[0],
      u[4],
      output.add(0).as_mut().unwrap(),
      output.add(1).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[6],
      u[2],
      output.add(2).as_mut().unwrap(),
      output.add(3).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[3],
      u[7],
      output.add(4).as_mut().unwrap(),
      output.add(5).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[5],
      u[1],
      output.add(6).as_mut().unwrap(),
      output.add(7).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
  }
}

#[target_feature(enable = "avx2")]
unsafe fn iadst8x8_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi4 = _mm256_set1_epi32(cospi[4]);
  let cospi60 = _mm256_set1_epi32(cospi[60]);
  let cospi20 = _mm256_set1_epi32(cospi[20]);
  let cospi44 = _mm256_set1_epi32(cospi[44]);
  let cospi36 = _mm256_set1_epi32(cospi[36]);
  let cospi28 = _mm256_set1_epi32(cospi[28]);
  let cospi52 = _mm256_set1_epi32(cospi[52]);
  let cospi12 = _mm256_set1_epi32(cospi[12]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let k_zero = _mm256_setzero_si256();
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  let mut u = [_mm256_setzero_si256(); 8];
  let mut v = [_mm256_setzero_si256(); 8];
  let mut x;

  // stage 0
  // stage 1
  // stage 2

  u[0] = _mm256_mullo_epi32(input.add(7).read(), cospi4);
  x = _mm256_mullo_epi32(input.add(0).read(), cospi60);
  u[0] = _mm256_add_epi32(u[0], x);
  u[0] = _mm256_add_epi32(u[0], rnding);
  u[0] = _mm256_srai_epi32(u[0], bit);

  u[1] = _mm256_mullo_epi32(input.add(7).read(), cospi60);
  x = _mm256_mullo_epi32(input.add(0).read(), cospi4);
  u[1] = _mm256_sub_epi32(u[1], x);
  u[1] = _mm256_add_epi32(u[1], rnding);
  u[1] = _mm256_srai_epi32(u[1], bit);

  u[2] = _mm256_mullo_epi32(input.add(5).read(), cospi20);
  x = _mm256_mullo_epi32(input.add(2).read(), cospi44);
  u[2] = _mm256_add_epi32(u[2], x);
  u[2] = _mm256_add_epi32(u[2], rnding);
  u[2] = _mm256_srai_epi32(u[2], bit);

  u[3] = _mm256_mullo_epi32(input.add(5).read(), cospi44);
  x = _mm256_mullo_epi32(input.add(2).read(), cospi20);
  u[3] = _mm256_sub_epi32(u[3], x);
  u[3] = _mm256_add_epi32(u[3], rnding);
  u[3] = _mm256_srai_epi32(u[3], bit);

  u[4] = _mm256_mullo_epi32(input.add(3).read(), cospi36);
  x = _mm256_mullo_epi32(input.add(4).read(), cospi28);
  u[4] = _mm256_add_epi32(u[4], x);
  u[4] = _mm256_add_epi32(u[4], rnding);
  u[4] = _mm256_srai_epi32(u[4], bit);

  u[5] = _mm256_mullo_epi32(input.add(3).read(), cospi28);
  x = _mm256_mullo_epi32(input.add(4).read(), cospi36);
  u[5] = _mm256_sub_epi32(u[5], x);
  u[5] = _mm256_add_epi32(u[5], rnding);
  u[5] = _mm256_srai_epi32(u[5], bit);

  u[6] = _mm256_mullo_epi32(input.add(1).read(), cospi52);
  x = _mm256_mullo_epi32(input.add(6).read(), cospi12);
  u[6] = _mm256_add_epi32(u[6], x);
  u[6] = _mm256_add_epi32(u[6], rnding);
  u[6] = _mm256_srai_epi32(u[6], bit);

  u[7] = _mm256_mullo_epi32(input.add(1).read(), cospi12);
  x = _mm256_mullo_epi32(input.add(6).read(), cospi52);
  u[7] = _mm256_sub_epi32(u[7], x);
  u[7] = _mm256_add_epi32(u[7], rnding);
  u[7] = _mm256_srai_epi32(u[7], bit);

  // stage 3
  addsub_avx2(u[0], u[4], &mut v[0], &mut v[4], clamp_lo, clamp_hi);
  addsub_avx2(u[1], u[5], &mut v[1], &mut v[5], clamp_lo, clamp_hi);
  addsub_avx2(u[2], u[6], &mut v[2], &mut v[6], clamp_lo, clamp_hi);
  addsub_avx2(u[3], u[7], &mut v[3], &mut v[7], clamp_lo, clamp_hi);

  // stage 4
  u[0] = v[0];
  u[1] = v[1];
  u[2] = v[2];
  u[3] = v[3];

  u[4] = _mm256_mullo_epi32(v[4], cospi16);
  x = _mm256_mullo_epi32(v[5], cospi48);
  u[4] = _mm256_add_epi32(u[4], x);
  u[4] = _mm256_add_epi32(u[4], rnding);
  u[4] = _mm256_srai_epi32(u[4], bit);

  u[5] = _mm256_mullo_epi32(v[4], cospi48);
  x = _mm256_mullo_epi32(v[5], cospi16);
  u[5] = _mm256_sub_epi32(u[5], x);
  u[5] = _mm256_add_epi32(u[5], rnding);
  u[5] = _mm256_srai_epi32(u[5], bit);

  u[6] = _mm256_mullo_epi32(v[6], cospim48);
  x = _mm256_mullo_epi32(v[7], cospi16);
  u[6] = _mm256_add_epi32(u[6], x);
  u[6] = _mm256_add_epi32(u[6], rnding);
  u[6] = _mm256_srai_epi32(u[6], bit);

  u[7] = _mm256_mullo_epi32(v[6], cospi16);
  x = _mm256_mullo_epi32(v[7], cospim48);
  u[7] = _mm256_sub_epi32(u[7], x);
  u[7] = _mm256_add_epi32(u[7], rnding);
  u[7] = _mm256_srai_epi32(u[7], bit);

  // stage 5
  addsub_avx2(u[0], u[2], &mut v[0], &mut v[2], clamp_lo, clamp_hi);
  addsub_avx2(u[1], u[3], &mut v[1], &mut v[3], clamp_lo, clamp_hi);
  addsub_avx2(u[4], u[6], &mut v[4], &mut v[6], clamp_lo, clamp_hi);
  addsub_avx2(u[5], u[7], &mut v[5], &mut v[7], clamp_lo, clamp_hi);

  // stage 6
  u[0] = v[0];
  u[1] = v[1];
  u[4] = v[4];
  u[5] = v[5];

  v[0] = _mm256_mullo_epi32(v[2], cospi32);
  x = _mm256_mullo_epi32(v[3], cospi32);
  u[2] = _mm256_add_epi32(v[0], x);
  u[2] = _mm256_add_epi32(u[2], rnding);
  u[2] = _mm256_srai_epi32(u[2], bit);

  u[3] = _mm256_sub_epi32(v[0], x);
  u[3] = _mm256_add_epi32(u[3], rnding);
  u[3] = _mm256_srai_epi32(u[3], bit);

  v[0] = _mm256_mullo_epi32(v[6], cospi32);
  x = _mm256_mullo_epi32(v[7], cospi32);
  u[6] = _mm256_add_epi32(v[0], x);
  u[6] = _mm256_add_epi32(u[6], rnding);
  u[6] = _mm256_srai_epi32(u[6], bit);

  u[7] = _mm256_sub_epi32(v[0], x);
  u[7] = _mm256_add_epi32(u[7], rnding);
  u[7] = _mm256_srai_epi32(u[7], bit);

  // stage 7
  if do_cols {
    output.add(0).write(u[0]);
    output.add(1).write(_mm256_sub_epi32(k_zero, u[4]));
    output.add(2).write(u[6]);
    output.add(3).write(_mm256_sub_epi32(k_zero, u[2]));
    output.add(4).write(u[3]);
    output.add(5).write(_mm256_sub_epi32(k_zero, u[7]));
    output.add(6).write(u[5]);
    output.add(7).write(_mm256_sub_epi32(k_zero, u[1]));
  } else {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);

    neg_shift_avx2(
      u[0],
      u[4],
      output.add(0).as_mut().unwrap(),
      output.add(1).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[6],
      u[2],
      output.add(2).as_mut().unwrap(),
      output.add(3).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[3],
      u[7],
      output.add(4).as_mut().unwrap(),
      output.add(5).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[5],
      u[1],
      output.add(6).as_mut().unwrap(),
      output.add(7).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
  }
}

#[target_feature(enable = "avx2")]
unsafe fn iadst16_low1_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi2 = _mm256_set1_epi32(cospi[2]);
  let cospi62 = _mm256_set1_epi32(cospi[62]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let zero = _mm256_setzero_si256();
  let mut v = [_mm256_setzero_si256(); 16];
  let mut x;
  let mut y;
  let mut temp1;
  let mut temp2;

  // Calculate the column 0, 1, 2, 3

  // stage 0
  // stage 1
  // stage 2
  x = _mm256_mullo_epi32(input.read(), cospi62);
  v[0] = _mm256_add_epi32(x, rnding);
  v[0] = _mm256_srai_epi32(v[0], bit);

  x = _mm256_mullo_epi32(input.read(), cospi2);
  v[1] = _mm256_sub_epi32(zero, x);
  v[1] = _mm256_add_epi32(v[1], rnding);
  v[1] = _mm256_srai_epi32(v[1], bit);

  // stage 3
  v[8] = v[0];
  v[9] = v[1];

  // stage 4
  temp1 = _mm256_mullo_epi32(v[8], cospi8);
  x = _mm256_mullo_epi32(v[9], cospi56);
  temp1 = _mm256_add_epi32(temp1, x);
  temp1 = _mm256_add_epi32(temp1, rnding);
  temp1 = _mm256_srai_epi32(temp1, bit);

  temp2 = _mm256_mullo_epi32(v[8], cospi56);
  x = _mm256_mullo_epi32(v[9], cospi8);
  temp2 = _mm256_sub_epi32(temp2, x);
  temp2 = _mm256_add_epi32(temp2, rnding);
  temp2 = _mm256_srai_epi32(temp2, bit);
  v[8] = temp1;
  v[9] = temp2;

  // stage 5
  v[4] = v[0];
  v[5] = v[1];
  v[12] = v[8];
  v[13] = v[9];

  // stage 6
  temp1 = _mm256_mullo_epi32(v[4], cospi16);
  x = _mm256_mullo_epi32(v[5], cospi48);
  temp1 = _mm256_add_epi32(temp1, x);
  temp1 = _mm256_add_epi32(temp1, rnding);
  temp1 = _mm256_srai_epi32(temp1, bit);

  temp2 = _mm256_mullo_epi32(v[4], cospi48);
  x = _mm256_mullo_epi32(v[5], cospi16);
  temp2 = _mm256_sub_epi32(temp2, x);
  temp2 = _mm256_add_epi32(temp2, rnding);
  temp2 = _mm256_srai_epi32(temp2, bit);
  v[4] = temp1;
  v[5] = temp2;

  temp1 = _mm256_mullo_epi32(v[12], cospi16);
  x = _mm256_mullo_epi32(v[13], cospi48);
  temp1 = _mm256_add_epi32(temp1, x);
  temp1 = _mm256_add_epi32(temp1, rnding);
  temp1 = _mm256_srai_epi32(temp1, bit);

  temp2 = _mm256_mullo_epi32(v[12], cospi48);
  x = _mm256_mullo_epi32(v[13], cospi16);
  temp2 = _mm256_sub_epi32(temp2, x);
  temp2 = _mm256_add_epi32(temp2, rnding);
  temp2 = _mm256_srai_epi32(temp2, bit);
  v[12] = temp1;
  v[13] = temp2;

  // stage 7
  v[2] = v[0];
  v[3] = v[1];
  v[6] = v[4];
  v[7] = v[5];
  v[10] = v[8];
  v[11] = v[9];
  v[14] = v[12];
  v[15] = v[13];

  // stage 8
  y = _mm256_mullo_epi32(v[2], cospi32);
  x = _mm256_mullo_epi32(v[3], cospi32);
  v[2] = _mm256_add_epi32(y, x);
  v[2] = _mm256_add_epi32(v[2], rnding);
  v[2] = _mm256_srai_epi32(v[2], bit);

  v[3] = _mm256_sub_epi32(y, x);
  v[3] = _mm256_add_epi32(v[3], rnding);
  v[3] = _mm256_srai_epi32(v[3], bit);

  y = _mm256_mullo_epi32(v[6], cospi32);
  x = _mm256_mullo_epi32(v[7], cospi32);
  v[6] = _mm256_add_epi32(y, x);
  v[6] = _mm256_add_epi32(v[6], rnding);
  v[6] = _mm256_srai_epi32(v[6], bit);

  v[7] = _mm256_sub_epi32(y, x);
  v[7] = _mm256_add_epi32(v[7], rnding);
  v[7] = _mm256_srai_epi32(v[7], bit);

  y = _mm256_mullo_epi32(v[10], cospi32);
  x = _mm256_mullo_epi32(v[11], cospi32);
  v[10] = _mm256_add_epi32(y, x);
  v[10] = _mm256_add_epi32(v[10], rnding);
  v[10] = _mm256_srai_epi32(v[10], bit);

  v[11] = _mm256_sub_epi32(y, x);
  v[11] = _mm256_add_epi32(v[11], rnding);
  v[11] = _mm256_srai_epi32(v[11], bit);

  y = _mm256_mullo_epi32(v[14], cospi32);
  x = _mm256_mullo_epi32(v[15], cospi32);
  v[14] = _mm256_add_epi32(y, x);
  v[14] = _mm256_add_epi32(v[14], rnding);
  v[14] = _mm256_srai_epi32(v[14], bit);

  v[15] = _mm256_sub_epi32(y, x);
  v[15] = _mm256_add_epi32(v[15], rnding);
  v[15] = _mm256_srai_epi32(v[15], bit);

  // stage 9
  if do_cols {
    output.add(0).write(v[0]);
    output.add(1).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[8]));
    output.add(2).write(v[12]);
    output.add(3).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[4]));
    output.add(4).write(v[6]);
    output.add(5).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[14]));
    output.add(6).write(v[10]);
    output.add(7).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[2]));
    output.add(8).write(v[3]);
    output.add(9).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[11]));
    output.add(10).write(v[15]);
    output.add(11).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[7]));
    output.add(12).write(v[5]);
    output.add(13).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[13]));
    output.add(14).write(v[9]);
    output.add(15).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[1]));
  } else {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);

    neg_shift_avx2(
      v[0],
      v[8],
      output.add(0).as_mut().unwrap(),
      output.add(1).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[12],
      v[4],
      output.add(2).as_mut().unwrap(),
      output.add(3).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[6],
      v[14],
      output.add(4).as_mut().unwrap(),
      output.add(5).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[10],
      v[2],
      output.add(6).as_mut().unwrap(),
      output.add(7).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[3],
      v[11],
      output.add(8).as_mut().unwrap(),
      output.add(9).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[15],
      v[7],
      output.add(10).as_mut().unwrap(),
      output.add(11).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[5],
      v[13],
      output.add(12).as_mut().unwrap(),
      output.add(13).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[9],
      v[1],
      output.add(14).as_mut().unwrap(),
      output.add(15).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
  }
}

#[target_feature(enable = "avx2")]
unsafe fn iadst16_low8_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi2 = _mm256_set1_epi32(cospi[2]);
  let cospi62 = _mm256_set1_epi32(cospi[62]);
  let cospi10 = _mm256_set1_epi32(cospi[10]);
  let cospi54 = _mm256_set1_epi32(cospi[54]);
  let cospi18 = _mm256_set1_epi32(cospi[18]);
  let cospi46 = _mm256_set1_epi32(cospi[46]);
  let cospi26 = _mm256_set1_epi32(cospi[26]);
  let cospi38 = _mm256_set1_epi32(cospi[38]);
  let cospi34 = _mm256_set1_epi32(cospi[34]);
  let cospi30 = _mm256_set1_epi32(cospi[30]);
  let cospi42 = _mm256_set1_epi32(cospi[42]);
  let cospi22 = _mm256_set1_epi32(cospi[22]);
  let cospi50 = _mm256_set1_epi32(cospi[50]);
  let cospi14 = _mm256_set1_epi32(cospi[14]);
  let cospi58 = _mm256_set1_epi32(cospi[58]);
  let cospi6 = _mm256_set1_epi32(cospi[6]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospim56 = _mm256_set1_epi32(-cospi[56]);
  let cospim24 = _mm256_set1_epi32(-cospi[24]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  let mut u = [_mm256_setzero_si256(); 16];
  let mut x;
  let mut y;

  // stage 0
  // stage 1
  // stage 2
  let zero = _mm256_setzero_si256();
  x = _mm256_mullo_epi32(input.add(0).read(), cospi62);
  u[0] = _mm256_add_epi32(x, rnding);
  u[0] = _mm256_srai_epi32(u[0], bit);

  x = _mm256_mullo_epi32(input.add(0).read(), cospi2);
  u[1] = _mm256_sub_epi32(zero, x);
  u[1] = _mm256_add_epi32(u[1], rnding);
  u[1] = _mm256_srai_epi32(u[1], bit);

  x = _mm256_mullo_epi32(input.add(2).read(), cospi54);
  u[2] = _mm256_add_epi32(x, rnding);
  u[2] = _mm256_srai_epi32(u[2], bit);

  x = _mm256_mullo_epi32(input.add(2).read(), cospi10);
  u[3] = _mm256_sub_epi32(zero, x);
  u[3] = _mm256_add_epi32(u[3], rnding);
  u[3] = _mm256_srai_epi32(u[3], bit);

  x = _mm256_mullo_epi32(input.add(4).read(), cospi46);
  u[4] = _mm256_add_epi32(x, rnding);
  u[4] = _mm256_srai_epi32(u[4], bit);

  x = _mm256_mullo_epi32(input.add(4).read(), cospi18);
  u[5] = _mm256_sub_epi32(zero, x);
  u[5] = _mm256_add_epi32(u[5], rnding);
  u[5] = _mm256_srai_epi32(u[5], bit);

  x = _mm256_mullo_epi32(input.add(6).read(), cospi38);
  u[6] = _mm256_add_epi32(x, rnding);
  u[6] = _mm256_srai_epi32(u[6], bit);

  x = _mm256_mullo_epi32(input.add(6).read(), cospi26);
  u[7] = _mm256_sub_epi32(zero, x);
  u[7] = _mm256_add_epi32(u[7], rnding);
  u[7] = _mm256_srai_epi32(u[7], bit);

  u[8] = _mm256_mullo_epi32(input.add(7).read(), cospi34);
  u[8] = _mm256_add_epi32(u[8], rnding);
  u[8] = _mm256_srai_epi32(u[8], bit);

  u[9] = _mm256_mullo_epi32(input.add(7).read(), cospi30);
  u[9] = _mm256_add_epi32(u[9], rnding);
  u[9] = _mm256_srai_epi32(u[9], bit);

  u[10] = _mm256_mullo_epi32(input.add(5).read(), cospi42);
  u[10] = _mm256_add_epi32(u[10], rnding);
  u[10] = _mm256_srai_epi32(u[10], bit);

  u[11] = _mm256_mullo_epi32(input.add(5).read(), cospi22);
  u[11] = _mm256_add_epi32(u[11], rnding);
  u[11] = _mm256_srai_epi32(u[11], bit);

  u[12] = _mm256_mullo_epi32(input.add(3).read(), cospi50);
  u[12] = _mm256_add_epi32(u[12], rnding);
  u[12] = _mm256_srai_epi32(u[12], bit);

  u[13] = _mm256_mullo_epi32(input.add(3).read(), cospi14);
  u[13] = _mm256_add_epi32(u[13], rnding);
  u[13] = _mm256_srai_epi32(u[13], bit);

  u[14] = _mm256_mullo_epi32(input.add(1).read(), cospi58);
  u[14] = _mm256_add_epi32(u[14], rnding);
  u[14] = _mm256_srai_epi32(u[14], bit);

  u[15] = _mm256_mullo_epi32(input.add(1).read(), cospi6);
  u[15] = _mm256_add_epi32(u[15], rnding);
  u[15] = _mm256_srai_epi32(u[15], bit);

  // stage 3
  addsub_avx2(u[0], u[8], &mut u[0], &mut u[8], clamp_lo, clamp_hi);
  addsub_avx2(u[1], u[9], &mut u[1], &mut u[9], clamp_lo, clamp_hi);
  addsub_avx2(u[2], u[10], &mut u[2], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(u[3], u[11], &mut u[3], &mut u[11], clamp_lo, clamp_hi);
  addsub_avx2(u[4], u[12], &mut u[4], &mut u[12], clamp_lo, clamp_hi);
  addsub_avx2(u[5], u[13], &mut u[5], &mut u[13], clamp_lo, clamp_hi);
  addsub_avx2(u[6], u[14], &mut u[6], &mut u[14], clamp_lo, clamp_hi);
  addsub_avx2(u[7], u[15], &mut u[7], &mut u[15], clamp_lo, clamp_hi);

  // stage 4
  y = _mm256_mullo_epi32(u[8], cospi56);
  x = _mm256_mullo_epi32(u[9], cospi56);
  u[8] = _mm256_mullo_epi32(u[8], cospi8);
  u[8] = _mm256_add_epi32(u[8], x);
  u[8] = _mm256_add_epi32(u[8], rnding);
  u[8] = _mm256_srai_epi32(u[8], bit);

  x = _mm256_mullo_epi32(u[9], cospi8);
  u[9] = _mm256_sub_epi32(y, x);
  u[9] = _mm256_add_epi32(u[9], rnding);
  u[9] = _mm256_srai_epi32(u[9], bit);

  x = _mm256_mullo_epi32(u[11], cospi24);
  y = _mm256_mullo_epi32(u[10], cospi24);
  u[10] = _mm256_mullo_epi32(u[10], cospi40);
  u[10] = _mm256_add_epi32(u[10], x);
  u[10] = _mm256_add_epi32(u[10], rnding);
  u[10] = _mm256_srai_epi32(u[10], bit);

  x = _mm256_mullo_epi32(u[11], cospi40);
  u[11] = _mm256_sub_epi32(y, x);
  u[11] = _mm256_add_epi32(u[11], rnding);
  u[11] = _mm256_srai_epi32(u[11], bit);

  x = _mm256_mullo_epi32(u[13], cospi8);
  y = _mm256_mullo_epi32(u[12], cospi8);
  u[12] = _mm256_mullo_epi32(u[12], cospim56);
  u[12] = _mm256_add_epi32(u[12], x);
  u[12] = _mm256_add_epi32(u[12], rnding);
  u[12] = _mm256_srai_epi32(u[12], bit);

  x = _mm256_mullo_epi32(u[13], cospim56);
  u[13] = _mm256_sub_epi32(y, x);
  u[13] = _mm256_add_epi32(u[13], rnding);
  u[13] = _mm256_srai_epi32(u[13], bit);

  x = _mm256_mullo_epi32(u[15], cospi40);
  y = _mm256_mullo_epi32(u[14], cospi40);
  u[14] = _mm256_mullo_epi32(u[14], cospim24);
  u[14] = _mm256_add_epi32(u[14], x);
  u[14] = _mm256_add_epi32(u[14], rnding);
  u[14] = _mm256_srai_epi32(u[14], bit);

  x = _mm256_mullo_epi32(u[15], cospim24);
  u[15] = _mm256_sub_epi32(y, x);
  u[15] = _mm256_add_epi32(u[15], rnding);
  u[15] = _mm256_srai_epi32(u[15], bit);

  // stage 5
  addsub_avx2(u[0], u[4], &mut u[0], &mut u[4], clamp_lo, clamp_hi);
  addsub_avx2(u[1], u[5], &mut u[1], &mut u[5], clamp_lo, clamp_hi);
  addsub_avx2(u[2], u[6], &mut u[2], &mut u[6], clamp_lo, clamp_hi);
  addsub_avx2(u[3], u[7], &mut u[3], &mut u[7], clamp_lo, clamp_hi);
  addsub_avx2(u[8], u[12], &mut u[8], &mut u[12], clamp_lo, clamp_hi);
  addsub_avx2(u[9], u[13], &mut u[9], &mut u[13], clamp_lo, clamp_hi);
  addsub_avx2(u[10], u[14], &mut u[10], &mut u[14], clamp_lo, clamp_hi);
  addsub_avx2(u[11], u[15], &mut u[11], &mut u[15], clamp_lo, clamp_hi);

  // stage 6
  x = _mm256_mullo_epi32(u[5], cospi48);
  y = _mm256_mullo_epi32(u[4], cospi48);
  u[4] = _mm256_mullo_epi32(u[4], cospi16);
  u[4] = _mm256_add_epi32(u[4], x);
  u[4] = _mm256_add_epi32(u[4], rnding);
  u[4] = _mm256_srai_epi32(u[4], bit);

  x = _mm256_mullo_epi32(u[5], cospi16);
  u[5] = _mm256_sub_epi32(y, x);
  u[5] = _mm256_add_epi32(u[5], rnding);
  u[5] = _mm256_srai_epi32(u[5], bit);

  x = _mm256_mullo_epi32(u[7], cospi16);
  y = _mm256_mullo_epi32(u[6], cospi16);
  u[6] = _mm256_mullo_epi32(u[6], cospim48);
  u[6] = _mm256_add_epi32(u[6], x);
  u[6] = _mm256_add_epi32(u[6], rnding);
  u[6] = _mm256_srai_epi32(u[6], bit);

  x = _mm256_mullo_epi32(u[7], cospim48);
  u[7] = _mm256_sub_epi32(y, x);
  u[7] = _mm256_add_epi32(u[7], rnding);
  u[7] = _mm256_srai_epi32(u[7], bit);

  x = _mm256_mullo_epi32(u[13], cospi48);
  y = _mm256_mullo_epi32(u[12], cospi48);
  u[12] = _mm256_mullo_epi32(u[12], cospi16);
  u[12] = _mm256_add_epi32(u[12], x);
  u[12] = _mm256_add_epi32(u[12], rnding);
  u[12] = _mm256_srai_epi32(u[12], bit);

  x = _mm256_mullo_epi32(u[13], cospi16);
  u[13] = _mm256_sub_epi32(y, x);
  u[13] = _mm256_add_epi32(u[13], rnding);
  u[13] = _mm256_srai_epi32(u[13], bit);

  x = _mm256_mullo_epi32(u[15], cospi16);
  y = _mm256_mullo_epi32(u[14], cospi16);
  u[14] = _mm256_mullo_epi32(u[14], cospim48);
  u[14] = _mm256_add_epi32(u[14], x);
  u[14] = _mm256_add_epi32(u[14], rnding);
  u[14] = _mm256_srai_epi32(u[14], bit);

  x = _mm256_mullo_epi32(u[15], cospim48);
  u[15] = _mm256_sub_epi32(y, x);
  u[15] = _mm256_add_epi32(u[15], rnding);
  u[15] = _mm256_srai_epi32(u[15], bit);

  // stage 7
  addsub_avx2(u[0], u[2], &mut u[0], &mut u[2], clamp_lo, clamp_hi);
  addsub_avx2(u[1], u[3], &mut u[1], &mut u[3], clamp_lo, clamp_hi);
  addsub_avx2(u[4], u[6], &mut u[4], &mut u[6], clamp_lo, clamp_hi);
  addsub_avx2(u[5], u[7], &mut u[5], &mut u[7], clamp_lo, clamp_hi);
  addsub_avx2(u[8], u[10], &mut u[8], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(u[9], u[11], &mut u[9], &mut u[11], clamp_lo, clamp_hi);
  addsub_avx2(u[12], u[14], &mut u[12], &mut u[14], clamp_lo, clamp_hi);
  addsub_avx2(u[13], u[15], &mut u[13], &mut u[15], clamp_lo, clamp_hi);

  // stage 8
  y = _mm256_mullo_epi32(u[2], cospi32);
  x = _mm256_mullo_epi32(u[3], cospi32);
  u[2] = _mm256_add_epi32(y, x);
  u[2] = _mm256_add_epi32(u[2], rnding);
  u[2] = _mm256_srai_epi32(u[2], bit);

  u[3] = _mm256_sub_epi32(y, x);
  u[3] = _mm256_add_epi32(u[3], rnding);
  u[3] = _mm256_srai_epi32(u[3], bit);
  y = _mm256_mullo_epi32(u[6], cospi32);
  x = _mm256_mullo_epi32(u[7], cospi32);
  u[6] = _mm256_add_epi32(y, x);
  u[6] = _mm256_add_epi32(u[6], rnding);
  u[6] = _mm256_srai_epi32(u[6], bit);

  u[7] = _mm256_sub_epi32(y, x);
  u[7] = _mm256_add_epi32(u[7], rnding);
  u[7] = _mm256_srai_epi32(u[7], bit);

  y = _mm256_mullo_epi32(u[10], cospi32);
  x = _mm256_mullo_epi32(u[11], cospi32);
  u[10] = _mm256_add_epi32(y, x);
  u[10] = _mm256_add_epi32(u[10], rnding);
  u[10] = _mm256_srai_epi32(u[10], bit);

  u[11] = _mm256_sub_epi32(y, x);
  u[11] = _mm256_add_epi32(u[11], rnding);
  u[11] = _mm256_srai_epi32(u[11], bit);

  y = _mm256_mullo_epi32(u[14], cospi32);
  x = _mm256_mullo_epi32(u[15], cospi32);
  u[14] = _mm256_add_epi32(y, x);
  u[14] = _mm256_add_epi32(u[14], rnding);
  u[14] = _mm256_srai_epi32(u[14], bit);

  u[15] = _mm256_sub_epi32(y, x);
  u[15] = _mm256_add_epi32(u[15], rnding);
  u[15] = _mm256_srai_epi32(u[15], bit);

  // stage 9
  if do_cols {
    output.add(0).write(u[0]);
    output.add(1).write(_mm256_sub_epi32(_mm256_setzero_si256(), u[8]));
    output.add(2).write(u[12]);
    output.add(3).write(_mm256_sub_epi32(_mm256_setzero_si256(), u[4]));
    output.add(4).write(u[6]);
    output.add(5).write(_mm256_sub_epi32(_mm256_setzero_si256(), u[14]));
    output.add(6).write(u[10]);
    output.add(7).write(_mm256_sub_epi32(_mm256_setzero_si256(), u[2]));
    output.add(8).write(u[3]);
    output.add(9).write(_mm256_sub_epi32(_mm256_setzero_si256(), u[11]));
    output.add(10).write(u[15]);
    output.add(11).write(_mm256_sub_epi32(_mm256_setzero_si256(), u[7]));
    output.add(12).write(u[5]);
    output.add(13).write(_mm256_sub_epi32(_mm256_setzero_si256(), u[13]));
    output.add(14).write(u[9]);
    output.add(15).write(_mm256_sub_epi32(_mm256_setzero_si256(), u[1]));
  } else {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);

    neg_shift_avx2(
      u[0],
      u[8],
      output.add(0).as_mut().unwrap(),
      output.add(1).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[12],
      u[4],
      output.add(2).as_mut().unwrap(),
      output.add(3).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[6],
      u[14],
      output.add(4).as_mut().unwrap(),
      output.add(5).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[10],
      u[2],
      output.add(6).as_mut().unwrap(),
      output.add(7).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[3],
      u[11],
      output.add(8).as_mut().unwrap(),
      output.add(9).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[15],
      u[7],
      output.add(10).as_mut().unwrap(),
      output.add(11).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[5],
      u[13],
      output.add(12).as_mut().unwrap(),
      output.add(13).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      u[9],
      u[1],
      output.add(14).as_mut().unwrap(),
      output.add(15).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
  }
}

#[target_feature(enable = "avx2")]
unsafe fn iadst16_avx2(
  input: *mut __m256i, output: *mut __m256i, bit: i32, do_cols: bool,
  bd: usize, out_shift: i32,
) {
  let cospi = cospi_arr(bit as usize);
  let cospi2 = _mm256_set1_epi32(cospi[2]);
  let cospi62 = _mm256_set1_epi32(cospi[62]);
  let cospi10 = _mm256_set1_epi32(cospi[10]);
  let cospi54 = _mm256_set1_epi32(cospi[54]);
  let cospi18 = _mm256_set1_epi32(cospi[18]);
  let cospi46 = _mm256_set1_epi32(cospi[46]);
  let cospi26 = _mm256_set1_epi32(cospi[26]);
  let cospi38 = _mm256_set1_epi32(cospi[38]);
  let cospi34 = _mm256_set1_epi32(cospi[34]);
  let cospi30 = _mm256_set1_epi32(cospi[30]);
  let cospi42 = _mm256_set1_epi32(cospi[42]);
  let cospi22 = _mm256_set1_epi32(cospi[22]);
  let cospi50 = _mm256_set1_epi32(cospi[50]);
  let cospi14 = _mm256_set1_epi32(cospi[14]);
  let cospi58 = _mm256_set1_epi32(cospi[58]);
  let cospi6 = _mm256_set1_epi32(cospi[6]);
  let cospi8 = _mm256_set1_epi32(cospi[8]);
  let cospi56 = _mm256_set1_epi32(cospi[56]);
  let cospi40 = _mm256_set1_epi32(cospi[40]);
  let cospi24 = _mm256_set1_epi32(cospi[24]);
  let cospim56 = _mm256_set1_epi32(-cospi[56]);
  let cospim24 = _mm256_set1_epi32(-cospi[24]);
  let cospi48 = _mm256_set1_epi32(cospi[48]);
  let cospi16 = _mm256_set1_epi32(cospi[16]);
  let cospim48 = _mm256_set1_epi32(-cospi[48]);
  let cospi32 = _mm256_set1_epi32(cospi[32]);
  let rnding = _mm256_set1_epi32(1 << (bit - 1));
  let log_range = cmp::max(16, bd + if do_cols { 6 } else { 8 });
  let clamp_lo = _mm256_set1_epi32(-(1 << (log_range - 1)));
  let clamp_hi = _mm256_set1_epi32((1 << (log_range - 1)) - 1);
  let mut u = [_mm256_setzero_si256(); 16];
  let mut v = [_mm256_setzero_si256(); 16];
  let mut x;
  let mut y;

  // stage 0
  // stage 1
  // stage 2
  v[0] = _mm256_mullo_epi32(input.add(15).read(), cospi2);
  x = _mm256_mullo_epi32(input.add(0).read(), cospi62);
  v[0] = _mm256_add_epi32(v[0], x);
  v[0] = _mm256_add_epi32(v[0], rnding);
  v[0] = _mm256_srai_epi32(v[0], bit);

  v[1] = _mm256_mullo_epi32(input.add(15).read(), cospi62);
  x = _mm256_mullo_epi32(input.add(0).read(), cospi2);
  v[1] = _mm256_sub_epi32(v[1], x);
  v[1] = _mm256_add_epi32(v[1], rnding);
  v[1] = _mm256_srai_epi32(v[1], bit);

  v[2] = _mm256_mullo_epi32(input.add(13).read(), cospi10);
  x = _mm256_mullo_epi32(input.add(2).read(), cospi54);
  v[2] = _mm256_add_epi32(v[2], x);
  v[2] = _mm256_add_epi32(v[2], rnding);
  v[2] = _mm256_srai_epi32(v[2], bit);

  v[3] = _mm256_mullo_epi32(input.add(13).read(), cospi54);
  x = _mm256_mullo_epi32(input.add(2).read(), cospi10);
  v[3] = _mm256_sub_epi32(v[3], x);
  v[3] = _mm256_add_epi32(v[3], rnding);
  v[3] = _mm256_srai_epi32(v[3], bit);

  v[4] = _mm256_mullo_epi32(input.add(11).read(), cospi18);
  x = _mm256_mullo_epi32(input.add(4).read(), cospi46);
  v[4] = _mm256_add_epi32(v[4], x);
  v[4] = _mm256_add_epi32(v[4], rnding);
  v[4] = _mm256_srai_epi32(v[4], bit);

  v[5] = _mm256_mullo_epi32(input.add(11).read(), cospi46);
  x = _mm256_mullo_epi32(input.add(4).read(), cospi18);
  v[5] = _mm256_sub_epi32(v[5], x);
  v[5] = _mm256_add_epi32(v[5], rnding);
  v[5] = _mm256_srai_epi32(v[5], bit);

  v[6] = _mm256_mullo_epi32(input.add(9).read(), cospi26);
  x = _mm256_mullo_epi32(input.add(6).read(), cospi38);
  v[6] = _mm256_add_epi32(v[6], x);
  v[6] = _mm256_add_epi32(v[6], rnding);
  v[6] = _mm256_srai_epi32(v[6], bit);

  v[7] = _mm256_mullo_epi32(input.add(9).read(), cospi38);
  x = _mm256_mullo_epi32(input.add(6).read(), cospi26);
  v[7] = _mm256_sub_epi32(v[7], x);
  v[7] = _mm256_add_epi32(v[7], rnding);
  v[7] = _mm256_srai_epi32(v[7], bit);

  v[8] = _mm256_mullo_epi32(input.add(7).read(), cospi34);
  x = _mm256_mullo_epi32(input.add(8).read(), cospi30);
  v[8] = _mm256_add_epi32(v[8], x);
  v[8] = _mm256_add_epi32(v[8], rnding);
  v[8] = _mm256_srai_epi32(v[8], bit);

  v[9] = _mm256_mullo_epi32(input.add(7).read(), cospi30);
  x = _mm256_mullo_epi32(input.add(8).read(), cospi34);
  v[9] = _mm256_sub_epi32(v[9], x);
  v[9] = _mm256_add_epi32(v[9], rnding);
  v[9] = _mm256_srai_epi32(v[9], bit);

  v[10] = _mm256_mullo_epi32(input.add(5).read(), cospi42);
  x = _mm256_mullo_epi32(input.add(10).read(), cospi22);
  v[10] = _mm256_add_epi32(v[10], x);
  v[10] = _mm256_add_epi32(v[10], rnding);
  v[10] = _mm256_srai_epi32(v[10], bit);

  v[11] = _mm256_mullo_epi32(input.add(5).read(), cospi22);
  x = _mm256_mullo_epi32(input.add(10).read(), cospi42);
  v[11] = _mm256_sub_epi32(v[11], x);
  v[11] = _mm256_add_epi32(v[11], rnding);
  v[11] = _mm256_srai_epi32(v[11], bit);

  v[12] = _mm256_mullo_epi32(input.add(3).read(), cospi50);
  x = _mm256_mullo_epi32(input.add(12).read(), cospi14);
  v[12] = _mm256_add_epi32(v[12], x);
  v[12] = _mm256_add_epi32(v[12], rnding);
  v[12] = _mm256_srai_epi32(v[12], bit);

  v[13] = _mm256_mullo_epi32(input.add(3).read(), cospi14);
  x = _mm256_mullo_epi32(input.add(12).read(), cospi50);
  v[13] = _mm256_sub_epi32(v[13], x);
  v[13] = _mm256_add_epi32(v[13], rnding);
  v[13] = _mm256_srai_epi32(v[13], bit);

  v[14] = _mm256_mullo_epi32(input.add(1).read(), cospi58);
  x = _mm256_mullo_epi32(input.add(14).read(), cospi6);
  v[14] = _mm256_add_epi32(v[14], x);
  v[14] = _mm256_add_epi32(v[14], rnding);
  v[14] = _mm256_srai_epi32(v[14], bit);

  v[15] = _mm256_mullo_epi32(input.add(1).read(), cospi6);
  x = _mm256_mullo_epi32(input.add(14).read(), cospi58);
  v[15] = _mm256_sub_epi32(v[15], x);
  v[15] = _mm256_add_epi32(v[15], rnding);
  v[15] = _mm256_srai_epi32(v[15], bit);

  // stage 3
  addsub_avx2(v[0], v[8], &mut u[0], &mut u[8], clamp_lo, clamp_hi);
  addsub_avx2(v[1], v[9], &mut u[1], &mut u[9], clamp_lo, clamp_hi);
  addsub_avx2(v[2], v[10], &mut u[2], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(v[3], v[11], &mut u[3], &mut u[11], clamp_lo, clamp_hi);
  addsub_avx2(v[4], v[12], &mut u[4], &mut u[12], clamp_lo, clamp_hi);
  addsub_avx2(v[5], v[13], &mut u[5], &mut u[13], clamp_lo, clamp_hi);
  addsub_avx2(v[6], v[14], &mut u[6], &mut u[14], clamp_lo, clamp_hi);
  addsub_avx2(v[7], v[15], &mut u[7], &mut u[15], clamp_lo, clamp_hi);

  // stage 4
  v[0] = u[0];
  v[1] = u[1];
  v[2] = u[2];
  v[3] = u[3];
  v[4] = u[4];
  v[5] = u[5];
  v[6] = u[6];
  v[7] = u[7];

  v[8] = _mm256_mullo_epi32(u[8], cospi8);
  x = _mm256_mullo_epi32(u[9], cospi56);
  v[8] = _mm256_add_epi32(v[8], x);
  v[8] = _mm256_add_epi32(v[8], rnding);
  v[8] = _mm256_srai_epi32(v[8], bit);

  v[9] = _mm256_mullo_epi32(u[8], cospi56);
  x = _mm256_mullo_epi32(u[9], cospi8);
  v[9] = _mm256_sub_epi32(v[9], x);
  v[9] = _mm256_add_epi32(v[9], rnding);
  v[9] = _mm256_srai_epi32(v[9], bit);

  v[10] = _mm256_mullo_epi32(u[10], cospi40);
  x = _mm256_mullo_epi32(u[11], cospi24);
  v[10] = _mm256_add_epi32(v[10], x);
  v[10] = _mm256_add_epi32(v[10], rnding);
  v[10] = _mm256_srai_epi32(v[10], bit);

  v[11] = _mm256_mullo_epi32(u[10], cospi24);
  x = _mm256_mullo_epi32(u[11], cospi40);
  v[11] = _mm256_sub_epi32(v[11], x);
  v[11] = _mm256_add_epi32(v[11], rnding);
  v[11] = _mm256_srai_epi32(v[11], bit);

  v[12] = _mm256_mullo_epi32(u[12], cospim56);
  x = _mm256_mullo_epi32(u[13], cospi8);
  v[12] = _mm256_add_epi32(v[12], x);
  v[12] = _mm256_add_epi32(v[12], rnding);
  v[12] = _mm256_srai_epi32(v[12], bit);

  v[13] = _mm256_mullo_epi32(u[12], cospi8);
  x = _mm256_mullo_epi32(u[13], cospim56);
  v[13] = _mm256_sub_epi32(v[13], x);
  v[13] = _mm256_add_epi32(v[13], rnding);
  v[13] = _mm256_srai_epi32(v[13], bit);

  v[14] = _mm256_mullo_epi32(u[14], cospim24);
  x = _mm256_mullo_epi32(u[15], cospi40);
  v[14] = _mm256_add_epi32(v[14], x);
  v[14] = _mm256_add_epi32(v[14], rnding);
  v[14] = _mm256_srai_epi32(v[14], bit);

  v[15] = _mm256_mullo_epi32(u[14], cospi40);
  x = _mm256_mullo_epi32(u[15], cospim24);
  v[15] = _mm256_sub_epi32(v[15], x);
  v[15] = _mm256_add_epi32(v[15], rnding);
  v[15] = _mm256_srai_epi32(v[15], bit);

  // stage 5
  addsub_avx2(v[0], v[4], &mut u[0], &mut u[4], clamp_lo, clamp_hi);
  addsub_avx2(v[1], v[5], &mut u[1], &mut u[5], clamp_lo, clamp_hi);
  addsub_avx2(v[2], v[6], &mut u[2], &mut u[6], clamp_lo, clamp_hi);
  addsub_avx2(v[3], v[7], &mut u[3], &mut u[7], clamp_lo, clamp_hi);
  addsub_avx2(v[8], v[12], &mut u[8], &mut u[12], clamp_lo, clamp_hi);
  addsub_avx2(v[9], v[13], &mut u[9], &mut u[13], clamp_lo, clamp_hi);
  addsub_avx2(v[10], v[14], &mut u[10], &mut u[14], clamp_lo, clamp_hi);
  addsub_avx2(v[11], v[15], &mut u[11], &mut u[15], clamp_lo, clamp_hi);

  // stage 6
  v[0] = u[0];
  v[1] = u[1];
  v[2] = u[2];
  v[3] = u[3];

  v[4] = _mm256_mullo_epi32(u[4], cospi16);
  x = _mm256_mullo_epi32(u[5], cospi48);
  v[4] = _mm256_add_epi32(v[4], x);
  v[4] = _mm256_add_epi32(v[4], rnding);
  v[4] = _mm256_srai_epi32(v[4], bit);

  v[5] = _mm256_mullo_epi32(u[4], cospi48);
  x = _mm256_mullo_epi32(u[5], cospi16);
  v[5] = _mm256_sub_epi32(v[5], x);
  v[5] = _mm256_add_epi32(v[5], rnding);
  v[5] = _mm256_srai_epi32(v[5], bit);

  v[6] = _mm256_mullo_epi32(u[6], cospim48);
  x = _mm256_mullo_epi32(u[7], cospi16);
  v[6] = _mm256_add_epi32(v[6], x);
  v[6] = _mm256_add_epi32(v[6], rnding);
  v[6] = _mm256_srai_epi32(v[6], bit);

  v[7] = _mm256_mullo_epi32(u[6], cospi16);
  x = _mm256_mullo_epi32(u[7], cospim48);
  v[7] = _mm256_sub_epi32(v[7], x);
  v[7] = _mm256_add_epi32(v[7], rnding);
  v[7] = _mm256_srai_epi32(v[7], bit);

  v[8] = u[8];
  v[9] = u[9];
  v[10] = u[10];
  v[11] = u[11];

  v[12] = _mm256_mullo_epi32(u[12], cospi16);
  x = _mm256_mullo_epi32(u[13], cospi48);
  v[12] = _mm256_add_epi32(v[12], x);
  v[12] = _mm256_add_epi32(v[12], rnding);
  v[12] = _mm256_srai_epi32(v[12], bit);

  v[13] = _mm256_mullo_epi32(u[12], cospi48);
  x = _mm256_mullo_epi32(u[13], cospi16);
  v[13] = _mm256_sub_epi32(v[13], x);
  v[13] = _mm256_add_epi32(v[13], rnding);
  v[13] = _mm256_srai_epi32(v[13], bit);

  v[14] = _mm256_mullo_epi32(u[14], cospim48);
  x = _mm256_mullo_epi32(u[15], cospi16);
  v[14] = _mm256_add_epi32(v[14], x);
  v[14] = _mm256_add_epi32(v[14], rnding);
  v[14] = _mm256_srai_epi32(v[14], bit);

  v[15] = _mm256_mullo_epi32(u[14], cospi16);
  x = _mm256_mullo_epi32(u[15], cospim48);
  v[15] = _mm256_sub_epi32(v[15], x);
  v[15] = _mm256_add_epi32(v[15], rnding);
  v[15] = _mm256_srai_epi32(v[15], bit);

  // stage 7
  addsub_avx2(v[0], v[2], &mut u[0], &mut u[2], clamp_lo, clamp_hi);
  addsub_avx2(v[1], v[3], &mut u[1], &mut u[3], clamp_lo, clamp_hi);
  addsub_avx2(v[4], v[6], &mut u[4], &mut u[6], clamp_lo, clamp_hi);
  addsub_avx2(v[5], v[7], &mut u[5], &mut u[7], clamp_lo, clamp_hi);
  addsub_avx2(v[8], v[10], &mut u[8], &mut u[10], clamp_lo, clamp_hi);
  addsub_avx2(v[9], v[11], &mut u[9], &mut u[11], clamp_lo, clamp_hi);
  addsub_avx2(v[12], v[14], &mut u[12], &mut u[14], clamp_lo, clamp_hi);
  addsub_avx2(v[13], v[15], &mut u[13], &mut u[15], clamp_lo, clamp_hi);

  // stage 8
  v[0] = u[0];
  v[1] = u[1];

  y = _mm256_mullo_epi32(u[2], cospi32);
  x = _mm256_mullo_epi32(u[3], cospi32);
  v[2] = _mm256_add_epi32(y, x);
  v[2] = _mm256_add_epi32(v[2], rnding);
  v[2] = _mm256_srai_epi32(v[2], bit);

  v[3] = _mm256_sub_epi32(y, x);
  v[3] = _mm256_add_epi32(v[3], rnding);
  v[3] = _mm256_srai_epi32(v[3], bit);

  v[4] = u[4];
  v[5] = u[5];

  y = _mm256_mullo_epi32(u[6], cospi32);
  x = _mm256_mullo_epi32(u[7], cospi32);
  v[6] = _mm256_add_epi32(y, x);
  v[6] = _mm256_add_epi32(v[6], rnding);
  v[6] = _mm256_srai_epi32(v[6], bit);

  v[7] = _mm256_sub_epi32(y, x);
  v[7] = _mm256_add_epi32(v[7], rnding);
  v[7] = _mm256_srai_epi32(v[7], bit);

  v[8] = u[8];
  v[9] = u[9];

  y = _mm256_mullo_epi32(u[10], cospi32);
  x = _mm256_mullo_epi32(u[11], cospi32);
  v[10] = _mm256_add_epi32(y, x);
  v[10] = _mm256_add_epi32(v[10], rnding);
  v[10] = _mm256_srai_epi32(v[10], bit);

  v[11] = _mm256_sub_epi32(y, x);
  v[11] = _mm256_add_epi32(v[11], rnding);
  v[11] = _mm256_srai_epi32(v[11], bit);

  v[12] = u[12];
  v[13] = u[13];

  y = _mm256_mullo_epi32(u[14], cospi32);
  x = _mm256_mullo_epi32(u[15], cospi32);
  v[14] = _mm256_add_epi32(y, x);
  v[14] = _mm256_add_epi32(v[14], rnding);
  v[14] = _mm256_srai_epi32(v[14], bit);

  v[15] = _mm256_sub_epi32(y, x);
  v[15] = _mm256_add_epi32(v[15], rnding);
  v[15] = _mm256_srai_epi32(v[15], bit);

  // stage 9
  if do_cols {
    output.add(0).write(v[0]);
    output.add(1).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[8]));
    output.add(2).write(v[12]);
    output.add(3).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[4]));
    output.add(4).write(v[6]);
    output.add(5).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[14]));
    output.add(6).write(v[10]);
    output.add(7).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[2]));
    output.add(8).write(v[3]);
    output.add(9).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[11]));
    output.add(10).write(v[15]);
    output.add(11).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[7]));
    output.add(12).write(v[5]);
    output.add(13).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[13]));
    output.add(14).write(v[9]);
    output.add(15).write(_mm256_sub_epi32(_mm256_setzero_si256(), v[1]));
  } else {
    let log_range_out = cmp::max(16, bd + 6);
    let clamp_lo_out = _mm256_set1_epi32(-(1 << (log_range_out - 1)));
    let clamp_hi_out = _mm256_set1_epi32((1 << (log_range_out - 1)) - 1);

    neg_shift_avx2(
      v[0],
      v[8],
      output.add(0).as_mut().unwrap(),
      output.add(1).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[12],
      v[4],
      output.add(2).as_mut().unwrap(),
      output.add(3).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[6],
      v[14],
      output.add(4).as_mut().unwrap(),
      output.add(5).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[10],
      v[2],
      output.add(6).as_mut().unwrap(),
      output.add(7).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[3],
      v[11],
      output.add(8).as_mut().unwrap(),
      output.add(9).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[15],
      v[7],
      output.add(10).as_mut().unwrap(),
      output.add(11).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[5],
      v[13],
      output.add(12).as_mut().unwrap(),
      output.add(13).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
    neg_shift_avx2(
      v[9],
      v[1],
      output.add(14).as_mut().unwrap(),
      output.add(15).as_mut().unwrap(),
      clamp_lo_out,
      clamp_hi_out,
      out_shift,
    );
  }
}
