use crate::api::internal::InterConfig;
use crate::config::EncoderConfig;
use crate::context::{BlockOffset, FrameBlocks, TileBlockOffset};
use crate::cpu_features::CpuFeatureLevel;
use crate::dist::get_satd;
use crate::encoder::{
  FrameInvariants, FrameState, Sequence, IMPORTANCE_BLOCK_SIZE,
};
use crate::frame::{AsRegion, PlaneOffset};
use crate::me::{estimate_tile_motion, FrameMEStats};
use crate::partition::{get_intra_edges, BlockSize, REF_FRAMES};
use crate::predict::{IntraParam, PredictionMode};
use crate::rate::{QuantizerParameters, RCState};
use crate::rayon::iter::*;
use crate::rdo::DistortionScale;
use crate::tiling::{Area, PlaneRegion, TileRect};
use crate::transform::TxSize;
use crate::{ReferenceFrame, ReferenceFramesSet};
use rust_hawktracer::*;
use std::sync::Arc;
use v_frame::frame::Frame;
use v_frame::pixel::{CastFromPrimitive, Pixel};
use v_frame::plane::Plane;

use super::size_in_imp_b;

pub(crate) const IMP_BLOCK_MV_UNITS_PER_PIXEL: i64 = 8;
pub(crate) const IMP_BLOCK_SIZE_IN_MV_UNITS: i64 =
  IMPORTANCE_BLOCK_SIZE as i64 * IMP_BLOCK_MV_UNITS_PER_PIXEL;
pub(crate) const IMP_BLOCK_AREA_IN_MV_UNITS: i64 =
  IMP_BLOCK_SIZE_IN_MV_UNITS * IMP_BLOCK_SIZE_IN_MV_UNITS;

#[hawktracer(estimate_intra_costs)]
pub(crate) fn estimate_intra_costs<T: Pixel>(
  frame: &Frame<T>, bit_depth: usize, cpu_feature_level: CpuFeatureLevel,
) -> Box<[u32]> {
  let plane = &frame.planes[0];
  let mut plane_after_prediction = frame.planes[0].clone();

  let bsize = BlockSize::from_width_and_height(
    IMPORTANCE_BLOCK_SIZE,
    IMPORTANCE_BLOCK_SIZE,
  );
  let tx_size = bsize.tx_size();

  let (w_in_imp_b, h_in_imp_b) =
    size_in_imp_b(plane.cfg.width, plane.cfg.height);
  let mut intra_costs = Vec::with_capacity(h_in_imp_b * w_in_imp_b);

  for y in 0..h_in_imp_b {
    for x in 0..w_in_imp_b {
      let plane_org = plane.region(Area::Rect {
        x: (x * IMPORTANCE_BLOCK_SIZE) as isize,
        y: (y * IMPORTANCE_BLOCK_SIZE) as isize,
        width: IMPORTANCE_BLOCK_SIZE,
        height: IMPORTANCE_BLOCK_SIZE,
      });

      // TODO: other intra prediction modes.
      let edge_buf = get_intra_edges(
        &plane.as_region(),
        TileBlockOffset(BlockOffset { x, y }),
        0,
        0,
        bsize,
        PlaneOffset {
          x: (x * IMPORTANCE_BLOCK_SIZE) as isize,
          y: (y * IMPORTANCE_BLOCK_SIZE) as isize,
        },
        TxSize::TX_8X8,
        bit_depth,
        Some(PredictionMode::DC_PRED),
        false,
        IntraParam::None,
      );

      let mut plane_after_prediction_region = plane_after_prediction
        .region_mut(Area::Rect {
          x: (x * IMPORTANCE_BLOCK_SIZE) as isize,
          y: (y * IMPORTANCE_BLOCK_SIZE) as isize,
          width: IMPORTANCE_BLOCK_SIZE,
          height: IMPORTANCE_BLOCK_SIZE,
        });

      PredictionMode::DC_PRED.predict_intra(
        TileRect {
          x: x * IMPORTANCE_BLOCK_SIZE,
          y: y * IMPORTANCE_BLOCK_SIZE,
          width: IMPORTANCE_BLOCK_SIZE,
          height: IMPORTANCE_BLOCK_SIZE,
        },
        &mut plane_after_prediction_region,
        tx_size,
        bit_depth,
        &[], // Not used by DC_PRED
        IntraParam::None,
        None, // Not used by DC_PRED
        &edge_buf,
        cpu_feature_level,
      );

      let plane_after_prediction_region =
        plane_after_prediction.region(Area::Rect {
          x: (x * IMPORTANCE_BLOCK_SIZE) as isize,
          y: (y * IMPORTANCE_BLOCK_SIZE) as isize,
          width: IMPORTANCE_BLOCK_SIZE,
          height: IMPORTANCE_BLOCK_SIZE,
        });

      let intra_cost = get_satd(
        &plane_org,
        &plane_after_prediction_region,
        bsize.width(),
        bsize.height(),
        bit_depth,
        cpu_feature_level,
      );

      intra_costs.push(intra_cost);
    }
  }

  intra_costs.into_boxed_slice()
}

#[hawktracer(estimate_importance_block_difference)]
pub(crate) fn estimate_importance_block_difference<T: Pixel>(
  frame: Arc<Frame<T>>, ref_frame: Arc<Frame<T>>,
) -> f64 {
  let plane_org = &frame.planes[0];
  let plane_ref = &ref_frame.planes[0];
  let (w_in_imp_b, h_in_imp_b) =
    size_in_imp_b(plane_org.cfg.width, plane_org.cfg.height);

  let mut imp_block_costs = 0;

  (0..h_in_imp_b).for_each(|y| {
    (0..w_in_imp_b).for_each(|x| {
      // Coordinates of the top-left corner of the reference block, in MV
      // units.
      let region_org = plane_org.region(Area::Rect {
        x: (x * IMPORTANCE_BLOCK_SIZE) as isize,
        y: (y * IMPORTANCE_BLOCK_SIZE) as isize,
        width: IMPORTANCE_BLOCK_SIZE,
        height: IMPORTANCE_BLOCK_SIZE,
      });

      let region_ref = plane_ref.region(Area::Rect {
        x: (x * IMPORTANCE_BLOCK_SIZE) as isize,
        y: (y * IMPORTANCE_BLOCK_SIZE) as isize,
        width: IMPORTANCE_BLOCK_SIZE,
        height: IMPORTANCE_BLOCK_SIZE,
      });

      let sum_8x8_block = |region: &PlaneRegion<T>| {
        region
          .rows_iter()
          .map(|row| {
            // 16-bit precision is sufficient for an 8px row, as IMPORTANCE_BLOCK_SIZE * (2^12 - 1) < 2^16 - 1,
            // so overflow is not possible
            row.iter().map(|pixel| u16::cast_from(*pixel)).sum::<u16>() as i64
          })
          .sum::<i64>()
      };

      let histogram_org_sum = sum_8x8_block(&region_org);
      let histogram_ref_sum = sum_8x8_block(&region_ref);

      let count = (IMPORTANCE_BLOCK_SIZE * IMPORTANCE_BLOCK_SIZE) as i64;

      let mean = (((histogram_org_sum + count / 2) / count)
        - ((histogram_ref_sum + count / 2) / count))
        .abs();

      imp_block_costs += mean as u64;
    });
  });

  imp_block_costs as f64 / (w_in_imp_b * h_in_imp_b) as f64
}

#[hawktracer(estimate_inter_costs)]
pub(crate) fn estimate_inter_costs<T: Pixel>(
  frame: Arc<Frame<T>>, ref_frame: Arc<Frame<T>>, bit_depth: usize,
  config: EncoderConfig, sequence: Arc<Sequence>,
  buffer: Arc<[FrameMEStats; REF_FRAMES]>,
) -> f64 {
  let (w_in_imp_b, h_in_imp_b) = size_in_imp_b(config.width, config.height);
  let mut lookahead_data = LookaheadData::new(w_in_imp_b, h_in_imp_b);
  compute_lookahead_motion_vectors(
    Arc::clone(&frame),
    &mut lookahead_data,
    buffer,
    config,
    sequence,
    false,
  );

  // Estimate inter costs
  let plane_org = &frame.planes[0];
  let plane_ref = &ref_frame.planes[0];
  let stats = &lookahead_data.lookahead_me_stats.unwrap()[0];
  let bsize = BlockSize::from_width_and_height(
    IMPORTANCE_BLOCK_SIZE,
    IMPORTANCE_BLOCK_SIZE,
  );

  let mut inter_costs = 0;
  (0..h_in_imp_b).for_each(|y| {
    (0..w_in_imp_b).for_each(|x| {
      let mv = stats[y * 2][x * 2].mv;

      // Coordinates of the top-left corner of the reference block, in MV
      // units.
      let reference_x = x as i64 * IMP_BLOCK_SIZE_IN_MV_UNITS + mv.col as i64;
      let reference_y = y as i64 * IMP_BLOCK_SIZE_IN_MV_UNITS + mv.row as i64;

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

      inter_costs += get_satd(
        &region_org,
        &region_ref,
        bsize.width(),
        bsize.height(),
        bit_depth,
        config.cpu_feature_level,
      ) as u64;
    });
  });
  inter_costs as f64 / (w_in_imp_b * h_in_imp_b) as f64
}

/// Computes simple lookahead motion vectors using one ref frame and returns a temporary
/// FrameInvariants and FrameState with relevant data filled in.
#[hawktracer(compute_lookahead_motion_vectors)]
pub(crate) fn compute_lookahead_motion_vectors<T: Pixel>(
  frame: Arc<Frame<T>>, lookahead_data: &mut LookaheadData<T>,
  buffer: Arc<[FrameMEStats; REF_FRAMES]>, mut config: EncoderConfig,
  sequence: Arc<Sequence>, propagate: bool,
) {
  config.low_latency = true;
  config.speed_settings.multiref = false;
  let inter_cfg = InterConfig::new(&config);
  let last_fi =
    FrameInvariants::new_key_frame(Arc::new(config), sequence, 0, 0);
  let mut fi = FrameInvariants::new_inter_frame(
    PyramidDecision {
      input_frameno: 1,
      output_frameno: 1,
      index_in_group: 0,
      level: 0,
      show_frame: true,
      show_existing_frame: false,
      placeholder_frame: false,
    },
    &last_fi,
    &inter_cfg,
    false,
    0,
  );

  if propagate {
    let fti = fi.get_frame_subtype();
    let (log_base_q, log_q) = RCState::calc_flat_quantizer(
      config.quantizer as u8,
      config.bit_depth,
      fti,
    );
    let qps = QuantizerParameters::new_from_log_q(
      log_base_q,
      log_q,
      config.bit_depth,
      config.chroma_sampling,
      false,
    );

    // Our lookahead_rec_buffer should be filled with correct original frame
    // data from the previous frames. Copy it into rec_buffer because that's
    // what the MV search uses.
    fi.rec_buffer = lookahead_data.lookahead_rec_buffer.clone();

    // Estimate lambda with rate-control dry-run
    fi.set_quantizers(&qps);
  }

  // Compute the motion vectors.
  let mut fs = FrameState::new_with_frame_and_me_stats_and_rec(
    &fi,
    Arc::clone(&frame),
    buffer,
    // We do not use this field, so we can avoid the expensive allocation
    Arc::new(Frame {
      planes: [
        Plane::new(0, 0, 0, 0, 0, 0),
        Plane::new(0, 0, 0, 0, 0, 0),
        Plane::new(0, 0, 0, 0, 0, 0),
      ],
    }),
  );
  compute_motion_vectors(&mut fi, &mut fs, &inter_cfg);

  // Save the motion vectors to LookaheadData.
  lookahead_data.lookahead_me_stats = Some(fs.frame_me_stats.clone());

  if propagate {
    // Set lookahead_rec_buffer on this FrameInvariants for future
    // FrameInvariants to pick it up.
    let rfs = Arc::new(ReferenceFrame {
      // TODO: Does this need changed?
      order_hint: fi.order_hint,
      width: fi.width as u32,
      height: fi.height as u32,
      render_width: fi.render_width,
      render_height: fi.render_height,
      // Use the original frame contents.
      frame: fs.input.clone(),
      input_hres: fs.input_hres.clone(),
      input_qres: fs.input_qres.clone(),
      cdfs: fs.cdfs,
      frame_me_stats: fs.frame_me_stats.clone(),
      output_frameno: 1,
      segmentation: fs.segmentation,
    });
    for i in 0..(REF_FRAMES as usize) {
      if (fi.refresh_frame_flags & (1 << i)) != 0 {
        lookahead_data.lookahead_rec_buffer.frames[i] = Some(Arc::clone(&rfs));
        lookahead_data.lookahead_rec_buffer.deblock[i] = fs.deblock;
      }
    }
  }
}

#[hawktracer(compute_motion_vectors)]
fn compute_motion_vectors<T: Pixel>(
  fi: &mut FrameInvariants<T>, fs: &mut FrameState<T>, inter_cfg: &InterConfig,
) {
  let mut blocks = FrameBlocks::new(fi.w_in_b, fi.h_in_b);
  fi.sequence
    .tiling
    .tile_iter_mut(fs, &mut blocks)
    .collect::<Vec<_>>()
    .into_par_iter()
    .for_each(|mut ctx| {
      let ts = &mut ctx.ts;
      estimate_tile_motion(fi, ts, inter_cfg);
    });
}

#[derive(Debug, Clone)]
pub struct LookaheadData<T: Pixel> {
  /// Intra prediction cost estimations for each importance block.
  pub lookahead_intra_costs: Box<[u32]>,
  /// Future importance values for each importance block. That is, a value
  /// indicating how much future frames depend on the block (for example, via
  /// inter-prediction).
  pub block_importances: Box<[f32]>,
  /// Pre-computed distortion_scale.
  pub distortion_scales: Box<[DistortionScale]>,
  /// Motion vectors to the _original_ reference frames (not reconstructed).
  /// Used for lookahead purposes.
  ///
  /// These objects are very expensive to create, so their creation
  /// is deferred until it is needed.
  pub lookahead_me_stats: Option<Arc<[FrameMEStats; REF_FRAMES as usize]>>,
  /// The lookahead version of `rec_buffer`, used for storing and propagating
  /// the original reference frames (rather than reconstructed ones). The
  /// lookahead uses both `rec_buffer` and `lookahead_rec_buffer`, where
  /// `rec_buffer` contains the current frame's reference frames and
  /// `lookahead_rec_buffer` contains the next frame's reference frames.
  pub lookahead_rec_buffer: ReferenceFramesSet<T>,
}

impl<T: Pixel> LookaheadData<T> {
  pub fn new(w_in_imp_b: usize, h_in_imp_b: usize) -> Self {
    Self {
      lookahead_intra_costs: vec![0; h_in_imp_b * w_in_imp_b]
        .into_boxed_slice(),
      block_importances: vec![0.0; h_in_imp_b * w_in_imp_b].into_boxed_slice(),
      distortion_scales: vec![
        DistortionScale::new(0.0);
        h_in_imp_b * w_in_imp_b
      ]
      .into_boxed_slice(),
      lookahead_me_stats: None,
      lookahead_rec_buffer: ReferenceFramesSet::new(),
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PyramidDecision {
  pub input_frameno: u64,
  pub output_frameno: u64,
  pub index_in_group: u64,
  pub level: u64,
  pub show_frame: bool,
  pub show_existing_frame: bool,
  /// These were previously known as `invalid` frames.
  /// Now these will no longer associate to a `FrameInvariants`,
  /// in order to improve performance by reducing allocations of `FrameInvariants`.
  pub placeholder_frame: bool,
}
