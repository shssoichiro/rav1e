// Copyright (c) 2018-2021, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.
#![deny(missing_docs)]

use crate::activity::ActivityMask;
use crate::api::lookahead::*;
use crate::api::{
  size_in_b, EncoderConfig, EncoderStatus, FrameType, Opaque, Packet,
};
use crate::color::ChromaSampling::Cs400;
use crate::dist::get_satd;
use crate::encoder::*;
use crate::frame::*;
use crate::me::FrameMEStats;
use crate::partition::*;
use crate::rate::{
  RCState, FRAME_NSUBTYPES, FRAME_SUBTYPE_I, FRAME_SUBTYPE_P,
  FRAME_SUBTYPE_SEF,
};
use crate::rayon::prelude::*;
use crate::scenechange::SceneChangeDetector;
use crate::stats::EncoderStats;
use crate::tiling::Area;
use crate::util::Pixel;
use rust_hawktracer::*;
use std::cmp;
use std::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "dump_lookahead_data")]
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(feature = "dump_lookahead_data")]
use std::{env, fs};

use super::size_in_imp_b;

/// The set of options that controls frame re-ordering and reference picture
///  selection.
/// The options stored here are invariant over the whole encode.
#[derive(Debug, Clone, Copy)]
pub struct InterConfig {
  /// Whether frame re-ordering is enabled.
  reorder: bool,
  /// Whether P-frames can use multiple references.
  pub(crate) multiref: bool,
  /// The depth of the re-ordering pyramid.
  /// The current code cannot support values larger than 2.
  pub(crate) pyramid_depth: u64,
  /// Number of input frames in group.
  pub(crate) group_input_len: u64,
  /// Number of output frames in group.
  /// This includes both hidden frames and "show existing frame" frames.
  pub(crate) group_output_len: u64,
  /// Interval between consecutive S-frames.
  /// Keyframes reset this interval.
  /// This MUST be a multiple of group_input_len.
  pub(crate) switch_frame_interval: u64,
}

impl InterConfig {
  pub(crate) fn new(enc_config: &EncoderConfig) -> InterConfig {
    let reorder = !enc_config.low_latency;
    // A group always starts with (group_output_len - group_input_len) hidden
    //  frames, followed by group_input_len shown frames.
    // The shown frames iterate over the input frames in order, with frames
    //  already encoded as hidden frames now displayed with Show Existing
    //  Frame.
    // For example, for a pyramid depth of 2, the group is as follows:
    //                      |TU         |TU |TU |TU
    // idx_in_group_output:   0   1   2   3   4   5
    // input_frameno:         4   2   1  SEF  3  SEF
    // output_frameno:        1   2   3   4   5   6
    // level:                 0   1   2   1   2   0
    //                        ^^^^^   ^^^^^^^^^^^^^
    //                        hidden      shown
    // TODO: This only works for pyramid_depth <= 2 --- after that we need
    //  more hidden frames in the middle of the group.
    let pyramid_depth = if reorder { 2 } else { 0 };
    let group_input_len = 1 << pyramid_depth;
    let group_output_len = group_input_len + pyramid_depth;
    let switch_frame_interval = enc_config.switch_frame_interval;
    assert!(switch_frame_interval % group_input_len == 0);
    InterConfig {
      reorder,
      multiref: enc_config.multiref(),
      pyramid_depth,
      group_input_len,
      group_output_len,
      switch_frame_interval,
    }
  }

  /// Get the index of an output frame in its re-ordering group given the output
  ///  frame number of the frame in the current keyframe gop.
  /// When re-ordering is disabled, this always returns 0.
  pub(crate) fn get_idx_in_group_output(
    &self, output_frameno_in_gop: u64,
  ) -> u64 {
    // The first frame in the GOP should be a keyframe and is not re-ordered,
    //  so we should not be calling this function on it.
    debug_assert!(output_frameno_in_gop > 0);
    (output_frameno_in_gop - 1) % self.group_output_len
  }

  /// Get the order-hint of an output frame given the output frame number of the
  ///  frame in the current keyframe gop and the index of that output frame
  ///  in its re-ordering gorup.
  pub(crate) fn get_order_hint(
    &self, output_frameno_in_gop: u64, idx_in_group_output: u64,
  ) -> u32 {
    // The first frame in the GOP should be a keyframe, but currently this
    //  function only handles inter frames.
    // We could return 0 for keyframes if keyframe support is needed.
    debug_assert!(output_frameno_in_gop > 0);
    // Which P-frame group in the current gop is this output frame in?
    // Subtract 1 because the first frame in the gop is always a keyframe.
    let group_idx = (output_frameno_in_gop - 1) / self.group_output_len;
    // Get the offset to the corresponding input frame.
    // TODO: This only works with pyramid_depth <= 2.
    let offset = if idx_in_group_output < self.pyramid_depth {
      self.group_input_len >> idx_in_group_output
    } else {
      idx_in_group_output - self.pyramid_depth + 1
    };
    // Construct the final order hint relative to the start of the group.
    (self.group_input_len * group_idx + offset) as u32
  }

  /// This is used primarily for rate control purposes,
  /// where we do not yet have frame invariants built past the current group of frames.
  pub(crate) fn guess_level(&self, idx_in_group_output: u64) -> u64 {
    if !self.reorder {
      0
    } else if idx_in_group_output < self.pyramid_depth {
      // Hidden frames are output first (to be shown in the future).
      idx_in_group_output
    } else {
      // Shown frames
      // TODO: This only works with pyramid_depth <= 2.
      pos_to_lvl(
        idx_in_group_output - self.pyramid_depth + 1,
        self.pyramid_depth,
      )
    }
  }

  pub(crate) fn get_slot_idx(&self, level: u64, order_hint: u32) -> u32 {
    // Frames with level == 0 are stored in slots 0..4, and frames with higher
    //  values of level in slots 4..8
    if level == 0 {
      (order_hint >> self.pyramid_depth) & 3
    } else {
      // This only works with pyramid_depth <= 4.
      3 + level as u32
    }
  }

  pub(crate) const fn get_show_frame(&self, idx_in_group_output: u64) -> bool {
    idx_in_group_output >= self.pyramid_depth
  }

  pub(crate) fn get_show_existing_frame(
    &self, idx_in_group_output: u64,
  ) -> bool {
    // The self.reorder test here is redundant, but short-circuits the rest,
    //  avoiding a bunch of work when it's false.
    self.reorder
      && self.get_show_frame(idx_in_group_output)
      && (idx_in_group_output - self.pyramid_depth + 1).count_ones() == 1
      && idx_in_group_output != self.pyramid_depth
  }

  const fn max_reordering_latency(&self) -> u64 {
    self.group_input_len
  }

  pub(crate) fn keyframe_lookahead_distance(&self) -> u64 {
    self.max_reordering_latency() + 1
  }

  pub(crate) fn allowed_ref_frames(&self) -> &[RefType] {
    use crate::partition::RefType::*;
    if self.reorder {
      &ALL_INTER_REFS
    } else if self.multiref {
      &[LAST_FRAME, LAST2_FRAME, LAST3_FRAME, GOLDEN_FRAME]
    } else {
      &[LAST_FRAME]
    }
  }
}

type FrameQueue<T> = BTreeMap<u64, Option<Arc<Frame<T>>>>;

// the fields pub(super) are accessed only by the tests
pub(crate) struct ContextInner<T: Pixel> {
  pub(crate) config: Arc<EncoderConfig>,
  pub(super) inter_cfg: InterConfig,
  seq: Arc<Sequence>,

  /// The number of input frames read.
  pub(crate) frame_count: u64,
  /// The number of output frames written.
  pub(super) frames_processed: u64,
  /// The maximum number of frames to encode.
  pub(crate) limit: Option<u64>,
  /// The next input frame number to be processed by the encoder.
  pub(crate) input_frameno: u64,
  /// The next output frame number to be processed by the encoder.
  pub(crate) output_frameno: u64,

  /// The next input frame number where we will need to build a new frame group.
  pub(crate) next_group_start_input_frameno: u64,
  /// The output frame number that will be used 1o start the next frame group.
  pub(crate) next_group_start_output_frameno: u64,
  /// Contains the current or most recent keyframe.
  pub(crate) current_keyframe: FrameInvariants<T>,
  /// Contains frame invariants for the current group of frames.
  /// Placeholder frames are indicated by a `None` and should be skipped in decoding.
  /// They exist in this data structure to preserve the output frame number count.
  pub(crate) current_frame_group: Vec<Option<FrameInvariants<T>>>,

  /// Maps *input_frameno* to frames
  pub(super) frame_q: FrameQueue<T>,
  /// A storage space for reordered frames.
  packet_data: Vec<u8>,
  /// Maps *input_frameno* to lookahead data
  lookahead_data: BTreeMap<u64, LookaheadData<T>>,
  /// Optional opaque to be sent back to the user
  opaque_q: BTreeMap<u64, Opaque>,

  keyframe_detector: SceneChangeDetector<T>,
  /// The next frame to be checked for a scenecut by the keyframe detector
  next_keyframe_detector_frame: u64,
  /// A list of the input_frameno for keyframes in this encode.
  keyframes: BTreeSet<u64>,
  keyframes_forced: BTreeSet<u64>,

  pub rc_state: RCState,
  pub maybe_prev_log_base_q: Option<i64>,
}

impl<T: Pixel> ContextInner<T> {
  pub fn new(enc: &EncoderConfig) -> Self {
    // initialize with temporal delimiter
    let packet_data = TEMPORAL_DELIMITER.to_vec();
    let mut keyframes = BTreeSet::new();
    keyframes.insert(0);

    let maybe_ac_qi_max =
      if enc.quantizer < 255 { Some(enc.quantizer as u8) } else { None };

    let seq = Arc::new(Sequence::new(enc));
    let inter_cfg = InterConfig::new(enc);
    let config = Arc::new(*enc);
    let lookahead_distance = inter_cfg.keyframe_lookahead_distance() as usize;
    let current_keyframe = FrameInvariants::new_key_frame(
      Arc::clone(&config),
      Arc::clone(&seq),
      0,
      0,
    );
    let keyframe_detector =
      SceneChangeDetector::new(*enc, lookahead_distance, Arc::clone(&seq));

    ContextInner {
      config,
      inter_cfg,
      seq,
      frame_count: 0,
      frames_processed: 0,
      limit: None,
      input_frameno: 0,
      output_frameno: 0,
      next_group_start_input_frameno: 0,
      next_group_start_output_frameno: 0,
      current_keyframe,
      current_frame_group: Vec::new(),
      frame_q: BTreeMap::new(),
      packet_data,
      opaque_q: BTreeMap::new(),
      keyframe_detector,
      next_keyframe_detector_frame: 1,
      keyframes,
      keyframes_forced: BTreeSet::new(),
      rc_state: RCState::new(
        enc.width as i32,
        enc.height as i32,
        enc.time_base.den as i64,
        enc.time_base.num as i64,
        enc.bitrate,
        maybe_ac_qi_max,
        enc.min_quantizer,
        enc.max_key_frame_interval as i32,
        enc.reservoir_frame_delay,
      ),
      maybe_prev_log_base_q: None,
      lookahead_data: BTreeMap::new(),
    }
  }

  #[hawktracer(send_frame)]
  pub fn send_frame(
    &mut self, mut frame: Option<Arc<Frame<T>>>,
    params: Option<FrameParameters>,
  ) -> Result<(), EncoderStatus> {
    if let Some(ref mut frame) = frame {
      use crate::api::color::ChromaSampling;
      let EncoderConfig { width, height, chroma_sampling, .. } = *self.config;
      let planes =
        if chroma_sampling == ChromaSampling::Cs400 { 1 } else { 3 };
      // Try to add padding
      if let Some(ref mut frame) = Arc::get_mut(frame) {
        for plane in frame.planes[..planes].iter_mut() {
          plane.pad(width, height);
        }
      }
      // Enforce that padding is added
      for (p, plane) in frame.planes[..planes].iter().enumerate() {
        assert!(
          plane.probe_padding(width, height),
          "Plane {} was not padded before passing Frame to send_frame().",
          p
        );
      }
    }

    let input_frameno = self.frame_count;
    self.frame_q.insert(input_frameno, frame);
    if !self.is_flushing() {
      self.frame_count += 1;
    }

    if let Some(params) = params {
      if params.frame_type_override == FrameTypeOverride::Key {
        self.keyframes_forced.insert(input_frameno);
      }
      if let Some(op) = params.opaque {
        self.opaque_q.insert(input_frameno, op);
      }
    }

    // We need to compute scenechanges as the first part of lookahead.
    // Other decisions, such as pyramid decision and motion vector lookahead, depend on this.
    while self.can_compute_scenechange() {
      let lookahead_frames = self
        .frame_q
        .range(self.next_keyframe_detector_frame - 1..)
        .filter_map(|(_, frame)| frame.clone())
        .collect::<Vec<_>>();

      self.compute_keyframe_placement(&lookahead_frames);
    }

    if !self.is_flushing() {
      let (w_in_imp_b, h_in_imp_b) =
        size_in_imp_b(self.config.width, self.config.height);
      self
        .lookahead_data
        .insert(input_frameno, LookaheadData::new(w_in_imp_b, h_in_imp_b));
      if self.config.temporal_rdo() {
        self
          .lookahead_data
          .get_mut(&input_frameno)
          .unwrap()
          .lookahead_intra_costs = self
          .keyframe_detector
          .intra_costs
          .remove(&input_frameno)
          .unwrap_or_else(|| {
            estimate_intra_costs(
              self.frame_q[&input_frameno].as_ref().unwrap(),
              self.config.bit_depth,
              self.config.cpu_feature_level,
            )
          });
      }
    }

    Ok(())
  }

  /// Indicates whether more frames need to be read into the frame queue
  /// in order for frame queue lookahead to be full.
  fn needs_more_lookahead(&self) -> bool {
    let lookahead_end = self.frame_q.keys().last().cloned().unwrap_or(0);
    let frames_needed = self.input_frameno
      + self.config.speed_settings.rdo_lookahead_frames as u64;
    lookahead_end < frames_needed
      && !self.at_frame_limit(lookahead_end)
      && !self.is_flushing()
  }

  fn can_compute_scenechange(&self) -> bool {
    let lookahead_end = self.frame_q.keys().last().cloned().unwrap_or(0);
    let frames_needed = self.next_keyframe_detector_frame
      + self.keyframe_detector.lookahead_offset as u64
      + 1;
    self.next_keyframe_detector_frame < self.frame_count
      && (lookahead_end >= frames_needed
        || self.at_frame_limit(lookahead_end)
        || self.is_flushing())
  }

  pub fn at_frame_limit(&self, frame_count: u64) -> bool {
    self.limit.map(|limit| frame_count >= limit).unwrap_or(false)
  }

  fn is_flushing(&self) -> bool {
    self.frame_q.values().last() == Some(&None)
  }

  fn next_keyframe_input_frameno(
    &self, gop_input_frameno_start: u64, ignore_limit: bool,
  ) -> u64 {
    let next_detected = self
      .keyframes
      .iter()
      .find(|&&input_frameno| input_frameno > gop_input_frameno_start)
      .cloned();
    let mut next_limit =
      gop_input_frameno_start + self.config.max_key_frame_interval;
    if !ignore_limit && self.limit.is_some() {
      next_limit = next_limit.min(self.limit.unwrap());
    }
    if next_detected.is_none() {
      return next_limit;
    }
    cmp::min(next_detected.unwrap(), next_limit)
  }

  fn get_current_fi_mut(&mut self) -> Option<&mut FrameInvariants<T>> {
    if self.output_frameno == self.current_keyframe.output_frameno {
      Some(&mut self.current_keyframe)
    } else {
      let output_frameno = self.output_frameno;
      self
        .current_frame_group
        .iter_mut()
        .find(|fi| {
          fi.as_ref().map(|fi| fi.output_frameno) == Some(output_frameno)
        })
        .and_then(|fi| fi.as_mut())
    }
  }

  pub(crate) fn done_processing(&self) -> bool {
    self.limit.map(|limit| self.frames_processed == limit).unwrap_or(false)
  }

  #[hawktracer(compute_keyframe_placement)]
  pub fn compute_keyframe_placement(
    &mut self, lookahead_frames: &[Arc<Frame<T>>],
  ) {
    if self.keyframes_forced.contains(&self.next_keyframe_detector_frame)
      || self.keyframe_detector.analyze_next_frame(
        lookahead_frames,
        self.next_keyframe_detector_frame,
        *self.keyframes.iter().last().unwrap(),
      )
    {
      self.keyframes.insert(self.next_keyframe_detector_frame);
    }
    self.next_keyframe_detector_frame += 1;
  }

  #[hawktracer(update_block_importances)]
  fn update_block_importances(
    &self, frame: Arc<Frame<T>>, reference_frame: Arc<Frame<T>>,
    ref_frame_count: usize, bsize: BlockSize, w_in_imp_b: usize,
    h_in_imp_b: usize, this_frame_lookahead_data: &mut LookaheadData<T>,
    this_frame_block_importances: &[f32],
    ref_frame_block_importances: &mut [f32], ref_frameno: u64,
  ) {
    let (cols, rows) =
      size_in_b(frame.planes[0].cfg.width, frame.planes[0].cfg.height);
    let buffer = self
      .lookahead_data
      .get(&ref_frameno)
      .and_then(|ld| ld.lookahead_me_stats.clone())
      .unwrap_or_else(|| FrameMEStats::new_arc_array(cols, rows));

    compute_lookahead_motion_vectors(
      Arc::clone(&frame),
      this_frame_lookahead_data,
      buffer,
      *self.config,
      Arc::clone(&self.seq),
      true,
    );
    let stats =
      &this_frame_lookahead_data.lookahead_me_stats.as_ref().unwrap()[0];

    let plane_org = &frame.planes[0];
    let plane_ref = &reference_frame.planes[0];
    let lookahead_intra_costs_lines = this_frame_lookahead_data
      .lookahead_intra_costs
      .par_chunks_exact(w_in_imp_b);
    let block_importances_lines =
      this_frame_block_importances.par_chunks_exact(w_in_imp_b);

    let costs: Vec<_> = lookahead_intra_costs_lines
      .zip(block_importances_lines)
      .enumerate()
      .flat_map_iter(|(y, (lookahead_intra_costs, block_importances))| {
        lookahead_intra_costs
          .iter()
          .zip(block_importances.iter())
          .enumerate()
          .map(move |(x, (&intra_cost, &future_importance))| {
            let mv = stats[y * 2][x * 2].mv;

            // Coordinates of the top-left corner of the reference block, in MV
            // units.
            let reference_x =
              x as i64 * IMP_BLOCK_SIZE_IN_MV_UNITS + mv.col as i64;
            let reference_y =
              y as i64 * IMP_BLOCK_SIZE_IN_MV_UNITS + mv.row as i64;

            let region_org = plane_org.region(Area::Rect {
              x: (x * IMPORTANCE_BLOCK_SIZE) as isize,
              y: (y * IMPORTANCE_BLOCK_SIZE) as isize,
              width: IMPORTANCE_BLOCK_SIZE,
              height: IMPORTANCE_BLOCK_SIZE,
            });

            let region_ref = plane_ref.region(Area::Rect {
              x: reference_x as isize / IMP_BLOCK_MV_UNITS_PER_PIXEL as isize,
              y: reference_y as isize / IMP_BLOCK_MV_UNITS_PER_PIXEL as isize,
              width: IMPORTANCE_BLOCK_SIZE,
              height: IMPORTANCE_BLOCK_SIZE,
            });

            let inter_cost = get_satd(
              &region_org,
              &region_ref,
              bsize.width(),
              bsize.height(),
              self.config.bit_depth,
              self.config.cpu_feature_level,
            ) as f32;

            let intra_cost = intra_cost as f32;
            //          let intra_cost = lookahead_intra_costs[x] as f32;
            //          let future_importance = block_importances[x];

            let propagate_fraction = if intra_cost <= inter_cost {
              0.
            } else {
              1. - inter_cost / intra_cost
            };

            let propagate_amount = (intra_cost + future_importance)
              * propagate_fraction
              / ref_frame_count as f32;
            (propagate_amount, reference_x, reference_y)
          })
      })
      .collect();

    costs.into_iter().for_each(
      |(propagate_amount, reference_x, reference_y)| {
        let mut propagate =
          |block_x_in_mv_units, block_y_in_mv_units, fraction| {
            let x = block_x_in_mv_units / IMP_BLOCK_SIZE_IN_MV_UNITS;
            let y = block_y_in_mv_units / IMP_BLOCK_SIZE_IN_MV_UNITS;

            // TODO: propagate partially if the block is partially off-frame
            // (possible on right and bottom edges)?
            if x >= 0
              && y >= 0
              && (x as usize) < w_in_imp_b
              && (y as usize) < h_in_imp_b
            {
              ref_frame_block_importances
                [y as usize * w_in_imp_b + x as usize] +=
                propagate_amount * fraction;
            }
          };

        // Coordinates of the top-left corner of the block intersecting the
        // reference block from the top-left.
        let top_left_block_x = (reference_x
          - if reference_x < 0 { IMP_BLOCK_SIZE_IN_MV_UNITS - 1 } else { 0 })
          / IMP_BLOCK_SIZE_IN_MV_UNITS
          * IMP_BLOCK_SIZE_IN_MV_UNITS;
        let top_left_block_y = (reference_y
          - if reference_y < 0 { IMP_BLOCK_SIZE_IN_MV_UNITS - 1 } else { 0 })
          / IMP_BLOCK_SIZE_IN_MV_UNITS
          * IMP_BLOCK_SIZE_IN_MV_UNITS;

        debug_assert!(reference_x >= top_left_block_x);
        debug_assert!(reference_y >= top_left_block_y);

        let top_right_block_x = top_left_block_x + IMP_BLOCK_SIZE_IN_MV_UNITS;
        let top_right_block_y = top_left_block_y;
        let bottom_left_block_x = top_left_block_x;
        let bottom_left_block_y =
          top_left_block_y + IMP_BLOCK_SIZE_IN_MV_UNITS;
        let bottom_right_block_x = top_right_block_x;
        let bottom_right_block_y = bottom_left_block_y;

        let top_left_block_fraction = ((top_right_block_x - reference_x)
          * (bottom_left_block_y - reference_y))
          as f32
          / IMP_BLOCK_AREA_IN_MV_UNITS as f32;

        propagate(top_left_block_x, top_left_block_y, top_left_block_fraction);

        let top_right_block_fraction =
          ((reference_x + IMP_BLOCK_SIZE_IN_MV_UNITS - top_right_block_x)
            * (bottom_left_block_y - reference_y)) as f32
            / IMP_BLOCK_AREA_IN_MV_UNITS as f32;

        propagate(
          top_right_block_x,
          top_right_block_y,
          top_right_block_fraction,
        );

        let bottom_left_block_fraction = ((top_right_block_x - reference_x)
          * (reference_y + IMP_BLOCK_SIZE_IN_MV_UNITS - bottom_left_block_y))
          as f32
          / IMP_BLOCK_AREA_IN_MV_UNITS as f32;

        propagate(
          bottom_left_block_x,
          bottom_left_block_y,
          bottom_left_block_fraction,
        );

        let bottom_right_block_fraction =
          ((reference_x + IMP_BLOCK_SIZE_IN_MV_UNITS - top_right_block_x)
            * (reference_y + IMP_BLOCK_SIZE_IN_MV_UNITS - bottom_left_block_y))
            as f32
            / IMP_BLOCK_AREA_IN_MV_UNITS as f32;

        propagate(
          bottom_right_block_x,
          bottom_right_block_y,
          bottom_right_block_fraction,
        );
      },
    );
  }

  /// Computes the block importances for the current frame.
  #[hawktracer(compute_block_importances)]
  fn compute_block_importances(&mut self) {
    // Compute and propagate the block importances from the end. The
    // current frame will get its block importances from the future frames.
    let bsize = BlockSize::from_width_and_height(
      IMPORTANCE_BLOCK_SIZE,
      IMPORTANCE_BLOCK_SIZE,
    );

    // We will evaluate each frame that could potentially reference this one,
    // i.e. any inter frame that is within this GOP.
    let start_frame = self.input_frameno.saturating_sub(2).max(
      self
        .keyframes
        .iter()
        .filter(|kf| **kf < self.input_frameno)
        .last()
        .copied()
        .unwrap_or(0),
    );
    let frame_set = self
      .frame_q
      .range(start_frame..)
      .filter_map(|(no, f)| f.as_ref().map(|f| (no, f)))
      .take_while(|&(no, _)| {
        *no <= self.input_frameno || !self.keyframes.contains(no)
      })
      .collect::<Vec<_>>();

    let (w_in_imp_b, h_in_imp_b) =
      size_in_imp_b(self.config.width, self.config.height);

    let mut block_importances = BTreeMap::new();
    for &(frameno, _) in frame_set.iter() {
      block_importances
        .insert(frameno, vec![0.; w_in_imp_b * h_in_imp_b].into_boxed_slice());
    }

    let frame_ref_list = frame_set
      .iter()
      .rev()
      .take_while(|(&frameno, _)| frameno > self.input_frameno)
      .map(|(&frameno, frame)| {
        (
          frameno,
          frame,
          ((-1i64)..=2)
            .filter_map(|lookback| {
              // Determine which frames can be referenced.
              // We will try to look at the previous 2 and next 1 frame from lookahead purposes.
              // These may not be available.
              if lookback > frameno as i64 {
                return None;
              }
              let ref_frameno = (frameno as i64 - lookback) as u64;
              if lookback == 0
                || ref_frameno > *frame_set[frame_set.len() - 1].0
              {
                None
              } else {
                let ref_frame =
                  self.frame_q.get(&ref_frameno).cloned().flatten();
                ref_frame.map(|ref_frame| (ref_frameno, ref_frame))
              }
            })
            .collect::<Vec<_>>(),
        )
      })
      .collect::<Vec<_>>();
    for (frameno, frame, ref_frame_set) in frame_ref_list {
      // To satisfy the borrow checker, we remove the data for this frame
      // here to modify it and re-insert it at the end of this loop iteration.
      let mut lookahead_data = self.lookahead_data.remove(&frameno).unwrap();
      ref_frame_set.iter().for_each(|(ref_frameno, ref_frame)| {
        self.update_block_importances(
          Arc::clone(frame),
          Arc::clone(ref_frame),
          ref_frame_set.len(),
          bsize,
          w_in_imp_b,
          h_in_imp_b,
          &mut lookahead_data,
          &block_importances[&frameno].clone(),
          block_importances.get_mut(ref_frameno).unwrap(),
          *ref_frameno,
        );

        #[cfg(feature = "dump_lookahead_data")]
        {
          let data_location = build_dump_properties();
          let plane = frame.planes[0].downscale(4);
          let mut file_name = format!("{:010}-qres", frameno);
          let buf: Vec<_> = plane.iter().map(|p| p.as_()).collect();
          image::GrayImage::from_vec(
            plane.cfg.width as u32,
            plane.cfg.height as u32,
            buf,
          )
          .unwrap()
          .save(data_location.join(file_name).with_extension("png"))
          .unwrap();
          let plane = frame.planes[0].downscale(2);
          file_name = format!("{:010}-hres", frameno);
          let buf: Vec<_> = plane.iter().map(|p| p.as_()).collect();
          image::GrayImage::from_vec(
            plane.cfg.width as u32,
            plane.cfg.height as u32,
            buf,
          )
          .unwrap()
          .save(data_location.join(file_name).with_extension("png"))
          .unwrap();

          use crate::partition::RefType::*;
          let data_location = build_dump_properties();
          let file_name = format!("{:010}-mvs", frameno);
          let second_ref_frame = if !self.config.speed_settings.multiref {
            LAST_FRAME // make second_ref_frame match first
          } else if fi.idx_in_group_output == 0 {
            LAST2_FRAME
          } else {
            ALTREF_FRAME
          };

          // Use the default index, it corresponds to the last P-frame or to the
          // backwards lower reference (so the closest previous frame).
          let index = if second_ref_frame.to_index() != 0 { 0 } else { 1 };

          let me_stats = &fs.frame_me_stats[index];
          use byteorder::{NativeEndian, WriteBytesExt};
          // dynamic allocation: debugging only
          let mut buf = vec![];
          buf.write_u64::<NativeEndian>(me_stats.rows as u64).unwrap();
          buf.write_u64::<NativeEndian>(me_stats.cols as u64).unwrap();
          for y in 0..me_stats.rows {
            for x in 0..me_stats.cols {
              let mv = me_stats[y][x].mv;
              buf.write_i16::<NativeEndian>(mv.row).unwrap();
              buf.write_i16::<NativeEndian>(mv.col).unwrap();
            }
          }
          ::std::fs::write(
            data_location.join(file_name).with_extension("bin"),
            buf,
          )
          .unwrap();
        }
      });
      self.lookahead_data.insert(frameno, lookahead_data);
    }

    let lookahead_data =
      self.lookahead_data.get_mut(&self.input_frameno).unwrap();
    const SCALE_FACTOR: f32 = 2.25;
    lookahead_data.block_importances = block_importances[&self.input_frameno]
      .iter()
      .map(|bi| *bi * SCALE_FACTOR)
      .collect();

    if !frame_set.is_empty() {
      let block_importances = lookahead_data.block_importances.iter();
      let lookahead_intra_costs = lookahead_data.lookahead_intra_costs.iter();
      let distortion_scales = lookahead_data.distortion_scales.iter_mut();
      for ((&propagate_cost, &intra_cost), distortion_scale) in
        block_importances.zip(lookahead_intra_costs).zip(distortion_scales)
      {
        *distortion_scale = crate::rdo::distortion_scale_for(
          propagate_cost as f64,
          intra_cost as f64,
        );
      }
      #[cfg(feature = "dump_lookahead_data")]
      {
        use byteorder::{NativeEndian, WriteBytesExt};
        let mut buf = vec![];
        let data_location = build_dump_properties();
        let file_name = format!("{:010}-imps", self.input_frameno);
        buf.write_u64::<NativeEndian>(h_in_imp_b as u64).unwrap();
        buf.write_u64::<NativeEndian>(w_in_imp_b as u64).unwrap();
        buf
          .write_u64::<NativeEndian>(if self
            .keyframes
            .contains(&self.input_frameno)
          {
            FRAME_SUBTYPE_I
          } else {
            FRAME_SUBTYPE_P
          } as u64)
          .unwrap();
        for y in 0..h_in_imp_b {
          for x in 0..w_in_imp_b {
            buf
              .write_f32::<NativeEndian>(f64::from(
                lookahead_data.distortion_scales[y * w_in_imp_b + x],
              ) as f32)
              .unwrap();
          }
        }
        ::std::fs::write(
          data_location.join(file_name).with_extension("bin"),
          buf,
        )
        .unwrap();
      }
    }
  }

  pub(crate) fn encode_packet(
    &mut self, cur_output_frameno: u64,
  ) -> Result<Packet<T>, EncoderStatus> {
    let fi = self.get_current_fi_mut();
    if let Some(fi) = fi {
      // TODO: See if we can avoid this clone here and the reinsert at the end of the function.
      let mut fi = fi.clone();
      let mut fs = FrameState::new_with_frame(
        &fi,
        Arc::clone(self.frame_q[&fi.input_frameno].as_ref().unwrap()),
      );

      let result;
      if fi.show_existing_frame {
        if !self.rc_state.ready() {
          return Err(EncoderStatus::NotReady);
        }

        fi.copy_quantizers(
          self.current_frame_group[fi.idx_in_group_output as usize - 1]
            .as_ref()
            .unwrap(),
        );

        let sef_data =
          encode_show_existing_frame(&fi, &mut fs, &self.inter_cfg);
        let bits = (sef_data.len() * 8) as i64;
        self.packet_data.extend(sef_data);
        self.rc_state.update_state(
          bits,
          FRAME_SUBTYPE_SEF,
          fi.show_frame,
          0,
          false,
          false,
        );
        let (rec, source) = if fi.show_frame {
          (Some(Arc::clone(&fs.rec)), Some(Arc::clone(&fs.input)))
        } else {
          (None, None)
        };

        self.output_frameno += 1;

        let input_frameno = fi.input_frameno;
        let frame_type = fi.frame_type;
        let qp = fi.base_q_idx;
        let enc_stats = fs.enc_stats;
        result = self.finalize_packet(
          rec,
          source,
          input_frameno,
          frame_type,
          qp,
          enc_stats,
        );
      } else if let Some(Some(frame)) = self.frame_q.get(&fi.input_frameno) {
        if !self.rc_state.ready() {
          return Err(EncoderStatus::NotReady);
        }
        let fti = fi.get_frame_subtype();
        let qps =
          self.rc_state.select_qi(self, fti, self.maybe_prev_log_base_q);
        fi.set_quantizers(&qps);

        if self.config.tune == Tune::Psychovisual {
          fi.activity_mask = ActivityMask::from_plane(&frame.planes[0]);
          fi.activity_mask
            .fill_scales(fi.sequence.bit_depth, &mut fi.activity_scales);
        } else {
          fi.activity_mask = ActivityMask::default();
        }

        if self.rc_state.needs_trial_encode(fti) {
          let mut trial_fs = fs.clone();
          let data = encode_frame(
            &fi,
            &mut trial_fs,
            &mut self.lookahead_data.get(&fi.input_frameno).unwrap().clone(),
            &self.inter_cfg,
          );
          self.rc_state.update_state(
            (data.len() * 8) as i64,
            fti,
            fi.show_frame,
            qps.log_target_q,
            true,
            false,
          );
          let qps =
            self.rc_state.select_qi(self, fti, self.maybe_prev_log_base_q);
          fi.set_quantizers(&qps);
        }

        let data = encode_frame(
          &fi,
          &mut fs,
          self.lookahead_data.get_mut(&fi.input_frameno).unwrap(),
          &self.inter_cfg,
        );
        let enc_stats = fs.enc_stats.clone();
        self.maybe_prev_log_base_q = Some(qps.log_base_q);
        // TODO: Add support for dropping frames.
        self.rc_state.update_state(
          (data.len() * 8) as i64,
          fti,
          fi.show_frame,
          qps.log_target_q,
          false,
          false,
        );
        self.packet_data.extend(data);

        let planes = if fi.sequence.chroma_sampling == Cs400 { 1 } else { 3 };

        Arc::get_mut(&mut fs.rec).unwrap().pad(fi.width, fi.height, planes);

        let (rec, source) = if fi.show_frame {
          (Some(Arc::clone(&fs.rec)), Some(Arc::clone(&fs.input)))
        } else {
          (None, None)
        };

        update_rec_buffer(cur_output_frameno, &mut fi, &fs);
        self.propagate_rec_buffer(&fi.rec_buffer, cur_output_frameno);

        self.output_frameno += 1;

        if fi.show_frame {
          let input_frameno = fi.input_frameno;
          let frame_type = fi.frame_type;
          let qp = fi.base_q_idx;
          result = self.finalize_packet(
            rec,
            source,
            input_frameno,
            frame_type,
            qp,
            enc_stats,
          );
        } else {
          return Err(EncoderStatus::Encoded);
        }
      } else {
        return Err(EncoderStatus::NeedMoreData);
      }

      // Insert the updated frame invariant back into the context. See TODO at top.
      if fi.frame_type == FrameType::KEY {
        self.current_keyframe = fi;
      } else {
        let pos = self
          .current_frame_group
          .iter_mut()
          .find(|cur_fi| {
            cur_fi.as_ref().map(|fi| fi.input_frameno)
              == Some(fi.input_frameno)
          })
          .unwrap()
          .as_mut()
          .unwrap();
        *pos = fi;
      }

      result
    } else {
      self.output_frameno += 1;
      Err(EncoderStatus::Encoded)
    }
  }

  /// Copy persistent fields into subsequent FrameInvariants
  fn propagate_rec_buffer(
    &mut self, rec_buffer: &ReferenceFramesSet<T>, cur_output_frameno: u64,
  ) {
    for subsequent_fi in self
      .current_frame_group
      .iter_mut()
      // Here we want the next valid non-show-existing-frame inter frame.
      //
      // Copying to show-existing-frame frames isn't actually required
      // for correct encoding, but it's needed for the reconstruction to
      // work correctly.
      .filter_map(|fi| fi.as_mut())
      .skip_while(|fi| fi.output_frameno <= cur_output_frameno)
    {
      subsequent_fi.rec_buffer = rec_buffer.clone();
      subsequent_fi.set_ref_frame_sign_bias();

      // Stop after the first non-show-existing-frame.
      if !subsequent_fi.show_existing_frame {
        break;
      }
    }
  }

  #[hawktracer(receive_packet)]
  pub fn receive_packet(&mut self) -> Result<Packet<T>, EncoderStatus> {
    if self.done_processing() {
      return Err(EncoderStatus::LimitReached);
    }

    if self.needs_more_lookahead() {
      return Err(EncoderStatus::NeedMoreData);
    }

    if self.at_frame_limit(self.input_frameno) {
      return Err(EncoderStatus::LimitReached);
    }

    if self.config.temporal_rdo() {
      self.compute_block_importances();
    }

    if self.next_group_start_input_frameno == self.input_frameno
      && self.next_group_start_output_frameno == self.output_frameno
    {
      self.compute_frame_invariants_group();
    }

    let cur_output_frameno = self.output_frameno;
    let mut ret = self.encode_packet(cur_output_frameno);

    if let Ok(ref mut pkt) = ret {
      self.garbage_collect(pkt.input_frameno);
      pkt.opaque = self.opaque_q.remove(&pkt.input_frameno);
      self.input_frameno += 1;
    }

    ret
  }

  fn finalize_packet(
    &mut self, rec: Option<Arc<Frame<T>>>, source: Option<Arc<Frame<T>>>,
    input_frameno: u64, frame_type: FrameType, qp: u8,
    enc_stats: EncoderStats,
  ) -> Result<Packet<T>, EncoderStatus> {
    let data = self.packet_data.clone();
    self.packet_data.clear();
    if write_temporal_delimiter(&mut self.packet_data).is_err() {
      return Err(EncoderStatus::Failure);
    }

    self.frames_processed += 1;
    Ok(Packet {
      data,
      rec,
      source,
      input_frameno,
      frame_type,
      qp,
      enc_stats,
      opaque: None,
    })
  }

  /// Releases frames from memory which are no longer needed.
  ///
  /// This keeps memory consumption at approximately O(n), where n = number of lookahead frames.
  fn garbage_collect(&mut self, cur_input_frameno: u64) {
    // We still need this frame and the one before it, but we can remove all previous ones.
    let frame_to_keep = cur_input_frameno.saturating_sub(1);
    if frame_to_keep == 0 {
      return;
    }

    let frame_q_start = self.frame_q.keys().next().cloned().unwrap_or(0);
    for i in frame_q_start..frame_to_keep {
      self.frame_q.remove(&i);
      self.keyframe_detector.intra_costs.remove(&i);
      self.lookahead_data.remove(&i);
    }
  }

  /// Builds the `FrameInvariant`s for the next group of frames
  fn compute_frame_invariants_group(&mut self) {
    // TODO: Use first-pass decisions if doing two-pass

    if self.keyframes.contains(&self.next_group_start_input_frameno) {
      // A keyframe is not part of a group, it is a separate entity.
      self.current_keyframe = FrameInvariants::new_key_frame(
        Arc::clone(&self.config),
        Arc::clone(&self.seq),
        self.next_group_start_input_frameno,
        self.next_group_start_output_frameno,
      );
      self.next_group_start_input_frameno += 1;
      self.next_group_start_output_frameno += 1;
      self.current_frame_group = Vec::new();
      return;
    }

    // A group always starts with (group_output_len - group_input_len) hidden
    //  frames, followed by group_input_len shown frames.
    // The shown frames iterate over the input frames in order, with frames
    //  already encoded as hidden frames now displayed with Show Existing
    //  Frame.
    // For example, for a pyramid depth of 2, the group is as follows:
    //                      |TU         |TU |TU |TU
    // idx_in_group_output:   0   1   2   3   4   5
    // input_frameno:         4   2   1  SEF  3  SEF
    // output_frameno:        1   2   3   4   5   6
    // level:                 0   1   2   1   2   0
    //                        ^^^^^   ^^^^^^^^^^^^^
    //                        hidden      shown
    // TODO: This only works for pyramid_depth <= 2 --- after that we need
    //  more hidden frames in the middle of the group.
    let pyramid_depth = if self.config.low_latency { 0 } else { 2 };
    // If there is a keyframe upcoming, do not include it in the input frames
    let ideal_input_width = 1 << pyramid_depth;
    let real_input_width = ideal_input_width.min(
      *self
        .keyframes
        .iter()
        .find(|kf| **kf > self.input_frameno)
        .unwrap_or(&u64::MAX)
        - self.input_frameno,
    );
    let pyramid_output_width = ideal_input_width + pyramid_depth;
    let switch_frame_interval = self.config.switch_frame_interval;
    assert!(switch_frame_interval % ideal_input_width == 0);

    let pyramid_shape = self.decide_pyramid_shape(
      ideal_input_width,
      real_input_width,
      pyramid_output_width,
      pyramid_depth,
    );
    // We need to propagate this forward at the end of the function
    // if the previous frame was not a show-existing-frame,
    // so we copy it here before it gets overwritten.
    let prev_rec_buffer = if self.current_keyframe.output_frameno + 1
      == pyramid_shape[0].output_frameno
    {
      None
    } else {
      let prev_fi = self
        .current_frame_group
        .iter()
        .filter_map(|fi| fi.as_ref())
        .last()
        .unwrap();
      if prev_fi.show_existing_frame {
        None
      } else {
        Some(prev_fi.rec_buffer.clone())
      }
    };
    let mut frame_group: Vec<Option<FrameInvariants<T>>> =
      Vec::with_capacity(pyramid_shape.len());
    pyramid_shape.into_iter().for_each(|pd| {
      if pd.placeholder_frame {
        frame_group.push(None);
        return;
      }

      let prev_fi =
        if pd.output_frameno == self.current_keyframe.output_frameno + 1 {
          Some(&self.current_keyframe)
        } else {
          match frame_group.iter().rfind(|fi| fi.is_some()) {
            Some(fi) => fi.as_ref(),
            None => self
              .current_frame_group
              .iter()
              .rfind(|fi| fi.is_some())
              .and_then(|fi| fi.as_ref()),
          }
        };
      let prev_fi = match prev_fi {
        Some(fi) => fi,
        None => &self.current_keyframe,
      };
      let fi = FrameInvariants::new_inter_frame(
        pd,
        prev_fi,
        &self.inter_cfg,
        self.config.error_resilient,
        self.current_keyframe.output_frameno,
      );
      frame_group.push(Some(fi));
    });

    self.current_frame_group = frame_group;
    if let Some(prev_rec_buffer) = prev_rec_buffer {
      self.propagate_rec_buffer(&prev_rec_buffer, 0);
    }
  }

  fn decide_pyramid_shape(
    &mut self, ideal_input_width: u64, real_input_width: u64,
    output_width: u64, max_pyramid_depth: u64,
  ) -> Vec<PyramidDecision> {
    let first_input_frame = self.input_frameno;
    let last_input_frame = self
      .keyframes
      .iter()
      .find(|&&frameno| frameno > first_input_frame)
      .copied()
      .or(self.limit)
      .unwrap_or(u64::MAX)
      .min(first_input_frame + real_input_width)
      - 1;

    let mut pyramid: Vec<PyramidDecision> =
      Vec::with_capacity(output_width as usize);
    for index in 0..output_width {
      // TODO: This only works with pyramid_depth <= 2.
      let input_frameno = if index < max_pyramid_depth {
        ideal_input_width >> index
      } else {
        index - max_pyramid_depth + 1
      } + first_input_frame
        - 1;
      pyramid.push(PyramidDecision {
        input_frameno: if input_frameno <= last_input_frame {
          input_frameno
        } else {
          pyramid
            .iter()
            .last()
            .map(|pd| pd.input_frameno)
            .unwrap_or(first_input_frame)
        },
        output_frameno: self.next_group_start_output_frameno + index,
        index_in_group: index,
        level: if index < max_pyramid_depth {
          // Hidden frames are output first (to be shown in the future).
          index
        } else {
          // Shown frames
          // TODO: This only works with pyramid_depth <= 2.
          pos_to_lvl(index - max_pyramid_depth + 1, max_pyramid_depth)
        },
        show_frame: index >= max_pyramid_depth,
        show_existing_frame: pyramid.iter().any(|pd: &PyramidDecision| {
          pd.input_frameno == input_frameno && !pd.placeholder_frame
        }),
        placeholder_frame: input_frameno > last_input_frame,
      });
    }

    self.next_group_start_input_frameno = last_input_frame + 1;
    self.next_group_start_output_frameno += pyramid.len() as u64;

    pyramid
  }

  /// Counts the number of output frames of each subtype in the next
  ///  reservoir_frame_delay temporal units (needed for rate control).
  /// Returns the number of output frames (excluding SEF frames) and output TUs
  ///  until the last keyframe in the next reservoir_frame_delay temporal units,
  ///  or the end of the interval, whichever comes first.
  /// The former is needed because it indicates the number of rate estimates we
  ///  will make.
  /// The latter is needed because it indicates the number of times new bitrate
  ///  is added to the buffer.
  pub(crate) fn guess_frame_subtypes(
    &self, nframes: &mut [i32; FRAME_NSUBTYPES + 1],
    reservoir_frame_delay: i32,
  ) -> (i32, i32) {
    for fti in 0..=FRAME_NSUBTYPES {
      nframes[fti] = 0;
    }

    // Two-pass calls this function before receive_packet(), and in particular
    // before the very first send_frame(), when the following maps are empty.
    // In this case, return 0 as the default value.
    let mut prev_keyframe_input_frameno = self.current_keyframe.input_frameno;
    let mut prev_keyframe_output_frameno =
      self.current_keyframe.output_frameno;

    let mut prev_keyframe_ntus = 0;
    // Does not include SEF frames.
    let mut prev_keyframe_nframes = 0;
    let mut acc: [i32; FRAME_NSUBTYPES + 1] = [0; FRAME_NSUBTYPES + 1];
    // Updates the frame counts with the accumulated values when we hit a
    //  keyframe.
    fn collect_counts(
      nframes: &mut [i32; FRAME_NSUBTYPES + 1],
      acc: &mut [i32; FRAME_NSUBTYPES + 1],
    ) {
      for fti in 0..=FRAME_NSUBTYPES {
        nframes[fti] += acc[fti];
        acc[fti] = 0;
      }
      acc[FRAME_SUBTYPE_I] += 1;
    }
    let mut output_frameno = self.output_frameno;
    let mut ntus = 0;
    // Does not include SEF frames.
    let mut nframes_total = 0;
    while ntus < reservoir_frame_delay {
      let output_frameno_in_gop =
        output_frameno - prev_keyframe_output_frameno;
      let is_kf = output_frameno_in_gop == 0;
      if is_kf {
        collect_counts(nframes, &mut acc);
        prev_keyframe_output_frameno = output_frameno;
        prev_keyframe_ntus = ntus;
        prev_keyframe_nframes = nframes_total;
        output_frameno += 1;
        ntus += 1;
        nframes_total += 1;
        continue;
      }
      let idx_in_group_output =
        self.inter_cfg.get_idx_in_group_output(output_frameno_in_gop);
      let input_frameno = prev_keyframe_input_frameno
        + self
          .inter_cfg
          .get_order_hint(output_frameno_in_gop, idx_in_group_output)
          as u64;
      // For rate control purposes, ignore any limit on frame count that has
      //  been set.
      // We pretend that we will keep encoding frames forever to prevent the
      //  control loop from driving us into the rails as we come up against a
      //  hard stop (with no more chance to correct outstanding errors).
      let next_keyframe_input_frameno =
        self.next_keyframe_input_frameno(prev_keyframe_input_frameno, true);
      // If we are re-ordering, we may skip some output frames in the final
      //  re-order group of the GOP.
      if input_frameno >= next_keyframe_input_frameno {
        // If we have encoded enough whole groups to reach the next keyframe,
        //  then start the next keyframe gop.
        if 1
          + (output_frameno - prev_keyframe_output_frameno)
            / self.inter_cfg.group_output_len
            * self.inter_cfg.group_input_len
          >= next_keyframe_input_frameno - prev_keyframe_input_frameno
        {
          collect_counts(nframes, &mut acc);
          prev_keyframe_input_frameno = input_frameno;
          prev_keyframe_output_frameno = output_frameno;
          prev_keyframe_ntus = ntus;
          prev_keyframe_nframes = nframes_total;
          // We do not currently use forward keyframes, so they should always
          //  end the current TU.
          output_frameno += 1;
          ntus += 1;
        }
        output_frameno += 1;
        continue;
      }
      if self.inter_cfg.get_show_existing_frame(idx_in_group_output) {
        acc[FRAME_SUBTYPE_SEF] += 1;
      } else {
        // TODO: Implement golden P-frames.
        let fti = FRAME_SUBTYPE_P
          + (self.inter_cfg.guess_level(idx_in_group_output) as usize);
        acc[fti] += 1;
        nframes_total += 1;
      }
      if self.inter_cfg.get_show_frame(idx_in_group_output) {
        ntus += 1;
      }
      output_frameno += 1;
    }
    if prev_keyframe_output_frameno <= self.output_frameno {
      // If there were no keyframes at all, or only the first frame was a
      //  keyframe, the accumulators never flushed and still contain counts for
      //  the entire buffer.
      // In both cases, we return these counts.
      collect_counts(nframes, &mut acc);
      (nframes_total, ntus)
    } else {
      // Otherwise, we discard what remains in the accumulators as they contain
      //  the counts from and past the last keyframe.
      (prev_keyframe_nframes, prev_keyframe_ntus)
    }
  }
}

#[cfg(feature = "dump_lookahead_data")]
pub fn build_dump_properties() -> PathBuf {
  let mut data_location = PathBuf::new();
  if env::var_os("RAV1E_DATA_PATH").is_some() {
    data_location.push(&env::var_os("RAV1E_DATA_PATH").unwrap());
  } else {
    data_location.push(&env::current_dir().unwrap());
    data_location.push(".lookahead_data");
  }
  fs::create_dir_all(&data_location).unwrap();
  data_location
}
