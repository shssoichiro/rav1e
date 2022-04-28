// Copyright (c) 2018-2020, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

use crate::math::*;
use crate::pixel::*;
use crate::plane::*;
use crate::serialize::{Deserialize, Serialize};
use std::mem::size_of_val;

// One video frame.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Frame<T: Pixel> {
  /// Planes constituting the frame.
  pub planes: [Plane<T>; 3],
}

impl<T: Pixel> Frame<T> {
  /// Creates a new frame with the given parameters.
  ///
  /// Allocates data for the planes.
  pub fn new_with_padding(
    width: usize, height: usize, chroma_sampling: ChromaSampling,
    superblock_size: usize, min_padding: usize,
  ) -> Self {
    let luma_width = width.align_power_of_two(3);
    let luma_height = height.align_power_of_two(3);
    let padding_shift = size_of_val(&superblock_size) * 8
      - (superblock_size.leading_zeros() + 1) as usize;
    let luma_x_stride = luma_width.align_power_of_two(padding_shift);
    let luma_y_stride = luma_height.align_power_of_two(padding_shift);

    let (chroma_decimation_x, chroma_decimation_y) =
      chroma_sampling.get_decimation().unwrap_or((0, 0));
    let (chroma_width, chroma_height) =
      chroma_sampling.get_chroma_dimensions(luma_width, luma_height);
    let chroma_x_stride = chroma_width.align_power_of_two(padding_shift);
    let chroma_y_stride = chroma_height.align_power_of_two(padding_shift);

    Frame {
      planes: [
        Plane::new(
          luma_width,
          luma_height,
          0,
          0,
          (luma_x_stride - luma_width).max(min_padding),
          (luma_y_stride - luma_height).max(min_padding),
        ),
        Plane::new(
          chroma_width,
          chroma_height,
          chroma_decimation_x,
          chroma_decimation_y,
          (chroma_x_stride - chroma_width).max(min_padding),
          (chroma_y_stride - chroma_height).max(min_padding),
        ),
        Plane::new(
          chroma_width,
          chroma_height,
          chroma_decimation_x,
          chroma_decimation_y,
          (chroma_x_stride - chroma_width).max(min_padding),
          (chroma_y_stride - chroma_height).max(min_padding),
        ),
      ],
    }
  }
}
