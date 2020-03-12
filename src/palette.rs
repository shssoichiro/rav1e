use crate::context::BlockOffset;
use crate::encoder::FrameInvariants;
use crate::partition::BlockSize;
use crate::rdo::{RawDistortion, ScaledDistortion};
use crate::tiling::TileStateMut;
use arrayvec::ArrayVec;
use std::cmp;
use v_frame::frame::Frame;
use v_frame::math::clamp;
use v_frame::pixel::{CastFromPrimitive, Pixel};
use v_frame::plane::Plane;

const PALETTE_MAX_SIZE: usize = 8;
const PALETTE_MIN_SIZE: usize = 2;

type ColorCache<T> = ArrayVec<[T; 2 * PALETTE_MAX_SIZE]>;
type ColorCounts = [usize; 1 << 12];
type ColorMap = ArrayVec<[u8; 64 * 64]>;
type Centroids = [usize; PALETTE_MAX_SIZE];

#[derive(Debug, Clone, Default)]
pub(crate) struct PaletteModeInfo<T: Pixel> {
  /// Value of base colors for Y, U, and V
  pub palette_colors: [T; 3 * PALETTE_MAX_SIZE],
  /// Number of base colors for Y (0) and UV (1)
  pub palette_size: [u8; 2],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaletteSearchLevel {
  None,
  /// Perform 2 way palette search from max colors to min colors (and min
  /// colors to remaining colors) and terminate the search if current number of
  /// palette colors is not the winner.
  FastSearch,
  /// Perform coarse search to prune the palette colors. For winner colors,
  /// neighbors are also evaluated using a finer search.
  CoarseSearch,
  /// No pruning
  FullSearch,
}

struct PaletteSearcher<'a, T: Pixel> {
  // Internally-generated items
  best_rd: u64,
  beat_best_rd: bool,
  rate: u64,
  rate_tokenonly: u64,
  distortion: RawDistortion,
  skippable: bool,
  color_cache: ColorCache<T>,
  palette_mode_info: PaletteModeInfo<T>,
  // External items
  input: &'a Frame<T>,
  frame_invariants: &'a FrameInvariants<T>,
  tile_state: &'a mut TileStateMut<'a, T>,
  block_size: BlockSize,
  block_offset: BlockOffset,
  bit_depth: usize,
}

impl<'a, T: Pixel> PaletteSearcher<'a, T> {
  fn new(
    input: &'a Frame<T>, frame_invariants: &'a FrameInvariants<T>,
    tile_state: &'a mut TileStateMut<'a, T>, block_size: BlockSize,
    block_offset: BlockOffset, bit_depth: usize,
  ) -> Self {
    PaletteSearcher {
      best_rd: u64::max_value(),
      beat_best_rd: false,
      rate: u64::max_value(),
      rate_tokenonly: u64::max_value(),
      distortion: RawDistortion::new(u64::max_value()),
      skippable: false,
      color_cache: ArrayVec::new(),
      palette_mode_info: Default::default(),
      input,
      frame_invariants,
      tile_state,
      block_size,
      block_offset,
      bit_depth,
    }
  }

  fn pick_palette_intra_luma(&mut self) -> Option<PaletteInfo> {
    assert!(frame_invariants.frame_type.all_intra());

    let plane = &self.input.planes[0];
    let mut color_counts =
      Self::count_colors(plane, self.block_size, self.block_offset);
    let colors = color_counts.iter().filter(|&&count| count > 0).count();
    if colors > 64 {
      return None;
    }

    let y_start = self.block_offset.y * self.block_size.height();
    let x_start = self.block_offset.x * self.block_size.width();
    let y_end = y_start + self.block_size.height();
    let x_end = x_start + self.block_size.width();

    let mut k_means_buf = [T::cast_from(0); 2 * 64 * 64];
    let mut lower_bound = plane.data[0];
    let mut upper_bound = plane.data[0];
    for y in y_start..y_end {
      for x in x_start..x_end {
        let val = plane.data[y * plane.cfg.stride + x];
        k_means_buf[y * self.block_size.width() + x] = val;
        lower_bound = cmp::min(lower_bound, val);
        upper_bound = cmp::max(upper_bound, val);
      }
    }

    // Find the dominant colors and store them in `top_colors`
    let mut top_colors = [0; PALETTE_MAX_SIZE];
    for i in 0..cmp::min(colors, PALETTE_MAX_SIZE) {
      let mut max_count = 0;
      for j in 0..(1 << self.bit_depth) {
        if color_counts[j] > max_count {
          max_count = color_counts[j];
          top_colors[i] = j;
        }
      }
      assert!(max_count > 0);
      color_counts[top_colors[i]] = 0;
    }

    let mut centroids = [0; PALETTE_MAX_SIZE];
    let color_cache = Self::get_palette_cache();

    // Try the dominant colors directly.
    // TODO: (from aom) Try to avoid duplicate computation in cases
    // where the dominant colors and the k-means results are similar.
    if self.frame_invariants.config.speed_settings.palette_search_level
      == PaletteSearchLevel::CoarseSearch
      && colors > PALETTE_MIN_SIZE
    {
      // Choose the start index and step size for coarse search based on number
      // of colors
      let end_n = cmp::min(colors, PALETTE_MAX_SIZE);
      let start_n = Self::get_start_n(end_n);
      let step_size = Self::get_step_size(end_n);

      // Perform top color coarse palette search to find the winner candidate
      let top_color_winner = self.do_top_color_coarse_palette_search(
        &top_colors,
        start_n,
        end_n,
        step_size,
      );
      // Evaluate neighbors for the winner color (if winner is found) in the
      // above coarse search for dominant colors
      if top_color_winner <= end_n {
        let start_n_stage2 = Self::get_start_n_stage2(top_color_winner);
        let end_n_stage2 = Self::get_end_n_stage2(top_color_winner, end_n);
        let step_size_stage2 = end_n_stage2 - start_n_stage2;
        // perform finer search for the winner candidate
        self.do_top_color_palette_search(
          &top_colors,
          start_n_stage2,
          end_n_stage2 + step_size_stage2,
          step_size_stage2,
        );
      }

      // K-means clustering.
      // Perform k-means coarse palette search to find the winner candidate
      let k_means_winner = self.do_k_means_coarse_palette_search(
        &mut plane_palette.color_map,
        start_n,
        end_n,
        step_size,
        lower_bound,
        upper_bound,
      );
      // Evaluate neighbors for the winner color (if winner is found) in the
      // above coarse search for k-means
      if k_means_winner <= end_n {
        let start_n_stage2 = Self::get_start_n_stage2(k_means_winner);
        let end_n_stage2 = Self::get_end_n_stage2(k_means_winner, end_n);
        let step_size_stage2 = end_n_stage2 - start_n_stage2;
        // perform finer search for the winner candidate
        self.do_k_means_palette_search(
          &mut plane_palette.color_map,
          start_n_stage2,
          end_n_stage2 + step_size_stage2,
          step_size_stage2,
          lower_bound,
          upper_bound,
        );
      }
    } else {
      let start_n = cmp::min(colors, PALETTE_MAX_SIZE);
      let end_n = PALETTE_MIN_SIZE;

      // Perform top color palette search from start_n
      let top_color_winner =
        self.do_top_color_palette_search(&top_colors, start_n, end_n - 1, -1);
      if top_color_winner > end_n {
        // Perform top color palette search in reverse order for the remaining
        // colors
        self.do_top_color_palette_search(
          &top_colors,
          send_n,
          top_color_winner,
          1,
        );
      }

      // K-means clustering.
      if colors == PALETTE_MIN_SIZE {
        // Special case: These colors automatically become the centroids.
        centroids[0] = lower_bound;
        centroids[1] = upper_bound;
        self.palette_rd_luma(&mut centroids, colors);
      } else {
        // Perform k-means palette search from start_n
        let k_means_winner = self.do_k_means_palette_search(
          &mut plane_palette.color_map,
          start_n,
          end_n - 1,
          -1,
          lower_bound,
          upper_bound,
        );
        if k_means_winner > end_n {
          // Perform k-means palette search in reverse order for the remaining
          // colors
          self.do_k_means_palette_search(
            &mut plane_palette.color_map,
            end_n,
            k_means_winner,
            1,
            lower_bound,
            upper_bound,
          );
        }
      }
    }
    todo!("intra_mode_search.c:884")
  }

  fn pick_palette_intra_chroma(&self) -> Option<PaletteInfo> {
    assert!(self.frame_invariants.frame_type.all_intra());

    todo!("intra_mode_search.c:998")
  }

  fn do_top_color_coarse_palette_search(
    &mut self, top_colors: &[usize], start_n: usize, end_n: usize,
    step_size: usize,
  ) -> usize {
    let mut centroids = [0; PALETTE_MAX_SIZE];
    let mut n = start_n;
    let mut top_color_winner = end_n + 1;

    loop {
      self.beat_best_rd = false;
      centroids[..n].copy_from_slice(&top_colors[..n]);
      self.palette_rd_luma(&mut centroids, n);

      if self.beat_best_rd {
        top_color_winner = n;
      }
      n += step_size;
      if n > end_n {
        break;
      }
    }

    top_color_winner
  }

  fn do_k_means_coarse_palette_search(
    &mut self, color_map: &mut ColorMap, start_n: usize, end_n: usize,
    step_size: usize, lower_bound: usize, upper_bound: usize,
  ) -> usize {
    let mut centroids = [0; PALETTE_MAX_SIZE];
    const MAX_ITERATIONS: usize = 50;
    let mut n = start_n;
    let mut k_means_winner = end_n + 1;

    loop {
      self.beat_best_rd = false;
      for i in 0..n {
        centroids[i] =
          lower_bound + (2 * i + 1) * (upper_bound - lower_bound) / n / 2;
      }
      calculate_k_means(
        data,
        &mut centroids,
        color_map,
        data_points,
        n,
        1,
        MAX_ITERATIONS,
      );
      self.palette_rd_luma(&mut centroids, n);

      if self.beat_best_rd {
        k_means_winner = n;
      }
      n += step_size;
      if n > end_n {
        break;
      }
    }

    k_means_winner
  }

  fn do_top_color_palette_search(
    &self, top_colors: &[usize], start_n: usize, end_n: usize,
    step_size: usize,
  ) -> usize {
    todo!("intra_mode_search.c:693")
  }

  fn do_k_means_palette_search(
    &self, color_map: &mut ColorMap, start_n: usize, end_n: usize,
    step_size: usize, lower_bound: usize, upper_bound: usize,
  ) -> usize {
    todo!("intra_mode_search.c:726")
  }

  fn palette_rd_luma(
    &mut self, centroids: &mut Centroids, num_colors: usize,
  ) -> Option<PaletteModeInfo<T>> {
    self.optimize_palette_colors(centroids, num_colors, 1);
    let num_unique = Self::remove_duplicates(centroids, num_colors);
    if num_unique < PALETTE_MIN_SIZE {
      // Too few unique colors to create a palette. And DC_PRED will work
      // well for that case anyway. So skip.
      return None;
    }

    for i in 0..num_unique {
      self.palette_mode_info.palette_colors[i] =
        clamp(centroids[i], 0, 1 << self.bit_depth);
    }
    self.palette_mode_info.palette_size[0] = num_unique as u8;
    todo!("intra_mode_search.c:575");
    //   MACROBLOCKD *const xd = &x->e_mbd;
    //   uint8_t *const color_map = xd->plane[0].color_index_map;
    //   int block_width, block_height, rows, cols;
    //   av1_get_block_dimensions(bsize, 0, xd, &block_width, &block_height, &rows,
    //                            &cols);
    //   av1_calc_indices(data, centroids, color_map, rows * cols, k, 1);
    //   extend_palette_color_map(color_map, cols, rows, block_width, block_height);
    //
    //   const int palette_mode_cost =
    //       intra_mode_info_cost_y(cpi, x, mbmi, bsize, dc_mode_cost);
    //   if (model_intra_yrd_and_prune(cpi, x, bsize, palette_mode_cost,
    //                                 best_model_rd)) {
    //     return;
    //   }
    //
    //   RD_STATS tokenonly_rd_stats;
    //   av1_super_block_yrd(cpi, x, &tokenonly_rd_stats, bsize, *best_rd);
    //   if (tokenonly_rd_stats.rate == INT_MAX) return;
    //   int this_rate = tokenonly_rd_stats.rate + palette_mode_cost;
    //   int64_t this_rd = RDCOST(x->rdmult, this_rate, tokenonly_rd_stats.dist);
    //   if (!xd->lossless[mbmi->segment_id] && block_signals_txsize(mbmi->sb_type)) {
    //     tokenonly_rd_stats.rate -= tx_size_cost(x, bsize, mbmi->tx_size);
    //   }
    //   // Collect mode stats for multiwinner mode processing
    //   const int txfm_search_done = 1;
    //   store_winner_mode_stats(
    //       &cpi->common, x, mbmi, NULL, NULL, NULL, THR_DC, color_map, bsize,
    //       this_rd, cpi->sf.winner_mode_sf.enable_multiwinner_mode_process,
    //       txfm_search_done);
    //   if (this_rd < *best_rd) {
    //     *best_rd = this_rd;
    //     // Setting beat_best_rd flag because current mode rd is better than best_rd.
    //     // This flag need to be updated only for palette evaluation in key frames
    //     if (beat_best_rd) *beat_best_rd = 1;
    //     memcpy(best_palette_color_map, color_map,
    //            block_width * block_height * sizeof(color_map[0]));
    //     *best_mbmi = *mbmi;
    //     memcpy(blk_skip, x->blk_skip, sizeof(x->blk_skip[0]) * ctx->num_4x4_blk);
    //     av1_copy_array(tx_type_map, xd->tx_type_map, ctx->num_4x4_blk);
    //     if (rate) *rate = this_rate;
    //     if (rate_tokenonly) *rate_tokenonly = tokenonly_rd_stats.rate;
    //     if (distortion) *distortion = tokenonly_rd_stats.dist;
    //     if (skippable) *skippable = tokenonly_rd_stats.skip;
    //     if (beat_best_pallette_rd) *beat_best_pallette_rd = 1;
    //   }
  }

  fn count_colors(
    plane: &Plane<T>, block_size: BlockSize, block_offset: BlockOffset,
  ) -> ColorCounts {
    let mut counts = [0; 1 << 12];

    let y_start = block_offset.y * block_size.height();
    let x_start = block_offset.x * block_size.width();
    let y_end = y_start + block_size.height();
    let x_end = x_start + block_size.width();

    for y in y_start..y_end {
      for x in x_start..x_end {
        let val = plane.data[y * plane.cfg.stride + x];
        counts[usize::cast_from(val)] += 1;
      }
    }

    counts
  }

  fn get_palette_cache() -> ColorCache<T> {
    let mut cache = ArrayVec::new();
    todo!("pred_common.c:73")
  }

  fn optimize_palette_colors(
    &self, centroids: &mut Centroids, num_colors: usize, stride: usize,
  ) {
    if self.color_cache.is_empty() {
      return;
    }

    for i in (0..(num_colors * stride)).step_by(stride) {
      let mut min_diff =
        (centroids[i] as i32 - i32::cast_from(self.color_cache[0])).abs();
      let mut idx = 0;

      for j in 1..self.color_cache.len() {
        let this_diff =
          (centroids[i] as i32 - i32::cast_from(self.color_cache[j])).abs();
        if this_diff < min_diff {
          min_diff = this_diff;
          idx = j;
        }
      }

      if min_diff <= 1 {
        centroids[i] = usize::cast_from(self.color_cache[idx]);
      }
    }
  }

  /// Returns the number of unique elements
  fn remove_duplicates(
    centroids: &mut Centroids, num_centroids: usize,
  ) -> usize {
    centroids.sort_unstable();
    let mut num_unique = 1;
    for i in 1..num_centroids {
      if centroids[i] != centroids[i - 1] {
        centroids[num_unique] = centroids[i];
        num_unique += 1;
      }
    }
    num_unique
  }

  // Start index and step size below are chosen to evaluate unique
  // candidates in neighbor search, in case a winner candidate is found in
  // coarse search. Example,
  // 1) 8 colors (end_n = 8): 2,3,4,5,6,7,8. start_n is chosen as 2 and step
  // size is chosen as 3. Therefore, coarse search will evaluate 2, 5 and 8.
  // If winner is found at 5, then 4 and 6 are evaluated. Similarly, for 2
  // (3) and 8 (7).
  // 2) 7 colors (end_n = 7): 2,3,4,5,6,7. If start_n is chosen as 2 (same
  // as for 8 colors) then step size should also be 2, to cover all
  // candidates. Coarse search will evaluate 2, 4 and 6. If winner is either
  // 2 or 4, 3 will be evaluated. Instead, if start_n=3 and step_size=3,
  // coarse search will evaluate 3 and 6. For the winner, unique neighbors
  // (3: 2,4 or 6: 5,7) would be evaluated.

  #[inline(always)]
  fn get_start_n(end_n: usize) -> usize {
    match end_n {
      0..=2 => 0,
      3..=4 | 6..=7 => 3,
      5 | 8 => 2,
      _ => unreachable!(),
    }
  }

  #[inline(always)]
  fn get_step_size(end_n: usize) -> usize {
    match end_n {
      0..=2 => 0,
      3..=8 => 3,
      _ => unreachable!(),
    }
  }

  #[inline(always)]
  fn get_start_n_stage2(winner: usize) -> usize {
    if winner == PALETTE_MIN_SIZE {
      PALETTE_MIN_SIZE + 1
    } else {
      cmp::max(x - 1, PALETTE_MIN_SIZE)
    }
  }

  #[inline(always)]
  fn get_end_n_stage2(winner: usize, end_n: usize) -> usize {
    if winner == end_n {
      x - 1
    } else {
      cmp::min(x + 1, PALETTE_MAX_SIZE)
    }
  }
}

pub(crate) fn search_palette_mode<T: Pixel>(
  input: &Frame<T>, frame_invariants: &FrameInvariants<T>,
  tile_state: &mut TileStateMut<'_, T>, block_size: BlockSize,
  block_offset: BlockOffset, bit_depth: usize,
) -> Option<PaletteInfo> {
  if !enable_palette_mode(frame_invariants, block_size) {
    return None;
  }

  let mut searcher = PaletteSearcher::new(
    input,
    frame_invariants,
    tile_state,
    block_size,
    block_offset,
    bit_depth,
  );
  let palette_y = searcher.pick_palette_intra_luma();
  if palette_y.is_none() {
    return None;
  }

  let palette_uv = if input.planes.len() > 1 {
    searcher.pick_palette_intra_chroma()
  } else {
    None
  };

  todo!("intra_mode_search.c:1549")
  // if (skippable) {
  //   rate2 -= rd_stats_y.rate;
  //   if (num_planes > 1) rate2 -= intra_search_state->rate_uv_tokenonly;
  //   rate2 += x->skip_cost[av1_get_skip_context(xd)][1];
  // } else {
  //   rate2 += x->skip_cost[av1_get_skip_context(xd)][0];
  // }
  // this_rd = RDCOST(x->rdmult, rate2, distortion2);
  // this_rd_cost->rate = rate2;
  // this_rd_cost->dist = distortion2;
  // this_rd_cost->rdcost = this_rd;
  // return skippable;
}

fn enable_palette_mode(
  frame_invariants: &FrameInvariants<T>, block_size: BlockSize,
) -> bool {
  frame_invariants.config.speed_settings.palette_search_level
    != PaletteSearchLevel::None
    && allow_palette_mode(frame_invariants, block_size)
}

fn allow_palette_mode<T: Pixel>(
  frame_invariants: &FrameInvariants<T>, block_size: BlockSize,
) -> bool {
  frame_invariants.allow_screen_content_tools
    && block_size.width() <= 64
    && block_size.height() <= 64
    && block_size.width() >= 8
    && block_size.height() >= 8
}
