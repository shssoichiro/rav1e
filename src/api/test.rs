// Copyright (c) 2018-2021, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

use crate::cpu_features::CpuFeatureLevel;
use crate::prelude::*;

use interpolate_name::interpolate_test;

fn setup_config(
  w: usize, h: usize, speed: usize, quantizer: usize, bit_depth: usize,
  chroma_sampling: ChromaSampling, min_keyint: u64, max_keyint: u64,
  bitrate: i32, low_latency: bool, switch_frame_interval: u64,
  no_scene_detection: bool, rdo_lookahead_frames: usize,
  min_quantizer: Option<u8>,
) -> Config {
  let mut enc = EncoderConfig::with_speed_preset(speed);
  enc.quantizer = quantizer;
  enc.min_key_frame_interval = min_keyint;
  enc.max_key_frame_interval = max_keyint;
  enc.low_latency = low_latency;
  enc.switch_frame_interval = switch_frame_interval;
  enc.width = w;
  enc.height = h;
  enc.bit_depth = bit_depth;
  enc.chroma_sampling = chroma_sampling;
  enc.bitrate = bitrate;
  if no_scene_detection {
    enc.speed_settings.scene_detection_mode = SceneDetectionSpeed::None;
  }
  enc.speed_settings.rdo_lookahead_frames = rdo_lookahead_frames;
  if let Some(min_quantizer) = min_quantizer {
    enc.min_quantizer = min_quantizer;
  }

  Config::new().with_encoder_config(enc).with_threads(1)
}

fn setup_encoder<T: Pixel>(
  w: usize, h: usize, speed: usize, quantizer: usize, bit_depth: usize,
  chroma_sampling: ChromaSampling, min_keyint: u64, max_keyint: u64,
  bitrate: i32, low_latency: bool, switch_frame_interval: u64,
  no_scene_detection: bool, rdo_lookahead_frames: usize,
  min_quantizer: Option<u8>,
) -> Context<T> {
  let cfg = setup_config(
    w,
    h,
    speed,
    quantizer,
    bit_depth,
    chroma_sampling,
    min_keyint,
    max_keyint,
    bitrate,
    low_latency,
    switch_frame_interval,
    no_scene_detection,
    rdo_lookahead_frames,
    min_quantizer,
  );
  cfg.new_context().unwrap()
}

/*
fn fill_frame<T: Pixel>(ra: &mut ChaChaRng, frame: &mut Frame<T>) {
  for plane in frame.planes.iter_mut() {
    let stride = plane.cfg.stride;
    for row in plane.data.chunks_mut(stride) {
      for pixel in row {
        let v: u8 = ra.gen();
        *pixel = T::cast_from(v);
      }
    }
  }
}
*/

fn fill_frame_const<T: Pixel>(frame: &mut Frame<T>, value: T) {
  for plane in frame.planes.iter_mut() {
    let stride = plane.cfg.stride;
    for row in plane.data.chunks_mut(stride) {
      for pixel in row {
        *pixel = value;
      }
    }
  }
}

struct TestFrameSender<T: Pixel> {
  ctx: Context<T>,
  scene_change_at: u64,
  limit: u64,
}

impl<T> TestFrameSender<T>
where
  T: Pixel,
{
  fn process_frame(&mut self) {
    for i in 0..self.limit {
      send_frame_kf(&mut self.ctx, i == self.scene_change_at);
    }
    self.ctx.flush();
    loop {
      match self.ctx.receive_packet() {
        Ok(_)
        | Err(EncoderStatus::LimitReached)
        | Err(EncoderStatus::Encoded) => {
          break;
        }
        _ => (),
      }
    }
  }
}

fn send_test_frame<T: Pixel>(ctx: &mut Context<T>, content_value: T) {
  let mut input = ctx.new_frame();
  fill_frame_const(&mut input, content_value);
  let _ = ctx.send_frame(input);
}

fn send_frame_kf<T: Pixel>(ctx: &mut Context<T>, keyframe: bool) {
  let input = ctx.new_frame();

  let frame_type_override =
    if keyframe { FrameTypeOverride::Key } else { FrameTypeOverride::No };

  let opaque = Some(Opaque::new(keyframe));

  let fp = FrameParameters { frame_type_override, opaque };

  let _ = ctx.send_frame((input, fp));
}

#[cfg(feature = "channel-api")]
mod channel {
  use super::*;

  #[interpolate_test(low_latency_no_scene_change, true, true)]
  #[interpolate_test(reorder_no_scene_change, false, true)]
  #[interpolate_test(low_latency_scene_change_detection, true, false)]
  #[interpolate_test(reorder_scene_change_detection, false, false)]
  fn flush(low_lantency: bool, no_scene_detection: bool) {
    let cfg = setup_config(
      64,
      80,
      10,
      100,
      8,
      ChromaSampling::Cs420,
      150,
      200,
      0,
      low_lantency,
      0,
      no_scene_detection,
      10,
      None,
    );

    let limit = 41;

    let (mut sf, rp) = cfg.new_channel::<u8>().unwrap();

    for _ in 0..limit {
      let input = sf.new_frame();
      let _ = sf.send(input);
    }

    drop(sf);

    let mut count = 0;

    for _ in 0..limit {
      let _ = rp
        .recv()
        .map(|_| {
          eprintln!("Packet Received {}/{}", count, limit);
          count += 1;
        })
        .unwrap();
    }

    assert_eq!(limit, count);
  }
}

#[interpolate_test(low_latency_no_scene_change, true, true)]
#[interpolate_test(reorder_no_scene_change, false, true)]
#[interpolate_test(low_latency_scene_change_detection, true, false)]
#[interpolate_test(reorder_scene_change_detection, false, false)]
fn flush(low_lantency: bool, no_scene_detection: bool) {
  let mut ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    150,
    200,
    0,
    low_lantency,
    0,
    no_scene_detection,
    10,
    None,
  );
  let limit = 41;

  for _ in 0..limit {
    let input = ctx.new_frame();
    let _ = ctx.send_frame(input);
  }

  ctx.flush();

  let mut count = 0;

  'out: for _ in 0..limit {
    loop {
      match ctx.receive_packet() {
        Ok(_) => {
          eprintln!("Packet Received {}/{}", count, limit);
          count += 1;
        }
        Err(EncoderStatus::EnoughData) => {
          eprintln!("{:?}", EncoderStatus::EnoughData);

          break 'out;
        }
        Err(e) => {
          eprintln!("{:?}", e);
          break;
        }
      }
    }
  }

  assert_eq!(limit, count);
}

#[interpolate_test(low_latency_no_scene_change, true, true)]
#[interpolate_test(reorder_no_scene_change, false, true)]
#[interpolate_test(low_latency_scene_change_detection, true, false)]
#[interpolate_test(reorder_scene_change_detection, false, false)]
fn flush_unlimited(low_lantency: bool, no_scene_detection: bool) {
  let mut ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    150,
    200,
    0,
    low_lantency,
    0,
    no_scene_detection,
    10,
    None,
  );
  let limit = 41;

  for _ in 0..limit {
    let input = ctx.new_frame();
    let _ = ctx.send_frame(input);
  }

  ctx.flush();

  let mut count = 0;

  'out: for _ in 0..limit {
    loop {
      match ctx.receive_packet() {
        Ok(_) => {
          eprintln!("Packet Received {}/{}", count, limit);
          count += 1;
        }
        Err(EncoderStatus::EnoughData) => {
          eprintln!("{:?}", EncoderStatus::EnoughData);

          break 'out;
        }
        Err(e) => {
          eprintln!("{:?}", e);
          break;
        }
      }
    }
  }

  assert_eq!(limit, count);
}

#[interpolate_test(0, 0)]
#[interpolate_test(1, 1)]
fn output_frameno_low_latency_minus(missing: usize) {
  // Test output_frameno configurations when there are <missing> less frames
  // than the perfect subgop size, in no-reorder mode.
  let expected = match missing {
    0 => {
      &[
        (0, true), // I-frame
        (1, true), // P-frame
        (2, true), // P-frame
        (3, true), // P-frame
        (4, true), // P-frame
        (5, true), // I-frame
        (6, true), // P-frame
        (7, true), // P-frame
        (8, true), // P-frame
        (9, true), // P-frame
      ][..]
    }
    1 => {
      &[
        (0, true), // I-frame
        (1, true), // P-frame
        (2, true), // P-frame
        (3, true), // P-frame
        (4, true), // P-frame
        (5, true), // I-frame
        (6, true), // P-frame
        (7, true), // P-frame
        (8, true), // P-frame
      ][..]
    }
    _ => unreachable!(),
  };

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    5,
    5,
    0,
    true,
    0,
    true,
    10,
    None,
  );
  let limit = 10 - missing;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 5 };
  for i in 0..limit {
    sender.process_frame();
    let frame = if i % 5 == 0 {
      &sender.ctx.inner.current_keyframe
    } else {
      sender.ctx.inner.current_frame_group[0].as_ref().unwrap()
    };
    assert_eq!(frame.input_frameno, expected[i].0);
  }
}

#[test]
fn switch_frame_interval() {
  // Test output_frameno configurations when there are <missing> less frames
  // than the perfect subgop size, in no-reorder mode.
  let expected = [
    (0, FrameType::KEY),
    (1, FrameType::INTER),
    (2, FrameType::SWITCH),
    (3, FrameType::INTER),
    (4, FrameType::SWITCH),
    (5, FrameType::KEY),
    (6, FrameType::INTER),
    (7, FrameType::SWITCH),
    (8, FrameType::INTER),
    (9, FrameType::SWITCH),
  ];

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    5,
    5,
    0,
    true,
    2,
    true,
    10,
    None,
  );
  let limit = 10;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 5 };
  for i in 0..limit {
    sender.process_frame();
    let frame = if i % 5 == 0 {
      &sender.ctx.inner.current_keyframe
    } else {
      sender.ctx.inner.current_frame_group[0].as_ref().unwrap()
    };
    assert_eq!(frame.input_frameno, expected[i].0);
    assert_eq!(frame.frame_type, expected[i].1);
  }
}

#[interpolate_test(0, 0)]
#[interpolate_test(1, 1)]
fn pyramid_level_low_latency_minus(missing: usize) {
  // Test pyramid_level configurations when there are <missing> less frames
  // than the perfect subgop size, in no-reorder mode.

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    5,
    5,
    0,
    true,
    0,
    true,
    10,
    None,
  );
  let limit = 10 - missing;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 0 };
  for i in 0..limit {
    sender.process_frame();
    let frame = if i % 5 == 0 {
      &sender.ctx.inner.current_keyframe
    } else {
      sender.ctx.inner.current_frame_group[0].as_ref().unwrap()
    };
    assert_eq!(frame.pyramid_level, 0);
  }
}

#[interpolate_test(0, 0)]
#[interpolate_test(1, 1)]
#[interpolate_test(2, 2)]
#[interpolate_test(3, 3)]
#[interpolate_test(4, 4)]
fn output_frameno_reorder_minus(missing: usize) {
  // Test output_frameno configurations when there are <missing> less frames
  // than the perfect subgop size.
  let expected = match missing {
    0 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
        Some(9), // P-frame
        Some(7), // B0-frame
        Some(6), // B1-frame (first)
        Some(7), // B0-frame (show existing)
        Some(8), // B1-frame (second)
        Some(9), // P-frame (show existing)
      ][..]
    }
    1 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
        None,    // Last frame (missing)
        Some(7), // B0-frame
        Some(6), // B1-frame (first)
        Some(7), // B0-frame (show existing)
        Some(8), // B1-frame (second)
        None,    // Last frame (missing)
      ][..]
    }
    2 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
        None,    // Last frame (missing)
        Some(7), // B0-frame
        Some(6), // B1-frame (first)
        Some(7), // B0-frame (show existing)
        None,    // 2nd last (missing)
        None,    // Last frame (missing)
      ][..]
    }
    3 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
        None,    // Last frame (missing)
        None,    // 3rd last (missing)
        Some(6), // B1-frame (first)
        None,    // 3rd last (missing)
        None,    // 2nd last (missing)
        None,    // Last frame (missing)
      ][..]
    }
    4 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
      ][..]
    }
    _ => unreachable!(),
  };

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    5,
    5,
    0,
    false,
    0,
    true,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);

  let limit = 10 - missing;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 5 };

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    sender.process_frame();

    if sender.ctx.inner.current_frame_group.is_empty() {
      let frame = &sender.ctx.inner.current_keyframe;
      assert_eq!(Some(frame.input_frameno), expected[i]);
    } else {
      let frame = sender
        .ctx
        .inner
        .current_frame_group
        .get(
          (i - sender.ctx.inner.current_keyframe.output_frameno as usize - 1)
            % sender.ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.input_frameno), expected[i]);
    };
  }
}

#[interpolate_test(0, 0)]
#[interpolate_test(1, 1)]
#[interpolate_test(2, 2)]
#[interpolate_test(3, 3)]
#[interpolate_test(4, 4)]
fn pyramid_level_reorder_minus(missing: usize) {
  // Test pyramid_level configurations when there are <missing> less frames
  // than the perfect subgop size.
  let expected = match missing {
    0 => {
      &[
        Some(0), // I-frame
        Some(0), // P-frame
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
        Some(0), // P-frame (show existing)
        Some(0), // I-frame
        Some(0), // P-frame
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
        Some(0), // P-frame (show existing)
      ][..]
    }
    1 => {
      &[
        Some(0), // I-frame
        Some(0), // P-frame
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
        Some(0), // P-frame (show existing)
        Some(0), // I-frame
        None,    // Last frame (missing)
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
        None,    // Last frame (missing)
      ][..]
    }
    2 => {
      &[
        Some(0), // I-frame
        Some(0), // P-frame
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
        Some(0), // P-frame (show existing)
        Some(0), // I-frame
        None,    // Last frame (missing)
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        None,    // 2nd last (missing)
        None,    // Last frame (missing)
      ][..]
    }
    3 => {
      &[
        Some(0), // I-frame
        Some(0), // P-frame
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
        Some(0), // P-frame (show existing)
        Some(0), // I-frame
        None,    // Last frame (missing)
        None,    // 3rd last (missing)
        Some(2), // B1-frame (first)
        None,    // 3rd last (missing)
        None,    // 2nd last (missing)
        None,    // Last frame (missing)
      ][..]
    }
    4 => {
      &[
        Some(0), // I-frame
        Some(0), // P-frame
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
        Some(0), // P-frame (show existing)
        Some(0), // I-frame
      ][..]
    }
    _ => unreachable!(),
  };

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    5,
    5,
    0,
    false,
    0,
    true,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);

  let limit = 10 - missing;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 5 };

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    sender.process_frame();
    if sender.ctx.inner.current_frame_group.is_empty() {
      let frame = &sender.ctx.inner.current_keyframe;
      assert_eq!(Some(frame.pyramid_level), expected[i]);
    } else {
      let frame = sender
        .ctx
        .inner
        .current_frame_group
        .get(
          (i - sender.ctx.inner.current_keyframe.output_frameno as usize - 1)
            % sender.ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.pyramid_level), expected[i]);
    };
  }
}

#[interpolate_test(0, 0)]
#[interpolate_test(1, 1)]
#[interpolate_test(2, 2)]
#[interpolate_test(3, 3)]
#[interpolate_test(4, 4)]
fn output_frameno_reorder_scene_change_at(scene_change_at: u64) {
  // Test output_frameno configurations when there's a scene change at the
  // <scene_change_at>th frame.
  let expected = match scene_change_at {
    0 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
      ][..]
    }
    1 => {
      &[
        Some(0), // I-frame
        Some(1), // I-frame
        None,
        Some(3), // B0-frame
        Some(2), // B1-frame (first)
        Some(3), // B0-frame (show existing)
        Some(4), // B1-frame (second)
      ][..]
    }
    2 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        None,    // Missing
        Some(1), // B1-frame (first)
        None,    // Missing
        None,    // Missing
        None,    // Missing
        Some(2), // I-frame
        None,
        Some(4), // B0-frame
        Some(3), // B1-frame (first)
        Some(4), // B0-frame (show existing)
      ][..]
    }
    3 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        None,    // Missing
        None,    // Missing
        Some(3), // I-frame
        None,
        None,
        Some(4), // B1-frame (first)
      ][..]
    }
    4 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        None,    // Missing
        Some(4), // I-frame
      ][..]
    }
    _ => unreachable!(),
  };

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    5,
    0,
    false,
    0,
    true,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);

  let limit = 5;
  let mut sender = TestFrameSender { ctx, limit, scene_change_at };

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    sender.process_frame();

    if sender.ctx.inner.current_frame_group.is_empty() {
      let frame = &sender.ctx.inner.current_keyframe;
      assert_eq!(Some(frame.input_frameno), expected[i]);
    } else {
      let frame = sender
        .ctx
        .inner
        .current_frame_group
        .get(
          (i - sender.ctx.inner.current_keyframe.output_frameno as usize - 1)
            % sender.ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.input_frameno), expected[i]);
    };
  }
}

#[interpolate_test(0, 0)]
#[interpolate_test(1, 1)]
#[interpolate_test(2, 2)]
#[interpolate_test(3, 3)]
#[interpolate_test(4, 4)]
fn pyramid_level_reorder_scene_change_at(scene_change_at: u64) {
  // Test pyramid_level configurations when there's a scene change at the
  // <scene_change_at>th frame.
  let expected = match scene_change_at {
    0 => {
      &[
        Some(0), // I-frame
        Some(0), // P-frame
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
        Some(0), // P-frame (show existing)
      ][..]
    }
    1 => {
      &[
        Some(0), // I-frame
        Some(0), // I-frame
        None,
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
      ][..]
    }
    2 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        None,    // Missing
        Some(2), // B1-frame (first)
        None,    // Missing
        None,    // Missing
        None,    // Missing
        Some(0), // I-frame
        None,
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
      ][..]
    }
    3 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        None,    // Missing
        None,    // Missing
        Some(0), // I-frame
        None,
        None,
        Some(2), // B1-frame (first)
      ][..]
    }
    4 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        Some(1), // B0-frame
        Some(2), // B1-frame (first)
        Some(1), // B0-frame (show existing)
        Some(2), // B1-frame (second)
        None,    // Missing
        Some(0), // I-frame
      ][..]
    }
    _ => unreachable!(),
  };

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    5,
    0,
    false,
    0,
    true,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);

  let limit = 5;
  let mut sender = TestFrameSender { ctx, limit, scene_change_at };

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    sender.process_frame();
    if sender.ctx.inner.current_frame_group.is_empty() {
      let frame = &sender.ctx.inner.current_keyframe;
      assert_eq!(Some(frame.pyramid_level), expected[i]);
    } else {
      let frame = sender
        .ctx
        .inner
        .current_frame_group
        .get(
          (i - sender.ctx.inner.current_keyframe.output_frameno as usize - 1)
            % sender.ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.pyramid_level), expected[i]);
    };
  }
}

#[interpolate_test(0, 0)]
#[interpolate_test(1, 1)]
#[interpolate_test(2, 2)]
#[interpolate_test(3, 3)]
#[interpolate_test(4, 4)]
fn output_frameno_incremental_reorder_minus(missing: usize) {
  // Test output_frameno configurations when there are <missing> less frames
  // than the perfect subgop size, computing the lookahead data incrementally.
  let expected = match missing {
    0 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
        Some(9), // P-frame
        Some(7), // B0-frame
        Some(6), // B1-frame (first)
        Some(7), // B0-frame (show existing)
        Some(8), // B1-frame (second)
        Some(9), // P-frame (show existing)
      ][..]
    }
    1 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
        None,    // Last frame (missing)
        Some(7), // B0-frame
        Some(6), // B1-frame (first)
        Some(7), // B0-frame (show existing)
        Some(8), // B1-frame (second)
        None,    // Last frame (missing)
      ][..]
    }
    2 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
        None,    // Last frame (missing)
        Some(7), // B0-frame
        Some(6), // B1-frame (first)
        Some(7), // B0-frame (show existing)
        None,    // 2nd last (missing)
        None,    // Last frame (missing)
      ][..]
    }
    3 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
        None,    // Last frame (missing)
        None,    // 3rd last (missing)
        Some(6), // B1-frame (first)
        None,    // 3rd last (missing)
        None,    // 2nd last (missing)
        None,    // Last frame (missing)
      ][..]
    }
    4 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
        Some(5), // I-frame
      ][..]
    }
    _ => unreachable!(),
  };

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    5,
    5,
    0,
    false,
    0,
    true,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);

  let limit = 10 - missing;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 5 };

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    sender.process_frame();

    if sender.ctx.inner.current_frame_group.is_empty() {
      let frame = &sender.ctx.inner.current_keyframe;
      assert_eq!(Some(frame.input_frameno), expected[i]);
    } else {
      let frame = sender
        .ctx
        .inner
        .current_frame_group
        .get(
          (i - sender.ctx.inner.current_keyframe.output_frameno as usize - 1)
            % sender.ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.input_frameno), expected[i]);
    };
  }
}

#[interpolate_test(0, 0)]
#[interpolate_test(1, 1)]
#[interpolate_test(2, 2)]
#[interpolate_test(3, 3)]
#[interpolate_test(4, 4)]
fn output_frameno_incremental_reorder_scene_change_at(scene_change_at: u64) {
  // Test output_frameno configurations when there's a scene change at the
  // <scene_change_at>th frame, computing the lookahead data incrementally.
  let expected = match scene_change_at {
    0 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
      ][..]
    }
    1 => {
      &[
        Some(0), // I-frame
        Some(1), // I-frame
        None,
        Some(3), // B0-frame
        Some(2), // B1-frame (first)
        Some(3), // B0-frame (show existing)
        Some(4), // B1-frame (second)
      ][..]
    }
    2 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        None,    // Missing
        Some(1), // B1-frame (first)
        None,    // Missing
        None,    // Missing
        None,    // Missing
        Some(2), // I-frame
        None,
        Some(4), // B0-frame
        Some(3), // B1-frame (first)
        Some(4), // B0-frame (show existing)
      ][..]
    }
    3 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        None,    // Missing
        None,    // Missing
        Some(3), // I-frame
        None,
        None,
        Some(4), // B1-frame (first)
      ][..]
    }
    4 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        None,    // Missing
        Some(4), // I-frame
      ][..]
    }
    _ => unreachable!(),
  };

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    5,
    0,
    false,
    0,
    true,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);

  let limit = 5;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at };

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    sender.process_frame();

    if sender.ctx.inner.current_frame_group.is_empty() {
      let frame = &sender.ctx.inner.current_keyframe;
      assert_eq!(Some(frame.input_frameno), expected[i]);
    } else {
      let frame = sender
        .ctx
        .inner
        .current_frame_group
        .get(
          (i - sender.ctx.inner.current_keyframe.output_frameno as usize - 1)
            % sender.ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.input_frameno), expected[i]);
    };
  }
}

#[test]
fn test_opaque_delivery() {
  let mut ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    5,
    0,
    true,
    0,
    false,
    10,
    None,
  );

  let kf_at = 3;

  let limit = 10;
  for i in 0..limit {
    send_frame_kf(&mut ctx, kf_at == i);
  }
  ctx.flush();

  while let Ok(pkt) = ctx.receive_packet() {
    let Packet { opaque, input_frameno, .. } = pkt;
    if let Some(opaque) = opaque {
      let kf = opaque.downcast::<bool>().unwrap();
      assert_eq!(kf, Box::new(input_frameno == kf_at));
    }
  }
}

#[interpolate_test(0, 0)]
#[interpolate_test(1, 1)]
#[interpolate_test(2, 2)]
#[interpolate_test(3, 3)]
#[interpolate_test(4, 4)]
fn output_frameno_incremental_reorder_keyframe_at(kf_at: u64) {
  // Test output_frameno configurations when there's a forced keyframe at the
  // <kf_at>th frame, computing the lookahead data incrementally.
  let expected = match kf_at {
    0 => {
      &[
        Some(0), // I-frame
        Some(4), // P-frame
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        Some(4), // P-frame (show existing)
      ][..]
    }
    1 => {
      &[
        Some(0), // I-frame
        Some(1), // I-frame
        None,    // Missing
        Some(3), // B0-frame
        Some(2), // B1-frame (first)
        Some(3), // B0-frame (show existing)
        Some(4), // B1-frame (second)
        None,    // Missing
      ][..]
    }
    2 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        None,    // Missing
        Some(1), // B1-frame (first)
        None,    // Missing
        None,    // Missing
        None,    // Missing
        Some(2), // I-frame
        None,    // Missing
        Some(4), // B0-frame
        Some(3), // B1-frame (first)
        Some(4), // B0-frame (show existing)
        None,    // Missing
        None,    // Missing
      ][..]
    }
    3 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        None,    // Missing
        None,    // Missing
        Some(3), // I-frame
        None,    // Missing
        None,    // Missing
        Some(4), // B1-frame (first)
        None,    // Missing
        None,    // Missing
        None,    // Missing
      ][..]
    }
    4 => {
      &[
        Some(0), // I-frame
        None,    // Missing
        Some(2), // B0-frame
        Some(1), // B1-frame (first)
        Some(2), // B0-frame (show existing)
        Some(3), // B1-frame (second)
        None,    // Missing
        Some(4), // I-frame
      ][..]
    }
    _ => unreachable!(),
  };

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    5,
    0,
    false,
    0,
    true,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);

  let limit = 5;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: kf_at };

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    sender.process_frame();

    if sender.ctx.inner.current_frame_group.is_empty() {
      let frame = &sender.ctx.inner.current_keyframe;
      assert_eq!(Some(frame.input_frameno), expected[i]);
    } else {
      let frame = sender
        .ctx
        .inner
        .current_frame_group
        .get(
          (i - sender.ctx.inner.current_keyframe.output_frameno as usize - 1)
            % sender.ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.input_frameno), expected[i]);
    };
  }
}

#[interpolate_test(1, 1)]
#[interpolate_test(2, 2)]
#[interpolate_test(3, 3)]
fn output_frameno_no_scene_change_at_short_flash(flash_at: usize) {
  // Test output_frameno configurations when there's a single-frame flash at the
  // <flash_at>th frame.
  let expected = [
    Some(0), // I-frame
    Some(4), // P-frame
    Some(2), // B0-frame
    Some(1), // B1-frame (first)
    Some(2), // B0-frame (show existing)
    Some(3), // B1-frame (second)
    Some(4), // P-frame (show existing)
  ];

  let mut ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    5,
    0,
    false,
    0,
    false,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);

  let limit = 5;

  for i in 0..limit {
    if i == flash_at {
      send_test_frame(&mut ctx, u8::MIN);
    } else {
      send_test_frame(&mut ctx, u8::MAX);
    }
  }
  ctx.flush();

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    loop {
      match ctx.receive_packet() {
        Ok(_)
        | Err(EncoderStatus::LimitReached)
        | Err(EncoderStatus::Encoded) => {
          break;
        }
        _ => (),
      }
    }
    if ctx.inner.current_frame_group.is_empty() {
      let frame = &ctx.inner.current_keyframe;
      assert_eq!(Some(frame.input_frameno), expected[i]);
    } else {
      let frame = &ctx
        .inner
        .current_frame_group
        .get(
          (i - ctx.inner.current_keyframe.output_frameno as usize - 1)
            % ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.input_frameno), expected[i]);
    };
  }
}

#[test]
fn output_frameno_no_scene_change_at_flash_smaller_than_max_len_flash() {
  // Test output_frameno configurations when there's a multi-frame flash
  // with length equal to the max flash length
  let expected = [
    Some(0), // I-frame
    Some(4), // P-frame
    Some(2), // B0-frame
    Some(1), // B1-frame (first)
    Some(2), // B0-frame (show existing)
    Some(3), // B1-frame (second)
    Some(4), // P-frame (show existing)
    None,    // invalid
    Some(6), // B0-frame
    Some(5), // B1-frame (first)
    Some(6), // B0-frame (show existing)
    Some(7), // B1-frame (second)
    None,    // invalid
  ];

  let mut ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    10,
    0,
    false,
    0,
    false,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);
  assert_eq!(ctx.inner.inter_cfg.group_input_len, 4);

  let limit = 8;

  for i in 0..limit {
    if i <= 1 || i >= 6 {
      send_test_frame(&mut ctx, u8::MIN);
    } else {
      send_test_frame(&mut ctx, u8::MAX);
    }
  }
  ctx.flush();

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    loop {
      match ctx.receive_packet() {
        Ok(_)
        | Err(EncoderStatus::LimitReached)
        | Err(EncoderStatus::Encoded) => {
          break;
        }
        _ => (),
      }
    }
    if ctx.inner.current_frame_group.is_empty() {
      let frame = &ctx.inner.current_keyframe;
      assert_eq!(Some(frame.input_frameno), expected[i]);
    } else {
      let frame = &ctx
        .inner
        .current_frame_group
        .get(
          (i - ctx.inner.current_keyframe.output_frameno as usize - 1)
            % ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.input_frameno), expected[i]);
    };
  }
}

#[test]
fn output_frameno_scene_change_before_flash_longer_than_max_flash_len() {
  // Test output_frameno configurations when there's a multi-frame flash
  // with length greater than the max flash length
  let expected = [
    Some(0), // I-frame
    None,    // invalid
    None,    // invalid
    Some(1), // B1-frame (first)
    None,    // invalid
    None,    // invalid
    None,    // invalid
    Some(2), // I-frame
    Some(6), // P-frame
    Some(4), // B0-frame
    Some(3), // B1-frame (first)
    Some(4), // B0-frame (show existing)
    Some(5), // B1-frame (second)
    Some(6), // P-frame (show existing)
    None,    // invalid
    None,    // invalid
    Some(7), // B1-frame (first)
    None,    // invalid
    None,    // invalid
    None,    // invalid
    Some(8), // I-frame
  ];

  let mut ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    10,
    0,
    false,
    0,
    false,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);
  assert_eq!(ctx.inner.inter_cfg.group_input_len, 4);

  let limit = 15;

  for i in 0..limit {
    if i <= 1 || i >= 8 {
      send_test_frame(&mut ctx, u8::MIN);
    } else {
      send_test_frame(&mut ctx, u8::MAX);
    }
  }
  ctx.flush();

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    loop {
      match ctx.receive_packet() {
        Ok(_)
        | Err(EncoderStatus::LimitReached)
        | Err(EncoderStatus::Encoded) => {
          break;
        }
        _ => (),
      }
    }
    if ctx.inner.current_frame_group.is_empty() {
      let frame = &ctx.inner.current_keyframe;
      assert_eq!(Some(frame.input_frameno), expected[i]);
    } else {
      let frame = &ctx
        .inner
        .current_frame_group
        .get(
          (i - ctx.inner.current_keyframe.output_frameno as usize - 1)
            % ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.input_frameno), expected[i]);
    };
  }
}

#[test]
fn output_frameno_scene_change_after_multiple_flashes() {
  // Test output_frameno configurations when there are multiple consecutive flashes
  let expected = [
    Some(0), // I-frame
    Some(4), // P-frame
    Some(2), // B0-frame
    Some(1), // B1-frame (first)
    Some(2), // B0-frame (show existing)
    Some(3), // B1-frame (second)
    Some(4), // P-frame (show existing),
    Some(5), // I-frame
    Some(9), // P-frame
    Some(7), // B0-frame
    Some(6), // B1-frame (first)
    Some(7), // B0-frame (show existing)
    Some(8), // B1-frame (second)
    Some(9), // P-frame (show existing),
  ];

  let mut ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    10,
    0,
    false,
    0,
    false,
    10,
    None,
  );

  // TODO: when we support more pyramid depths, this test will need tweaks.
  assert_eq!(ctx.inner.inter_cfg.pyramid_depth, 2);
  assert_eq!(ctx.inner.inter_cfg.group_input_len, 4);

  let limit = 11;
  let values = [u8::MIN, u8::MIN, 40, 100, 160, 240, 240, 240, 240, 240, 240];

  for i in 0..limit {
    send_test_frame(&mut ctx, values[i]);
  }
  ctx.flush();

  for i in 0..999 {
    if i == expected.len() {
      break;
    }

    loop {
      match ctx.receive_packet() {
        Ok(_)
        | Err(EncoderStatus::LimitReached)
        | Err(EncoderStatus::Encoded) => {
          break;
        }
        _ => (),
      }
    }
    if ctx.inner.current_frame_group.is_empty() {
      let frame = &ctx.inner.current_keyframe;
      assert_eq!(Some(frame.input_frameno), expected[i]);
    } else {
      let frame = &ctx
        .inner
        .current_frame_group
        .get(
          (i - ctx.inner.current_keyframe.output_frameno as usize - 1)
            % ctx.inner.inter_cfg.group_output_len as usize,
        )
        .unwrap_or(&None);
      assert_eq!(frame.as_ref().map(|fi| fi.input_frameno), expected[i]);
    };
  }
}

#[derive(Clone, Copy)]
struct LookaheadTestExpectations {
  post_receive_input_frameno: [u64; 60],
  post_receive_frame_q_lens: [usize; 60],
}

#[test]
fn lookahead_size_properly_bounded_8() {
  const LOOKAHEAD_SIZE: usize = 8;
  const EXPECTATIONS: LookaheadTestExpectations = LookaheadTestExpectations {
    post_receive_input_frameno: [
      0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 4, 5, 5, 5, 8, 9, 9, 9, 12, 13, 13, 13,
      16, 17, 17, 17, 20, 21, 21, 21, 24, 25, 25, 25, 28, 29, 29, 29, 32, 33,
      33, 33, 36, 37, 37, 37, 40, 41, 41, 41, 44, 45, 45, 45, 48, 49, 49, 49,
      52,
    ],
    post_receive_frame_q_lens: [
      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 10, 10, 11, 12, 10, 10, 11, 12, 10,
      10, 11, 12, 10, 10, 11, 12, 10, 10, 11, 12, 10, 10, 11, 12, 10, 10, 11,
      12, 10, 10, 11, 12, 10, 10, 11, 12, 10, 10, 11, 12, 10, 10, 11, 12, 10,
      10, 11, 12, 10,
    ],
  };
  lookahead_size_properly_bounded(LOOKAHEAD_SIZE, false, &EXPECTATIONS);
}

#[test]
fn lookahead_size_properly_bounded_10() {
  const LOOKAHEAD_SIZE: usize = 10;
  const EXPECTATIONS: LookaheadTestExpectations = LookaheadTestExpectations {
    post_receive_input_frameno: [
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 4, 5, 5, 5, 8, 9, 9, 9, 12, 13,
      13, 13, 16, 17, 17, 17, 20, 21, 21, 21, 24, 25, 25, 25, 28, 29, 29, 29,
      32, 33, 33, 33, 36, 37, 37, 37, 40, 41, 41, 41, 44, 45, 45, 45, 48, 49,
      49,
    ],
    post_receive_frame_q_lens: [
      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 12, 12, 13, 14, 12, 12, 13,
      14, 12, 12, 13, 14, 12, 12, 13, 14, 12, 12, 13, 14, 12, 12, 13, 14, 12,
      12, 13, 14, 12, 12, 13, 14, 12, 12, 13, 14, 12, 12, 13, 14, 12, 12, 13,
      14, 12, 12, 13,
    ],
  };
  lookahead_size_properly_bounded(LOOKAHEAD_SIZE, false, &EXPECTATIONS);
}

#[test]
fn lookahead_size_properly_bounded_16() {
  const LOOKAHEAD_SIZE: usize = 16;
  const EXPECTATIONS: LookaheadTestExpectations = LookaheadTestExpectations {
    post_receive_input_frameno: [
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 4, 5, 5, 5, 8,
      9, 9, 9, 12, 13, 13, 13, 16, 17, 17, 17, 20, 21, 21, 21, 24, 25, 25, 25,
      28, 29, 29, 29, 32, 33, 33, 33, 36, 37, 37, 37, 40, 41, 41, 41, 44,
    ],
    post_receive_frame_q_lens: [
      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 18,
      18, 19, 20, 18, 18, 19, 20, 18, 18, 19, 20, 18, 18, 19, 20, 18, 18, 19,
      20, 18, 18, 19, 20, 18, 18, 19, 20, 18, 18, 19, 20, 18, 18, 19, 20, 18,
      18, 19, 20, 18,
    ],
  };
  lookahead_size_properly_bounded(LOOKAHEAD_SIZE, false, &EXPECTATIONS);
}

#[test]
fn lookahead_size_properly_bounded_lowlatency_8() {
  const LOOKAHEAD_SIZE: usize = 8;
  const EXPECTATIONS: LookaheadTestExpectations = LookaheadTestExpectations {
    post_receive_input_frameno: [
      0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
      15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
      33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
      51, 52,
    ],
    post_receive_frame_q_lens: [
      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
      10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
      10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
      10, 10, 10, 10,
    ],
  };
  lookahead_size_properly_bounded(LOOKAHEAD_SIZE, true, &EXPECTATIONS);
}

#[test]
fn lookahead_size_properly_bounded_lowlatency_1() {
  const LOOKAHEAD_SIZE: usize = 1;
  const EXPECTATIONS: LookaheadTestExpectations = LookaheadTestExpectations {
    post_receive_input_frameno: [
      0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
      20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
      38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
      56, 57, 58, 59,
    ],
    post_receive_frame_q_lens: [
      1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
      3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
      3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    ],
  };
  lookahead_size_properly_bounded(LOOKAHEAD_SIZE, true, &EXPECTATIONS);
}

fn lookahead_size_properly_bounded(
  rdo_lookahead: usize, low_latency: bool,
  expectations: &LookaheadTestExpectations,
) {
  // Test that lookahead reads in the proper number of frames at once

  let mut ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    100,
    8,
    ChromaSampling::Cs420,
    0,
    100,
    0,
    low_latency,
    0,
    true,
    rdo_lookahead,
    None,
  );

  const LIMIT: usize = 60;

  let mut post_receive_input_frameno = [0; LIMIT];
  let mut post_receive_frame_q_lens = [0; LIMIT];

  for i in 0..LIMIT {
    let input = ctx.new_frame();
    let _ = ctx.send_frame(input);
    while ctx.receive_packet().is_ok() {
      // Receive packets until lookahead consumed, due to pyramids receiving frames in groups
    }
    post_receive_input_frameno[i] = ctx.inner.input_frameno;
    post_receive_frame_q_lens[i] = ctx.inner.frame_q.len();
  }

  assert_eq!(
    &post_receive_input_frameno[..],
    &expectations.post_receive_input_frameno[..]
  );
  assert_eq!(
    &post_receive_frame_q_lens[..],
    &expectations.post_receive_frame_q_lens[..]
  );

  ctx.flush();
  let end = ctx.inner.frame_q.get(&(LIMIT as u64));
  assert!(end.is_some());
  assert!(end.unwrap().is_none());

  loop {
    match ctx.receive_packet() {
      Ok(_) | Err(EncoderStatus::Encoded) => {
        // Receive packets until all frames consumed
      }
      _ => {
        break;
      }
    }
  }
  assert_eq!(ctx.inner.frames_processed, LIMIT as u64);
}

#[test]
fn zero_frames() {
  let config = Config::default();
  let mut ctx: Context<u8> = config.new_context().unwrap();
  ctx.flush();
  assert_eq!(ctx.receive_packet(), Err(EncoderStatus::LimitReached));
}

#[test]
fn tile_cols_overflow() {
  let mut enc = EncoderConfig::default();
  enc.tile_cols = usize::max_value();
  let config = Config::new().with_encoder_config(enc);
  let _: Result<Context<u8>, _> = config.new_context();
}

#[test]
fn max_key_frame_interval_overflow() {
  let mut enc = EncoderConfig::default();
  enc.max_key_frame_interval = i32::max_value() as u64;
  enc.reservoir_frame_delay = None;
  let config = Config::new().with_encoder_config(enc);
  let _: Result<Context<u8>, _> = config.new_context();
}

#[test]
fn target_bitrate_overflow() {
  let mut enc = EncoderConfig::default();
  enc.bitrate = i32::max_value();
  enc.time_base = Rational::new(i64::max_value() as u64, 1);
  let config = Config::new().with_encoder_config(enc);
  let _: Result<Context<u8>, _> = config.new_context();
}

#[test]
fn time_base_den_divide_by_zero() {
  let mut enc = EncoderConfig::default();
  enc.time_base = Rational::new(1, 0);
  let config = Config::new().with_encoder_config(enc);
  let _: Result<Context<u8>, _> = config.new_context();
}

#[test]
fn large_width_assert() {
  let mut enc = EncoderConfig::default();
  enc.width = u32::max_value() as usize;
  let config = Config::new().with_encoder_config(enc);
  let _: Result<Context<u8>, _> = config.new_context();
}

#[test]
fn reservoir_max_overflow() {
  let mut enc = EncoderConfig::default();
  enc.reservoir_frame_delay = Some(i32::max_value());
  enc.bitrate = i32::max_value();
  enc.time_base = Rational::new(i32::max_value() as u64 * 2, 1);
  let config = Config::new().with_encoder_config(enc);
  let _: Result<Context<u8>, _> = config.new_context();
}

#[test]
fn zero_width() {
  let mut enc = EncoderConfig::default();
  enc.width = 0;
  let config = Config::new().with_encoder_config(enc);
  let res: Result<Context<u8>, _> = config.new_context();
  assert!(res.is_err());
}

#[test]
fn rdo_lookahead_frames_overflow() {
  let mut enc = EncoderConfig::default();
  enc.speed_settings.rdo_lookahead_frames = usize::max_value();
  let config = Config::new().with_encoder_config(enc);
  let res: Result<Context<u8>, _> = config.new_context();
  assert!(res.is_err());
}

#[test]
fn log_q_exp_overflow() {
  let enc = EncoderConfig {
    width: 16,
    height: 16,
    sample_aspect_ratio: Rational::new(1, 1),
    bit_depth: 8,
    chroma_sampling: ChromaSampling::Cs420,
    chroma_sample_position: ChromaSamplePosition::Unknown,
    pixel_range: PixelRange::Limited,
    color_description: None,
    mastering_display: None,
    content_light: None,
    enable_timing_info: false,
    still_picture: false,
    error_resilient: false,
    switch_frame_interval: 0,
    time_base: Rational { num: 1, den: 25 },
    min_key_frame_interval: 12,
    max_key_frame_interval: 240,
    reservoir_frame_delay: None,
    low_latency: false,
    quantizer: 100,
    min_quantizer: 64,
    bitrate: 1,
    tune: Tune::Psychovisual,
    tile_cols: 0,
    tile_rows: 0,
    tiles: 0,
    cpu_feature_level: CpuFeatureLevel::default(),
    speed_settings: SpeedSettings {
      multiref: false,
      fast_deblock: true,
      rdo_lookahead_frames: 40,
      scene_detection_mode: SceneDetectionSpeed::None,
      cdef: true,
      lrf: true,
      partition: PartitionSpeedSettings {
        partition_range: PartitionRange::new(
          BlockSize::BLOCK_64X64,
          BlockSize::BLOCK_64X64,
        ),
        encode_bottomup: false,
        non_square_partition_threshold: BlockSize::BLOCK_64X64,
      },
      transform: TransformSpeedSettings {
        reduced_tx_set: true,
        tx_domain_distortion: true,
        tx_domain_rate: false,
        rdo_tx_decision: false,
        ..Default::default()
      },
      prediction: PredictionSpeedSettings {
        prediction_modes: PredictionModesSetting::Simple,
        ..Default::default()
      },
      motion: MotionSpeedSettings {
        include_near_mvs: false,
        use_satd_subpel: false,
        ..Default::default()
      },
      ..Default::default()
    },
  };
  let config = Config::new().with_encoder_config(enc).with_threads(1);

  let mut ctx: Context<u8> = config.new_context().unwrap();
  for _ in 0..2 {
    ctx.send_frame(ctx.new_frame()).unwrap();
  }
  ctx.flush();

  ctx.receive_packet().unwrap();
  let _ = ctx.receive_packet();
}

#[test]
fn guess_frame_subtypes_assert() {
  let enc = EncoderConfig {
    width: 16,
    height: 16,
    sample_aspect_ratio: Rational::new(1, 1),
    bit_depth: 8,
    chroma_sampling: ChromaSampling::Cs420,
    chroma_sample_position: ChromaSamplePosition::Unknown,
    pixel_range: PixelRange::Limited,
    color_description: None,
    mastering_display: None,
    content_light: None,
    enable_timing_info: false,
    still_picture: false,
    error_resilient: false,
    switch_frame_interval: 0,
    time_base: Rational { num: 1, den: 25 },
    min_key_frame_interval: 0,
    max_key_frame_interval: 1,
    reservoir_frame_delay: None,
    low_latency: false,
    quantizer: 100,
    min_quantizer: 0,
    bitrate: 16384,
    tune: Tune::Psychovisual,
    tile_cols: 0,
    tile_rows: 0,
    tiles: 0,
    cpu_feature_level: CpuFeatureLevel::default(),
    speed_settings: SpeedSettings {
      multiref: false,
      fast_deblock: true,
      rdo_lookahead_frames: 40,
      scene_detection_mode: SceneDetectionSpeed::None,
      cdef: true,
      lrf: true,
      partition: PartitionSpeedSettings {
        partition_range: PartitionRange::new(
          BlockSize::BLOCK_64X64,
          BlockSize::BLOCK_64X64,
        ),
        encode_bottomup: false,
        non_square_partition_threshold: BlockSize::BLOCK_64X64,
      },
      transform: TransformSpeedSettings {
        reduced_tx_set: true,
        tx_domain_distortion: true,
        tx_domain_rate: false,
        rdo_tx_decision: false,
        ..Default::default()
      },
      prediction: PredictionSpeedSettings {
        prediction_modes: PredictionModesSetting::Simple,
        ..Default::default()
      },
      motion: MotionSpeedSettings {
        include_near_mvs: false,
        use_satd_subpel: false,
        ..Default::default()
      },
      ..Default::default()
    },
  };
  let config = Config::new().with_encoder_config(enc).with_threads(1);

  let mut ctx: Context<u8> = config.new_context().unwrap();
  ctx.send_frame(ctx.new_frame()).unwrap();
  ctx.flush();

  ctx.receive_packet().unwrap();
}

#[test]
fn min_quantizer_bounds_correctly() {
  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    255,
    8,
    ChromaSampling::Cs420,
    25,
    25,
    25000,
    true,
    0,
    true,
    1,
    Some(100),
  );

  let limit = 25;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 0 };
  for i in 0..limit {
    sender.process_frame();

    if i == 0 {
      let fi = &sender.ctx.inner.current_keyframe;
      assert_eq!(79, fi.base_q_idx);
    } else {
      let fi = &sender
        .ctx
        .inner
        .current_frame_group
        .iter()
        .find(|fi| fi.as_ref().unwrap().input_frameno == i)
        .unwrap();
      assert_eq!(103, fi.as_ref().unwrap().base_q_idx);
    }
  }

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    255,
    8,
    ChromaSampling::Cs420,
    25,
    25,
    2000,
    true,
    0,
    true,
    1,
    Some(100),
  );

  let limit = 25;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 0 };
  for i in 0..limit {
    sender.process_frame();

    if i == 0 {
      let fi = &sender.ctx.inner.current_keyframe;
      assert!(fi.base_q_idx > 79);
    } else {
      let fi = &sender
        .ctx
        .inner
        .current_frame_group
        .iter()
        .find(|fi| fi.as_ref().unwrap().input_frameno == i)
        .unwrap();
      assert!(fi.as_ref().unwrap().base_q_idx > 103);
    }
  }
}

#[test]
fn max_quantizer_bounds_correctly() {
  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    120,
    8,
    ChromaSampling::Cs420,
    25,
    25,
    2000,
    true,
    0,
    true,
    1,
    None,
  );

  let limit = 25;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 0 };
  for i in 0..limit {
    sender.process_frame();

    if i == 0 {
      let fi = &sender.ctx.inner.current_keyframe;
      assert_eq!(102, fi.base_q_idx);
    } else {
      let fi = &sender
        .ctx
        .inner
        .current_frame_group
        .iter()
        .find(|fi| fi.as_ref().unwrap().input_frameno == i)
        .unwrap();
      assert_eq!(123, fi.as_ref().unwrap().base_q_idx);
    }
  }

  let ctx = setup_encoder::<u8>(
    64,
    80,
    10,
    120,
    8,
    ChromaSampling::Cs420,
    25,
    25,
    20000,
    true,
    0,
    true,
    1,
    None,
  );

  let limit = 25;
  let mut sender =
    TestFrameSender { ctx, limit: limit as u64, scene_change_at: 0 };
  for i in 0..limit {
    sender.process_frame();

    if i == 0 {
      let fi = &sender.ctx.inner.current_keyframe;
      assert!(fi.base_q_idx < 102);
    } else {
      let fi = &sender
        .ctx
        .inner
        .current_frame_group
        .iter()
        .find(|fi| fi.as_ref().unwrap().input_frameno == i)
        .unwrap();
      assert!(fi.as_ref().unwrap().base_q_idx < 123);
    }
  }
}
