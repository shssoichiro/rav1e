// Copyright (c) 2017-2020, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

use crate::frame::*;
use crate::rdo::{ssim_boost, DistortionScale};
use crate::tiling::*;
use crate::util::*;
use itertools::izip;
use rust_hawktracer::*;

#[derive(Debug, Default, Clone)]
pub struct ActivityMask {
  pub variances: Box<[u32]>,
  // Width and height of the original frame that is masked
  width: usize,
  height: usize,
}

impl ActivityMask {
  #[hawktracer(activity_mask_from_plane)]
  pub fn from_plane<T: Pixel>(luma_plane: &Plane<T>) -> ActivityMask {
    let PlaneConfig { width, height, .. } = luma_plane.cfg;

    // Width and height are padded to 8×8 block size.
    let w_in_imp_b = width.align_power_of_two_and_shift(3);
    let h_in_imp_b = height.align_power_of_two_and_shift(3);

    let aligned_luma = Rect {
      x: 0_isize,
      y: 0_isize,
      width: w_in_imp_b << 3,
      height: h_in_imp_b << 3,
    };
    let luma = PlaneRegion::new(luma_plane, aligned_luma);

    let mut variances = Vec::with_capacity(w_in_imp_b * h_in_imp_b);

    for y in 0..h_in_imp_b {
      for x in 0..w_in_imp_b {
        let block_rect = Area::Rect {
          x: (x << 3) as isize,
          y: (y << 3) as isize,
          width: 8,
          height: 8,
        };

        let block = luma.subregion(block_rect);
        let variance = variance_8x8(&block);
        variances.push(variance);
      }
    }
    ActivityMask { variances: variances.into_boxed_slice(), width, height }
  }

  #[hawktracer(activity_mask_fill_scales)]
  pub fn fill_scales(
    &self, bit_depth: usize, activity_scales: &mut Box<[DistortionScale]>,
  ) {
    for (dst, &src) in activity_scales.iter_mut().zip(self.variances.iter()) {
      *dst = ssim_boost(src as i64, src as i64, bit_depth);
    }
  }
}

macro_rules! variance {
  ($($SIZE:expr),*) => {
    $(
      paste::item! {
        // Adapted from the source variance calculation in cdef_dist_wxh_8x8.
        pub(crate) fn [<variance_ $SIZE x $SIZE>]<T: Pixel>(src: &PlaneRegion<'_, T>) -> u32 {
          debug_assert!(src.plane_cfg.xdec == 0);
          debug_assert!(src.plane_cfg.ydec == 0);

          // Sum into columns to improve auto-vectorization
          let mut sum_s_cols: [u16; $SIZE] = [0; $SIZE];
          let mut sum_s2_cols: [u32; $SIZE] = [0; $SIZE];

          // Check upfront that $SIZE rows are available.
          let _row = &src[$SIZE - 1];

          for j in 0..$SIZE {
            let row = &src[j][0..$SIZE];
            for (sum_s, sum_s2, s) in izip!(&mut sum_s_cols, &mut sum_s2_cols, row) {
              // Don't convert directly to u32 to allow better vectorization
              let s: u16 = u16::cast_from(*s);
              *sum_s += s;

              // Convert to u32 to avoid overflows when multiplying
              let s: u32 = s as u32;
              *sum_s2 += s * s;
            }
          }

          // 8x8 can safely use u32 to be faster
          // Branching is optimized away at compile time
          if $SIZE == 8 {
            // Sum together the sum of columns
            let sum_s = sum_s_cols.iter().map(|&a| a as u32).sum::<u32>();
            let sum_s2 = sum_s2_cols.iter().map(|&a| a as u32).sum::<u32>();

            // Use sums to calculate variance
            sum_s2 - ((sum_s * sum_s + 32) >> 6)
          } else {
            // Sum together the sum of columns
            let sum_s = sum_s_cols.iter().map(|&a| a as u64).sum::<u64>();
            let sum_s2 = sum_s2_cols.iter().map(|&a| a as u64).sum::<u64>();

            // Use sums to calculate variance
            let result = sum_s2 - ((sum_s * sum_s + 32) >> ($SIZE - 2));
            debug_assert!(result <= u32::MAX as u64);
            result as u32
          }
        }
      }
    )*
  }
}

variance!(8, 16, 32, 64);
