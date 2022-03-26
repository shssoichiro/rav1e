use crate::{api::PixelRange, frame::Frame};
use v_frame::math::clamp;
use v_frame::pixel::CastFromPrimitive;
use v_frame::pixel::Pixel;
use v_frame::plane::PlaneConfig;
use v_frame::plane::PlaneOffset;

pub(super) fn apply_retinex_transform<T: Pixel>(
  frame: &Frame<T>, bit_depth: usize, pixel_range: PixelRange,
) -> Frame<T> {
  // Calculate quantization parameters according to bit per sample and limited/full range
  // Floor and Ceil for limited range src will be determined later
  // according to minimum and maximum value in the frame
  let mut src_floor = T::cast_from(0);
  let mut src_ceil = T::cast_from((1 << bit_depth) - 1);
  let dest_floor = match pixel_range {
    PixelRange::Full => 0,
    PixelRange::Limited => 16 << (bit_depth - 8),
  } as f64;
  let range = match pixel_range {
    PixelRange::Full => (1 << bit_depth) - 1,
    PixelRange::Limited => 219 << (bit_depth - 8),
  } as f64;

  let luma_plane = &frame.planes[0];
  let input_data: Box<[f64]>;

  // Derive floating point intensity channel from integer Y channel
  match pixel_range {
    PixelRange::Full => {
      let gain = 1. / range;
      input_data = luma_plane
        .rows_iter()
        .flat_map(|row| {
          row.iter().map(|&pix| u16::cast_from(pix) as f64 * gain)
        })
        .collect();
    }
    PixelRange::Limited => {
      // If src is of limited range, determine the Floor and Ceil by the minimum and maximum value in the frame
      let mut min = src_ceil;
      let mut max = src_floor;
      luma_plane.rows_iter().flat_map(|row| row.iter()).for_each(|&pix| {
        if pix < min {
          min = pix;
        }
        if pix > max {
          max = pix;
        }
      });

      assert!(max > min);
      src_floor = min;
      src_ceil = max;

      let gain = 1. / u16::cast_from(src_ceil - src_floor) as f64;
      input_data = luma_plane
        .rows_iter()
        .flat_map(|row| {
          row.iter().map(|&pix| u16::cast_from(pix - src_floor) as f64 * gain)
        })
        .collect();
    }
  }

  let mut output_data = vec![1.0; input_data.len()].into_boxed_slice();

  // Apply MSR to floating point intensity channel
  msr_kernel(&input_data, &mut output_data, &luma_plane.cfg);
  // Simplest color balance with pixel clipping on either side of the dynamic range
  simplest_color_balance(&mut output_data);

  // The full implementation of retinex has a chroma protection step.
  // Since we only use the luma plane for scenechange, we skip that step.
  let mut output_frame = frame.clone();
  let offset_y = dest_floor + 0.5;
  output_frame.planes[0]
    .mut_slice(PlaneOffset::default())
    .rows_iter_mut()
    .flat_map(|row| row.iter_mut())
    .zip(output_data.iter())
    .for_each(|(out_pix, &in_pix)| {
      *out_pix = T::cast_from(in_pix.mul_add(range, offset_y) as u16);
    });

  output_frame
}

const SIGMA: [f64; 3] = [25., 80., 250.];

fn msr_kernel(input: &[f64], output: &mut [f64], cfg: &PlaneConfig) {
  for sigma in SIGMA {
    let mut gauss = vec![0.; input.len()].into_boxed_slice();
    let parameters = recursive_gaussian_parameters(sigma);
    recursive_gaussian_2d_horizontal(input, &mut gauss, cfg, parameters);
    recursive_gaussian_2d_vertical(&mut gauss, cfg, parameters);

    output.iter_mut().zip(input.iter()).zip(gauss.iter()).for_each(
      |((out_pix, &in_pix), &gauss)| {
        if gauss > 0. {
          *out_pix *= in_pix / gauss + 1.;
        }
      },
    );
  }
  output.iter_mut().for_each(|pix| {
    *pix = pix.ln() / (SIGMA.len() as f64);
  });
}

#[derive(Debug, Clone, Copy)]
struct GaussParameters {
  beta: f64,
  beta1: f64,
  beta2: f64,
  beta3: f64,
}

fn recursive_gaussian_parameters(sigma: f64) -> GaussParameters {
  let q = if sigma < 2.5 {
    3.97156 - 4.14554 * (1. - 0.26891 * sigma).sqrt()
  } else {
    0.98711 * sigma - 0.96330
  };

  let b0 = (0.422205 * q * q)
    .mul_add(q, (1.4281 * q).mul_add(q, 2.44413f64.mul_add(q, 1.57825)));
  let b1 =
    (1.26661 * q * q).mul_add(q, 2.44413f64.mul_add(q, 2.85619 * q * q));
  let b2 = -(1.4281 * q).mul_add(q, 1.26661 * q * q * q);
  let b3 = 0.422205 * q * q * q;

  GaussParameters {
    beta: 1. - (b1 + b2 + b3) / b0,
    beta1: b1 / b0,
    beta2: b2 / b0,
    beta3: b3 / b0,
  }
}

fn recursive_gaussian_2d_horizontal(
  input: &[f64], output: &mut [f64], cfg: &PlaneConfig,
  params: GaussParameters,
) {
  for j in 0..cfg.height {
    let lower = cfg.width * j;
    let upper = lower + cfg.width;

    let mut p = input[lower];
    output[lower] = p;

    for i in lower..upper {
      // P0 = B*input[i] + B1*P1 + B2*P2 + B3*P3;
      p = params.beta3.mul_add(
        p,
        params
          .beta2
          .mul_add(p, params.beta.mul_add(input[i], params.beta1 * p)),
      );
      output[i] = p;
    }

    p = output[upper - 1];

    for i in (lower..upper).rev() {
      // P0 = B*output[i] + B1*P1 + B2*P2 + B3*P3;
      p = params.beta3.mul_add(
        p,
        params
          .beta2
          .mul_add(p, params.beta.mul_add(output[i], params.beta1 * p)),
      );
      output[i] = p;
    }
  }
}

fn recursive_gaussian_2d_vertical(
  data: &mut [f64], cfg: &PlaneConfig, params: GaussParameters,
) {
  for j in 0..cfg.height {
    let lower = cfg.width * j;
    let upper = lower + cfg.width;

    let mut i_0 = lower;
    let mut i_1 = if j < 1 { i_0 } else { i_0 - cfg.width };
    let mut i_2 = if j < 2 { i_1 } else { i_1 - cfg.width };
    let mut i_3 = if j < 3 { i_2 } else { i_2 - cfg.width };

    while i_0 < upper {
      let p3 = data[i_3];
      let p2 = data[i_2];
      let p1 = data[i_1];
      let p0 = data[i_0];
      // output[i0] = B*P0 + B1*P1 + B2*P2 + B3*P3;
      data[i_0] = params.beta3.mul_add(
        p3,
        params.beta2.mul_add(p2, params.beta.mul_add(p0, params.beta1 * p1)),
      );

      i_0 += 1;
      i_1 += 1;
      i_2 += 1;
      i_3 += 1;
    }
  }

  for j in (0..cfg.height).rev() {
    let lower = cfg.width * j;
    let upper = lower + cfg.width;

    let mut i_0 = lower;
    let mut i_1 = if j >= cfg.height - 1 { i_0 } else { i_0 + cfg.width };
    let mut i_2 = if j >= cfg.height - 2 { i_1 } else { i_1 + cfg.width };
    let mut i_3 = if j >= cfg.height - 3 { i_2 } else { i_2 + cfg.width };

    while i_0 < upper {
      let p3 = data[i_3];
      let p2 = data[i_2];
      let p1 = data[i_1];
      let p0 = data[i_0];
      // output[i0] = B*P0 + B1*P1 + B2*P2 + B3*P3;
      data[i_0] = params.beta3.mul_add(
        p3,
        params.beta2.mul_add(p2, params.beta.mul_add(p0, params.beta1 * p1)),
      );

      i_0 += 1;
      i_1 += 1;
      i_2 += 1;
      i_3 += 1;
    }
  }
}

const LOWER_THR: f64 = 0.001;
const UPPER_THR: f64 = 0.001;
const HIST_BINS: usize = 4096;

fn simplest_color_balance(data: &mut [f64]) {
  let mut min = f64::MAX;
  let mut max = f64::MIN;
  data.iter().for_each(|&pix| {
    if pix < min {
      min = pix;
    }
    if pix > max {
      max = pix;
    }
  });
  assert!(max > min);

  let mut histogram = [0u32; HIST_BINS];
  let gain = (HIST_BINS - 1) as f64 / (max - min);
  let offset = -min * gain;

  data.iter().for_each(|&pix| {
    histogram[pix.mul_add(gain, offset) as usize] += 1;
  });

  let gain = (max - min) / (HIST_BINS - 1) as f64;
  let offset = min;
  let max_count = (data.len() as f64).mul_add(LOWER_THR, 0.5) as u32;
  let mut count = 0;
  let mut h = HIST_BINS;
  for (i, pix) in histogram.into_iter().enumerate() {
    count += pix;
    if count > max_count {
      h = i;
      break;
    }
  }
  min = (h as f64).mul_add(gain, offset);

  let max_count = (data.len() as f64).mul_add(UPPER_THR, 0.5) as u32;
  let mut count = 0;
  let mut h = 0;
  for (i, pix) in histogram.into_iter().enumerate().rev() {
    count += pix;
    if count > max_count {
      h = i;
      break;
    }
  }
  max = (h as f64).mul_add(gain, offset);

  let gain = 1.0 / (max - min);
  let offset = -min * gain;
  data.iter_mut().for_each(|pix| {
    *pix = clamp((*pix).mul_add(gain, offset), 0., 1.);
  });
}
