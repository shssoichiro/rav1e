// Copyright (c) 2001-2016, Alliance for Open Media. All rights reserved
// Copyright (c) 2017-2020, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

use std::io::Read;

use crate::decoder::{DecodeError, Decoder, VideoDetails};
use crate::Frame;
use rav1e::prelude::*;

impl Decoder for y4m::Decoder<Box<dyn Read + Send>> {
  fn get_video_details(&self) -> VideoDetails {
    let width = self.get_width();
    let height = self.get_height();
    let aspect_ratio = self.get_pixel_aspect();
    let color_space = self.get_colorspace();
    let bit_depth = color_space.get_bit_depth();
    let (chroma_sampling, chroma_sample_position) =
      map_y4m_color_space(color_space);
    let framerate = self.get_framerate();
    let time_base = Rational::new(framerate.den as u64, framerate.num as u64);

    VideoDetails {
      width,
      height,
      sample_aspect_ratio: if aspect_ratio.num == 0 && aspect_ratio.den == 0 {
        Rational::new(1, 1)
      } else {
        Rational::new(aspect_ratio.num as u64, aspect_ratio.den as u64)
      },
      bit_depth,
      chroma_sampling,
      chroma_sample_position,
      time_base,
    }
  }

  fn read_frame<T: Pixel>(&mut self) -> Result<Frame<T>, DecodeError> {
    let details = self.get_video_details();
    self
      .read_frame()
      .map(|frame| {
        // SAFETY: We do not reuse the input frame after this,
        // we essentially are moving it into the output Frame
        unsafe {
          Frame::new_zerocopy(
            [frame.get_y_plane(), frame.get_u_plane(), frame.get_v_plane()],
            details.width,
            details.height,
            details.chroma_sampling,
          )
        }
      })
      .map_err(Into::into)
  }
}

impl From<y4m::Error> for DecodeError {
  fn from(e: y4m::Error) -> DecodeError {
    match e {
      y4m::Error::EOF => DecodeError::EOF,
      y4m::Error::BadInput => DecodeError::BadInput,
      y4m::Error::UnknownColorspace => DecodeError::UnknownColorspace,
      y4m::Error::ParseError(_) => DecodeError::ParseError,
      y4m::Error::IoError(e) => DecodeError::IoError(e),
      // Note that this error code has nothing to do with the system running out of memory,
      // it means the y4m decoder has exceeded its memory allocation limit.
      y4m::Error::OutOfMemory => DecodeError::MemoryLimitExceeded,
    }
  }
}

pub const fn map_y4m_color_space(
  color_space: y4m::Colorspace,
) -> (ChromaSampling, ChromaSamplePosition) {
  use crate::ChromaSamplePosition::*;
  use crate::ChromaSampling::*;
  use y4m::Colorspace::*;
  match color_space {
    Cmono => (Cs400, Unknown),
    C420jpeg | C420paldv => (Cs420, Unknown),
    C420mpeg2 => (Cs420, Vertical),
    C420 | C420p10 | C420p12 => (Cs420, Colocated),
    C422 | C422p10 | C422p12 => (Cs422, Colocated),
    C444 | C444p10 | C444p12 => (Cs444, Colocated),
  }
}
