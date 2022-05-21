use crate::api::FrameQueue;
use crate::util::Aligned;
use crate::EncoderStatus;
use arrayvec::ArrayVec;
use ndarray::{Array3, ArrayView3, ArrayViewMut3};
use ndrustfft::{
  ndfft, ndfft_r2c, ndifft, ndifft_r2c, FftHandler, R2cFftHandler,
};
use std::collections::{BTreeMap, VecDeque};
use std::f64::consts::PI;
use std::iter::once;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;
use std::sync::Arc;
use v_frame::frame::Frame;
use v_frame::math::clamp;
use v_frame::pixel::{CastFromPrimitive, ChromaSampling, Pixel};
use v_frame::plane::Plane;

pub type Complex = ndrustfft::Complex<f32>;

const BETA: f32 = 1.0;
// Assume we are working with square blocks
const BLOCK_DIMENSION: usize = 32;
const BLOCK_OVERLAP: usize = BLOCK_DIMENSION / 3;
const BLOCK_INTERVAL: usize = BLOCK_DIMENSION - BLOCK_OVERLAP;
const KRATIO: f32 = 2.0;
const TEMPORAL_SIZE: usize = 3;

/// This denoiser is based on the FFT3DFilter plugin from Vapoursynth.
/// This type of denoising was chosen because it provides
/// high quality while not being too slow.
pub(crate) struct FftDenoiser<T>
where
  T: Pixel,
{
  chroma_sampling: ChromaSampling,

  /// This stores a copy of the unfiltered previous frame,
  /// since in `frame_q` it will be filtered already.
  previous_frame: Option<Arc<Frame<T>>>,
  pub(crate) cur_frameno: u64,
}

impl<T> FftDenoiser<T>
where
  T: Pixel,
{
  // This should only need to run once per video.
  pub fn new(
    sigma: f32, width: usize, height: usize, bit_depth: u8,
    chroma_sampling: ChromaSampling,
  ) -> Self {
    if size_of::<T>() == 1 {
      assert!(bit_depth <= 8);
    } else {
      assert!(bit_depth > 8);
    }

    let num_planes =
      if chroma_sampling == ChromaSampling::Cs400 { 1 } else { 3 };

    let mut out_nodes: ArrayVec<_, 3> = ArrayVec::new();
    for plane in 0..num_planes {
      let transform =
        FftTransform::new(plane, width, height, bit_depth, chroma_sampling);
      //     FFT3DFilterTransform *transform = new FFT3DFilterTransform(
      //         false, vsapi->addNodeRef(node), plane, wintype, bw, bh, ow, oh, px,
      //         py, pcutoff, degrid, interlaced, measure, ncpu, core, vsapi);

      //     VSFilterDependency deps1[] = {{node, 1}};
      //     VSNode *transformednode = vsapi->createVideoFilter2(
      //         ("FFT3DFilterTrans" + std::to_string(plane)).c_str(),
      //         transform->GetOutputVI(), FFT3DFilterTransform::GetFrame,
      //         FFT3DFilterTransform::Free, fmParallel, deps1, 1, transform, core);

      //     FFT3DFilter *mainFilter = new FFT3DFilter(
      //         transform, vi, sigma1, beta, plane, bw, bh, bt, ow, oh, kratio,
      //         sharpen, scutoff, svr, smin, smax, pframe, px, py, pshow, pcutoff,
      //         pfactor, sigma2, sigma3, sigma4, degrid, dehalo, hr, ht, ncpu,
      //         transformednode, core, vsapi);

      //     VSFilterDependency deps2[] = {{transformednode, bt <= 1}};
      //     VSNode *mainnode = vsapi->createVideoFilter2(
      //         ("FFT3DFilterMain" + std::to_string(plane)).c_str(),
      //         vsapi->getVideoInfo(transformednode), FFT3DFilter::GetFrame,
      //         FFT3DFilter::Free, bt == 0 ? fmParallelRequests : fmParallel, deps2,
      //         1, mainFilter, core);

      //     FFT3DFilterInvTransform *invtransform = new FFT3DFilterInvTransform(
      //         mainnode, vi, plane, wintype, bw, bh, ow, oh, interlaced, measure,
      //         ncpu, core, vsapi);

      //     VSFilterDependency deps3[] = {{mainnode, 1}};
      //     vsapi->createVideoFilter(
      //         vi->format.numPlanes == 1 ? out : tmp,
      //         ("FFT3DFilterInv" + std::to_string(plane)).c_str(),
      //         invtransform->GetOutputVI(), FFT3DFilterInvTransform::GetFrame,
      //         FFT3DFilterInvTransform::Free, fmParallel, deps3, 1, invtransform,
      //         core);

      if num_planes > 1 {
        //         outnodes[plane] = vsapi->mapGetNode(tmp, "clip", 0, nullptr);
        //         vsapi->clearMap(tmp);
      }
    }

    if num_planes > 1 {
      // for (int plane = 0; plane < vi->format.numPlanes; plane++)
      //     vsapi->mapConsumeNode(tmp, "clips", outnodes[plane], maAppend);
      // int64_t pvals[] = {0, process[1] ? 0 : 1, process[2] ? 0 : 2};
      // vsapi->mapSetIntArray(tmp, "planes", pvals, 3);
      // vsapi->mapSetInt(tmp, "colorfamily", vi->format.colorFamily, maAppend);
      // VSMap *tmp2 =
      //     vsapi->invoke(vsapi->getPluginByID("com.vapoursynth.std", core),
      //                     "ShufflePlanes", tmp);
      // vsapi->mapConsumeNode(
      //     out, "clip", vsapi->mapGetNode(tmp2, "clip", 0, nullptr), maAppend);
      // vsapi->freeMap(tmp2);
    }

    FftDenoiser { chroma_sampling, previous_frame: None, cur_frameno: 0 }
  }

  pub fn filter_frame(
    &mut self, frame_q: &FrameQueue<T>,
  ) -> Result<Frame<T>, EncoderStatus> {
    if self.previous_frame.is_none() {
      // We need to have the previous unfiltered frame
      // in the buffer for temporal filtering.
      return Err(EncoderStatus::NeedMoreData);
    }
    let future_frame = frame_q.get(&(self.cur_frameno + 1));
    if future_frame.is_none() {
      // We also need to have the next unfiltered frame,
      // unless we are at the end of the video.
      return Err(EncoderStatus::NeedMoreData);
    }

    let orig_frame = frame_q.get(&self.cur_frameno).unwrap().as_ref().unwrap();
    let mut frames: Vec<&Frame<T>> = Vec::with_capacity(TEMPORAL_SIZE);
    frames.push(self.previous_frame.as_ref().unwrap());
    frames.push(&orig_frame);
    if let Some(frame) = future_frame.and_then(|f| f.as_ref()) {
      frames.push(&frame);
    }

    let mut dest = (**orig_frame).clone();

    self.do_filtering(todo!(), &mut dest);

    self.previous_frame = Some(Arc::clone(orig_frame));
    self.cur_frameno += 1;

    Ok(dest)
  }

  fn do_filtering(&mut self, src: &[[Plane<T>; 3]], dest: &mut Frame<T>) {
    todo!();
  }
}

struct FftTransform {
  plane_idx: usize,
  n_overlap_x: usize,
  n_overlap_y: usize,
  wanxl: [f32; BLOCK_OVERLAP],
  wanxr: [f32; BLOCK_OVERLAP],
  wanyl: [f32; BLOCK_OVERLAP],
  wanyr: [f32; BLOCK_OVERLAP],
  mirror_w: usize,
  mirror_h: usize,
  cover_width: usize,
  cover_height: usize,
  cover_pitch: usize,
  cover_buf: Box<[u8]>,
  in_buf: Box<[f32]>,
  out_width: usize,
  out_pitch_elems: usize,
  out_size: usize,
  dest_buf: Box<[Complex]>,
  plan: Plan,
}

impl FftTransform {
  pub fn new<T: Pixel>(
    plane_idx: usize, width: usize, height: usize, bit_depth: u8,
    chroma_sampling: ChromaSampling,
  ) -> Self {
    let plane_base = if plane_idx > 0 { 1 << (bit_depth - 1) } else { 0 };

    let dec = if plane_idx > 0 {
      chroma_sampling.get_decimation().unwrap()
    } else {
      (0, 0)
    };
    let mut n_overlap_x = ((width >> dec.0) - BLOCK_OVERLAP
      + (BLOCK_INTERVAL - 1))
      / BLOCK_INTERVAL;
    let mut n_overlap_y = ((height >> dec.1) - BLOCK_OVERLAP
      + (BLOCK_INTERVAL - 1))
      / BLOCK_INTERVAL;

    let mut wanxl = [0f32; BLOCK_OVERLAP];
    let mut wanxr = [0f32; BLOCK_OVERLAP];
    let mut wanyl = [0f32; BLOCK_OVERLAP];
    let mut wanyr = [0f32; BLOCK_OVERLAP];

    get_analysis_window(&mut wanxl, &mut wanxr, &mut wanyl, &mut wanyr);

    // padding by 1 block per side
    n_overlap_x += 2;
    n_overlap_y += 2;
    // set mirror size as block interval
    let mirror_w = BLOCK_INTERVAL;
    let mirror_h = BLOCK_INTERVAL;

    let cover_width = n_overlap_x * BLOCK_INTERVAL + BLOCK_OVERLAP;
    let cover_height = n_overlap_y * BLOCK_INTERVAL + BLOCK_OVERLAP;
    let cover_pitch = ((cover_width + 7) / 8) * 8 * size_of::<T>();
    let mut cover_buf = vec![0; cover_height * cover_pitch].into_boxed_slice();

    let in_size =
      BLOCK_DIMENSION * BLOCK_DIMENSION * n_overlap_x * n_overlap_y;
    let mut in_buf = vec![0f32; in_size].into_boxed_slice();
    // width (pitch) of complex fft block
    let out_width = BLOCK_DIMENSION / 2 + 1;
    let out_pitch_elems = ((out_width + 1) / 2) * 2;
    let out_size =
      out_pitch_elems * BLOCK_DIMENSION * n_overlap_x * n_overlap_y;
    let dims = (BLOCK_DIMENSION, BLOCK_DIMENSION);
    let in_dist = BLOCK_DIMENSION * BLOCK_DIMENSION;
    let out_dist = out_pitch_elems * BLOCK_DIMENSION;
    let in_embed = (BLOCK_DIMENSION, BLOCK_DIMENSION);
    let out_embed = (BLOCK_DIMENSION, out_pitch_elems);
    let block_count = n_overlap_x * n_overlap_y;

    let dest_buf = vec![Complex::default(); out_size].into_boxed_slice();
    let plan = plan_real_to_complex_fft(
      2,
      dims,
      block_count,
      &mut in_buf,
      in_embed,
      1,
      in_dist,
      &mut dest_buf,
      out_embed,
      1,
      out_dist,
    );

    Self {
      plane_idx,
      n_overlap_x,
      n_overlap_y,
      wanxl,
      wanxr,
      wanyl,
      wanyr,
      mirror_w,
      mirror_h,
      cover_width,
      cover_height,
      cover_pitch,
      cover_buf,
      in_buf,
      out_width,
      out_pitch_elems,
      out_size,
      dest_buf,
      plan,
    }
  }
}

fn plan_real_to_complex_fft(
  rank: usize, dims: (usize, usize), how_many: usize, input: &mut [f32],
  in_embed: (usize, usize), istride: usize, idist: usize,
  output: &mut [Complex], out_embed: (usize, usize), ostride: usize,
  odist: usize,
) {
  todo!()
}
