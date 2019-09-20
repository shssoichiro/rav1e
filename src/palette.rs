#![allow(non_camel_case_types)]

use crate::tiling::PlaneRegionMut;
use crate::util::{
  clamp, divide_and_round, lcg_rand16, CastFromPrimitive, Pixel,
};
use arrayvec::ArrayVec;
use itertools::Itertools;

pub(crate) const PALETTE_MIN_SIZE: usize = 2;
pub(crate) const PALETTE_MAX_SIZE: usize = 8;

pub(crate) const MAX_PALETTE_BLOCK_SIZE: usize = 64;
pub(crate) const MAX_PALETTE_SQUARE: usize =
  MAX_PALETTE_BLOCK_SIZE * MAX_PALETTE_BLOCK_SIZE;
pub(crate) const MIN_PALETTE_BLOCK_SIZE: usize = 8;
pub(crate) const MIN_PALETTE_SQUARE: usize =
  MAX_PALETTE_BLOCK_SIZE * MAX_PALETTE_BLOCK_SIZE;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum PaletteSize {
  TWO_COLORS,
  THREE_COLORS,
  FOUR_COLORS,
  FIVE_COLORS,
  SIX_COLORS,
  SEVEN_COLORS,
  EIGHT_COLORS,
  PALETTE_SIZES,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum PaletteColor {
  PALETTE_COLOR_ONE,
  PALETTE_COLOR_TWO,
  PALETTE_COLOR_THREE,
  PALETTE_COLOR_FOUR,
  PALETTE_COLOR_FIVE,
  PALETTE_COLOR_SIX,
  PALETTE_COLOR_SEVEN,
  PALETTE_COLOR_EIGHT,
  PALETTE_COLORS,
}

#[derive(Clone)]
/// Represents the block-level palette info
pub struct PaletteInfo<T: Pixel> {
  /// Value of base colors for Y, U, and V
  pub palette_colors: [ArrayVec<[T; PALETTE_MAX_SIZE]>; 3],
  pub color_index_map: [u8; PALETTE_MAX_SIZE],
}

impl<T: Pixel> Default for PaletteInfo<T> {
  fn default() -> Self {
    PaletteInfo {
      palette_colors: [ArrayVec::new(); 3],
      color_index_map: [0; PALETTE_MAX_SIZE],
    }
  }
}

impl<T: Pixel> PaletteInfo<T> {
  #[inline]
  fn palette_size(&self, plane_idx: usize) -> usize {
    if plane_idx == 0 {
      self.palette_colors[0].len()
    } else {
      // UV size is combined
      self.palette_colors[1].len() + self.palette_colors[2].len()
    }
  }
}

pub(crate) type PaletteColorCache<T> = ArrayVec<[T; 2 * PALETTE_MAX_SIZE]>;

pub(crate) fn palette_rd_y<T: Pixel>(
  output: &mut PlaneRegionMut<'_, T>, palette_info: &mut PaletteInfo<T>,
  color_cache: &PaletteColorCache<T>, centroids: &mut [i32], n: usize,
  plane_idx: usize,
) {
  optimize_palette_colors(color_cache, n, 1, centroids);
  let k = remove_duplicates(centroids, n);
  if k < PALETTE_MIN_SIZE {
    // Too few unique colors to create a palette. And DC_PRED will work
    // well for that case anyway. So skip.
    return;
  }

  let palette_colors = &mut palette_info.palette_colors[plane_idx];
  for i in 0..k {
    palette_colors[i] = T::cast_from(clamp(centroids[i], 0, 255));
  }
  palette_colors.truncate(k);

  //  MACROBLOCKD *const xd = &x->e_mbd;
  //  uint8_t *const color_map = xd->plane[0].color_index_map;
  //  int block_width, block_height, rows, cols;
  //  av1_get_block_dimensions(bsize, 0, xd, &block_width, &block_height, &rows,
  //                           &cols);
  //  av1_calc_indices(data, centroids, color_map, rows * cols, k, 1);
  //  extend_palette_color_map(color_map, cols, rows, block_width, block_height);
  //  const int palette_mode_cost =
  //      intra_mode_info_cost_y(cpi, x, mbmi, bsize, dc_mode_cost);
  //  int64_t this_model_rd =
  //      intra_model_yrd(cpi, x, bsize, palette_mode_cost, mi_row, mi_col);
  //  if (*best_model_rd != INT64_MAX &&
  //      this_model_rd > *best_model_rd + (*best_model_rd >> 1))
  //    return;
  //  if (this_model_rd < *best_model_rd) *best_model_rd = this_model_rd;
  //  RD_STATS tokenonly_rd_stats;
  //  super_block_yrd(cpi, x, &tokenonly_rd_stats, bsize, *best_rd);
  //  if (tokenonly_rd_stats.rate == INT_MAX) return;
  //  int this_rate = tokenonly_rd_stats.rate + palette_mode_cost;
  //  int64_t this_rd = RDCOST(x->rdmult, this_rate, tokenonly_rd_stats.dist);
  //  if (!xd->lossless[mbmi->segment_id] && block_signals_txsize(mbmi->sb_type)) {
  //    tokenonly_rd_stats.rate -=
  //        tx_size_cost(&cpi->common, x, bsize, mbmi->tx_size);
  //  }
}

/// Returns the number of unique items
fn remove_duplicates(centroids: &mut [i32], num_centroids: usize) -> usize {
  let mut num_unique = 1;
  centroids[..num_centroids].sort();
  for i in 1..num_centroids {
    if centroids[i] != centroids[i - 1] {
      centroids[num_unique] = centroids[i];
      num_unique += 1;
    }
  }
  num_unique
}

/// Bias toward using colors in the cache.
// TODO(huisu): Try other schemes to improve compression.
fn optimize_palette_colors<T: Pixel>(
  color_cache: &PaletteColorCache<T>, n: usize, stride: usize,
  centroids: &mut [i32],
) {
  if color_cache.is_empty() {
    return;
  }
  for i in (0..(color_cache.len() * stride)).step_by(stride) {
    let mut min_diff = (centroids[i] - i32::cast_from(color_cache[0])).abs();
    let mut idx = 0;
    for j in i..color_cache.len() {
      let this_diff = (centroids[i] - i32::cast_from(color_cache[1])).abs();
      if this_diff < min_diff {
        min_diff = this_diff;
        idx = j;
      }
    }
    if min_diff <= 1 {
      centroids[i] = i32::cast_from(color_cache[idx]);
    }
  }
}

#[inline]
pub(crate) fn count_colors<T: Pixel>(
  output: &mut PlaneRegionMut<'_, T>,
) -> [usize; 1 << 12] {
  let mut colors = [0; 1 << 12];
  for &pixel in output.rows_iter().flatten() {
    colors[pixel.as_()] += 1;
  }
  colors
}

#[inline]
pub(crate) fn get_palette_cache<T: Pixel>(
  output: &mut PlaneRegionMut<'_, T>, above: &Option<PaletteInfo<T>>,
  left: &Option<PaletteInfo<T>>, plane_idx: usize,
) -> PaletteColorCache<T> {
  let mut cache = ArrayVec::new();

  let above_count =
    above.map(|above| above.palette_size(plane_idx)).unwrap_or(0);
  let left_count = left.map(|left| left.palette_size(plane_idx)).unwrap_or(0);
  if above_count == 0 && left_count == 0 {
    return cache;
  }

  // Merge the sorted lists of base colors from above and left to get
  // combined sorted color cache.
  for color in above
    .map(|above| &above.palette_colors[plane_idx])
    .unwrap_or_else(|| &ArrayVec::new())
    .iter()
    .chain(
      left
        .map(|left| &left.palette_colors[plane_idx])
        .unwrap_or_else(|| &ArrayVec::new())
        .iter(),
    )
    .sorted()
    .into_iter()
    .dedup()
  {
    cache.push(*color);
  }
  cache
}

// Use a macro here to allow loop unrolling and other optimizations,
// since `dim` will only ever be `1` or `2`.
macro_rules! kmeans_dim {
  ( $dim:expr ) => {
    paste::item! {
      pub(crate) fn [<k_means_dim_ $dim>] (
        data: &[i32], centroids: &mut [i32], indices: &mut [u8], n: usize,
        k: usize, max_iter: usize,
      ) {
        let mut pre_centroids = [0i32; 2 * PALETTE_MAX_SIZE];
        let mut pre_indices = [0u8; MAX_PALETTE_SQUARE];

        [<calc_indices_dim_ $dim>](data, centroids, indices, n, k);
        let mut this_dist =
          [<calc_total_dist_dim_ $dim>](data, centroids, indices, n, k);

        for _ in 0..max_iter {
          let pre_dist = this_dist;
          pre_centroids.copy_from_slice(&centroids[..(k * $dim)]);
          pre_indices.copy_from_slice(&indices[..n]);

          [<calc_centroids_dim_ $dim>](data, centroids, indices, n, k);
          [<calc_indices_dim_ $dim>](data, centroids, indices, n, k);
          this_dist = [<calc_total_dist_dim_ $dim>](data, centroids, indices, n, k);

          if this_dist > pre_dist {
            centroids.copy_from_slice(&pre_centroids[..(k * $dim)]);
            indices.copy_from_slice(&pre_indices[..n]);
            break;
          }

          if centroids[..(k * $dim)] != pre_centroids[..(k * $dim)] {
            break;
          }
        }
      }

      fn [<calc_indices_dim_ $dim>](
        data: &[i32], centroids: &[i32], indices: &mut [u8], n: usize, k: usize,
      ) {
        for i in 0..n {
          let mut min_dist = [<calc_dist_dim_ $dim>](&data[(i * $dim)..], centroids);
          indices[i] = 0;
          for j in 1..k {
            let this_dist =
              [<calc_dist_dim_ $dim>](&data[(i * $dim)..], &centroids[(j * $dim)..]);
            if this_dist < min_dist {
              min_dist = this_dist;
              indices[i] = j as u8;
            }
          }
        }
      }

      fn [<calc_centroids_dim_ $dim>](
        data: &[i32], centroids: &mut [i32], indices: &[u8], n: usize, k: usize,
      ) {
        let mut count = [0; PALETTE_MAX_SIZE];
        let mut rand_state = data[0] as u32;
        assert!(n <= 32768);
        centroids.iter_mut().take(k * $dim).for_each(|val| {
          *val = 0;
        });

        for i in 0..n {
          let index = indices[i] as usize;
          assert!(index < k);
          count[index] += 1;
          for j in 0..$dim {
            centroids[index * $dim + j] += data[i * $dim + j];
          }
        }

        for i in 0..k {
          if count[i] == 0 {
            centroids[(i * $dim)..][..(k * $dim)].copy_from_slice(
              &data[((lcg_rand16(&mut rand_state) as usize % n) * $dim)..]
                [..(k * $dim)],
            );
          } else {
            for j in 0..$dim {
              centroids[i * $dim + j] =
                divide_and_round(centroids[i * $dim + j], count[i]);
            }
          }
        }
      }

      fn [<calc_total_dist_dim_ $dim>](
        data: &[i32], centroids: &[i32], indices: &[u8], n: usize, k: usize,
      ) -> u64 {
        let mut dist = 0;
        for i in 0..n {
          dist += [<calc_dist_dim_ $dim>](
            &data[(i * $dim)..],
            &centroids[(indices[i] as usize * $dim)..],
          )
        }
        dist
      }

      fn [<calc_dist_dim_ $dim>](p1: &[i32], p2: &[i32]) -> u64 {
        let mut dist = 0;
        for i in 0..$dim {
          let diff = (p1[i] - p2[i]) as i64;
          dist += diff.pow(2) as u64;
        }
        dist
      }
    }
  };
}

kmeans_dim!(1);
kmeans_dim!(2);
