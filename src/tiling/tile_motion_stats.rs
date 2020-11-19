// Copyright (c) 2019-2020, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

use crate::me::*;

use std::marker::PhantomData;
use std::ops::Index;
use std::slice;

/// Tiled view of FrameMEStats
#[derive(Debug)]
pub struct TileMEStats<'a> {
  data: *const MEStats,
  // expressed in mi blocks
  // private to guarantee borrowing rules
  x: usize,
  y: usize,
  cols: usize,
  rows: usize,
  stride: usize, // number of cols in the underlying FrameMEStats
  phantom: PhantomData<&'a AtomicMotionVector>,
}

impl<'a> TileMEStats<'a> {
  #[inline(always)]
  pub fn new(
    frame_mvs: &'a FrameMEStats, x: usize, y: usize, cols: usize, rows: usize,
  ) -> Self {
    assert!(x + cols <= frame_mvs.cols);
    assert!(y + rows <= frame_mvs.rows);
    Self {
      data: &frame_mvs[y][x],
      x,
      y,
      cols,
      rows,
      stride: frame_mvs.cols,
      phantom: PhantomData,
    }
  }

  #[inline(always)]
  pub const fn x(&self) -> usize {
    self.x
  }

  #[inline(always)]
  pub const fn y(&self) -> usize {
    self.y
  }

  #[inline(always)]
  pub const fn cols(&self) -> usize {
    self.cols
  }

  #[inline(always)]
  pub const fn rows(&self) -> usize {
    self.rows
  }
}

unsafe impl Send for TileMEStats<'_> {}
unsafe impl Sync for TileMEStats<'_> {}

impl Index<usize> for TileMEStats<'_> {
  type Output = [MEStats];

  #[inline(always)]
  fn index(&self, index: usize) -> &Self::Output {
    assert!(index < self.rows);
    unsafe {
      let ptr = self.data.add(index * self.stride);
      slice::from_raw_parts(ptr, self.cols)
    }
  }
}
