use std::collections::{BTreeMap, BTreeSet};

use av_metrics::video::Pixel;

use crate::{
  api::FrameData,
  encoder::{MAX_PYRAMID_WIDTH, MIN_PYRAMID_WIDTH},
};

/// This chooses how long the upcoming frame pyramid should be,
/// and the pyramid level for each frame within the pyramid,
/// using lookahead data.
///
/// The first frame in `lookahead_frames` is the alt ref frame for this pyramid.
pub(crate) fn select_pyramid_frames<T: Pixel>(
  lookahead_frames: &[(&u64, &FrameData<T>)], keyframes: &BTreeSet<u64>,
  flashes: &BTreeSet<u64>,
) -> FramePyramidDecision<T> {
  assert!(lookahead_frames.len() <= MAX_PYRAMID_WIDTH);

  let starting_output_frameno = *lookahead_frames[0].0;
  let ending_output_frameno = *lookahead_frames[lookahead_frames.len() - 1].0;

  let mut cut_point = lookahead_frames.len();
  for (i, &(output_frameno, _)) in lookahead_frames.iter().enumerate() {
    let flash_detected = flashes.contains(output_frameno);
    if should_cut_pyramid(&lookahead_frames[0].1, i, flash_detected) {
      cut_point = i;
      break;
    }
  }

  let this_pyramid = &lookahead_frames[..cut_point];
  let next_pyramid = &lookahead_frames[cut_point..];

  let mut new_fis = BTreeMap::new();
  todo!("Select the pyramid depths");

  // Adjust output framenos for next pyramid
  let adjustment = new_fis.len() - cut_point;
  next_pyramid.into_iter().for_each(|&(&frameno, data)| {
    let existing_key =
      new_fis.insert(frameno + adjustment as u64, data.clone());
    debug_assert!(existing_key.is_none());
  });

  FramePyramidDecision {
    new_fis,
    previous_output_frame_range: (
      starting_output_frameno,
      ending_output_frameno,
    ),
  }
}

fn should_cut_pyramid<T: Pixel>(
  altref_frame_data: &FrameData<T>, frame_index: usize, flash_detected: bool,
) -> bool {
  if frame_index < MIN_PYRAMID_WIDTH || flash_detected {
    return false;
  }

  let importance_threshold = ((128
    << (altref_frame_data.fi.config.bit_depth - 8))
    * altref_frame_data.fi.config.speed_settings.rdo_lookahead_frames)
    as f32
    * (1.0 + (frame_index * 2) as f32 / MAX_PYRAMID_WIDTH as f32);
  let important_block_count = altref_frame_data
    .fi
    .block_importances
    .iter()
    .filter(|val| **val > importance_threshold)
    .count();
  if important_block_count < (altref_frame_data.fi.block_importances.len() / 2)
  {
    return true;
  }

  false
}

pub(crate) struct FramePyramidDecision<T: Pixel> {
  pub new_fis: BTreeMap<u64, FrameData<T>>,
  pub previous_output_frame_range: (u64, u64),
}
