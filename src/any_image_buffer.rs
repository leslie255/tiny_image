use std::{
    alloc::{self, Layout},
    error::Error,
    fmt::{self, Display},
    mem::forget,
    ptr::{NonNull, copy_nonoverlapping},
    slice,
};

use crate::*;

pub struct AnyImageBuffer {
    width: u32,
    height: u32,
    data: NonNull<u8>,
    format: PixelFormat,
}

impl Drop for AnyImageBuffer {
    fn drop(&mut self) {
        unsafe { Self::dealloc(self.width, self.height, self.format, self.data) };
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum PngDecodeError {
    ZunePngError(zune_png::error::PngDecodeErrors),
    ZeroWidthOrHeight,
    UnsupportedPngColorspace(zune_png::zune_core::colorspace::ColorSpace),
    UnknownBitdepth,
    WidthTooLarge(usize),
    HeightTooLarge(usize),
}

impl From<zune_png::error::PngDecodeErrors> for PngDecodeError {
    fn from(v: zune_png::error::PngDecodeErrors) -> Self {
        Self::ZunePngError(v)
    }
}

impl Display for PngDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ZunePngError(error) => Display::fmt(error, formatter),
            Self::ZeroWidthOrHeight => write!(formatter, "width or height is zero"),
            Self::UnsupportedPngColorspace(colorspace) => {
                use zune_png::zune_core::colorspace::ColorSpace;
                let colorspace = fmt::from_fn(|f| match colorspace {
                    ColorSpace::RGB => write!(f, "RGB"),
                    ColorSpace::RGBA => write!(f, "RGBA"),
                    ColorSpace::YCbCr => write!(f, "YCbCr"),
                    ColorSpace::Luma => write!(f, "Luma"),
                    ColorSpace::LumaA => write!(f, "LumaA"),
                    ColorSpace::YCCK => write!(f, "YCCK"),
                    ColorSpace::CMYK => write!(f, "CMYK"),
                    ColorSpace::BGR => write!(f, "BGR"),
                    ColorSpace::BGRA => write!(f, "BGRA"),
                    ColorSpace::Unknown => write!(f, "Unknown"),
                    ColorSpace::ARGB => write!(f, "ARGB"),
                    ColorSpace::HSL => write!(f, "HSL"),
                    ColorSpace::HSV => write!(f, "HSV"),
                    ColorSpace::MultiBand(n) => write!(f, "multiband {n}"),
                    _ => todo!(),
                });
                write!(formatter, "unsupported PNG colorspace: {colorspace}")
            }
            Self::UnknownBitdepth => write!(formatter, "unknown bitdepth"),
            Self::WidthTooLarge(u) => {
                write!(formatter, "width ({u}) is too large to fit into u32")
            }
            Self::HeightTooLarge(u) => {
                write!(formatter, "height ({u}) is too large to fit into u32")
            }
        }
    }
}

impl Error for PngDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ZunePngError(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum QoiDecodeError {
    QoiError(qoi::Error),
    ZeroWidthOrHeight,
    WidthTooLarge(usize),
    HeightTooLarge(usize),
}

impl From<qoi::Error> for QoiDecodeError {
    fn from(v: qoi::Error) -> Self {
        Self::QoiError(v)
    }
}

impl Display for QoiDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::QoiError(error) => Display::fmt(error, formatter),
            Self::ZeroWidthOrHeight => write!(formatter, "width or height is zero"),
            Self::WidthTooLarge(u) => {
                write!(formatter, "width ({u}) is too large to fit into u32")
            }
            Self::HeightTooLarge(u) => {
                write!(formatter, "height ({u}) is too large to fit into u32")
            }
        }
    }
}

impl Error for QoiDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            QoiDecodeError::QoiError(error) => Some(error),
            _ => None,
        }
    }
}

impl AnyImageBuffer {
    /// # Safety
    ///
    /// - `width` and `height` must be both non-zero.
    /// - `buffer` must be a value returned by the global allocator
    /// - `buffer` must not have any extra capacity at its tail
    pub unsafe fn from_raw_parts(
        width: u32,
        height: u32,
        data: *mut u8,
        format: PixelFormat,
    ) -> Self {
        debug_assert!(width != 0);
        debug_assert!(height != 0);
        Self {
            width,
            height,
            data: unsafe { NonNull::new_unchecked(data) },
            format,
        }
    }

    /// # Panics
    ///
    /// - if either `width` and `height` are zero
    pub fn new_zeroed(width: u32, height: u32, format: PixelFormat) -> Self {
        assert!(width != 0);
        assert!(height != 0);
        Self {
            width,
            height,
            data: Self::alloc(width, height, format, true),
            format,
        }
    }

    pub fn from_image_ref<PF: PixelFormatTrait>(image: ImageRef<PF>) -> Self {
        let data = Self::alloc(image.width(), image.height(), image.format(), false);
        unsafe {
            copy_nonoverlapping::<u8>(
                image.as_image_ptr().as_bytes_ptr(), // src
                data.as_ptr(),                       // dst
                image.as_image_ptr().bytes_count(),  // count
            );
        }
        Self {
            width: image.width(),
            height: image.height(),
            data,
            format: image.format(),
        }
    }

    pub fn from_image_buffer<PF: PixelFormatTrait>(image: ImageBuffer<PF>) -> Self {
        Self {
            width: image.width(),
            height: image.height(),
            data: NonNull::new(image.as_image_ptr().as_bytes_ptr()).unwrap(),
            format: PF::FORMAT_ENUM,
        }
    }

    pub fn into_vec(self) -> Vec<u8> {
        unsafe { Vec::from_raw_parts(self.as_bytes_ptr(), self.bytes_count(), self.bytes_count()) }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> PixelFormat {
        self.format
    }

    pub fn as_bytes_ptr(&self) -> *mut u8 {
        self.data.as_ptr()
    }

    pub fn pixel_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub fn bytes_count(&self) -> usize {
        self.pixel_count() * self.format.bytes_per_pixel()
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data.as_ptr(), self.bytes_count()) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr(), self.bytes_count()) }
    }

    pub fn decode_from_png(content: &[u8]) -> Result<Self, PngDecodeError> {
        use zune_png::{
            PngDecoder,
            zune_core::{
                bit_depth::BitDepth, bytestream::ZCursor, colorspace::ColorSpace,
                options::DecoderOptions, result::DecodingResult,
            },
        };
        let mut decoder = PngDecoder::new_with_options(
            ZCursor::new(content),
            DecoderOptions::default().png_set_strip_to_8bit(true),
        );

        let decode_result = decoder.decode()?;

        // Uses of unwrap:
        // `zune_png`'s documentation guarantees `Some` after decode.
        let (width_, height_) = decoder.dimensions().unwrap();
        let width: u32 = width_
            .try_into()
            .map_err(|_| PngDecodeError::WidthTooLarge(width_))?;
        let height: u32 = height_
            .try_into()
            .map_err(|_| PngDecodeError::HeightTooLarge(height_))?;
        let depth = decoder.depth().unwrap();
        let colorspace = decoder.colorspace().unwrap();
        let format: PixelFormat = match (depth, colorspace) {
            (BitDepth::Eight, ColorSpace::RGB) => PixelFormat::Rgb8U,
            (BitDepth::Eight, ColorSpace::RGBA) => PixelFormat::Rgba8U,
            (BitDepth::Eight, ColorSpace::Luma) => PixelFormat::Luma8U,
            (BitDepth::Float32, ColorSpace::RGB) => PixelFormat::Rgb32F,
            (BitDepth::Float32, ColorSpace::RGBA) => PixelFormat::Rgba32F,
            (BitDepth::Float32, ColorSpace::Luma) => PixelFormat::Luma32F,
            (BitDepth::Sixteen, _) => unreachable!(), // we set `png_set_strip_to_8bit` earlier
            (BitDepth::Unknown, _) => return Err(PngDecodeError::UnknownBitdepth),
            (_, colorspace) => return Err(PngDecodeError::UnsupportedPngColorspace(colorspace)),
        };
        let data: *mut u8 = match decode_result {
            DecodingResult::U8(mut vec) => {
                vec.shrink_to_fit();
                vec.into_raw_parts().0
            }
            DecodingResult::U16(_) => unreachable!(), // we set `png_set_strip_to_8bit` earlier
            DecodingResult::F32(mut vec) => {
                vec.shrink_to_fit();
                vec.into_raw_parts().0 as *mut u8
            }
            // DecodingResult is #[non_exhaustive]
            _ => panic!("unsupported `DecodingResult` enum variant returned from zune_png"),
        };

        Ok(Self {
            width,
            height,
            data: NonNull::new(data).unwrap(),
            format,
        })
    }

    pub fn decode_from_qoi(content: &[u8]) -> Result<Self, QoiDecodeError> {
        let mut decoder = qoi::Decoder::new(content)?;
        let data = decoder.decode_to_vec()?;
        let header = decoder.header();
        if header.width == 0 || header.height == 0 {
            return Err(QoiDecodeError::ZeroWidthOrHeight);
        }
        let format = match header.channels {
            qoi::Channels::Rgb => PixelFormat::Rgb8U,
            qoi::Channels::Rgba => PixelFormat::Rgba8U,
        };
        Ok(Self {
            width: header.width,
            height: header.height,
            data: NonNull::new(data.into_raw_parts().0).unwrap(),
            format,
        })
    }

    pub fn as_image_ptr<PF: PixelFormatTrait>(&self) -> Option<ImagePtr<PF>> {
        if self.format == PF::FORMAT_ENUM {
            Some(unsafe {
                ImagePtr::from_raw_parts(self.width, self.height, self.data.as_ptr() as *mut _)
            })
        } else {
            None
        }
    }

    pub fn as_image_buffer<PF: PixelFormatTrait>(self) -> Option<ImageBuffer<PF>> {
        let image_buffer = self
            .as_image_ptr()
            .map(|ptr| unsafe { ImageBuffer::from_image_ptr(ptr) });
        forget(self);
        image_buffer
    }

    pub fn as_image_ref<PF: PixelFormatTrait>(&self) -> Option<ImageRef<'_, PF>> {
        self.as_image_ptr::<PF>()
            .map(|ptr| unsafe { ptr.as_image_ref() })
    }

    pub fn as_image_mut<PF: PixelFormatTrait>(&self) -> Option<ImageMut<'_, PF>> {
        self.as_image_ptr::<PF>()
            .map(|ptr| unsafe { ptr.as_image_mut() })
    }

    fn alloc(width: u32, height: u32, format: PixelFormat, zeroed: bool) -> NonNull<u8> {
        let n_pixels = width as usize * height as usize;
        let layout = Layout::array::<u8>(n_pixels * format.bytes_per_pixel()).unwrap();
        let buffer = match zeroed {
            true => unsafe { alloc::alloc_zeroed(layout) },
            false => unsafe { alloc::alloc(layout) },
        };
        NonNull::new(buffer).unwrap_or_else(|| {
            panic!("allocator returned null in AnyImageBuffer::alloc");
        })
    }

    /// # Safety
    ///
    /// - `raw` cannot have any spare capacity at the end
    unsafe fn dealloc(width: u32, height: u32, format: PixelFormat, buffer: NonNull<u8>) {
        let n_pixels = width as usize * height as usize;
        let layout = Layout::array::<u8>(n_pixels * format.bytes_per_pixel()).unwrap();
        unsafe { alloc::dealloc(buffer.as_ptr(), layout) };
    }

    /// Convert to the specified pixel format. If not already that format.
    pub fn into_format_lossy<PF: PixelFormatTrait>(self) -> ImageBuffer<PF> {
        if PF::FORMAT_ENUM == self.format {
            self.as_image_buffer().unwrap()
        } else {
            let mut result = ImageBuffer::<PF>::new_zeroed(self.width, self.height);
            unsafe {
                format_convert::convert(
                    self.format,
                    self.as_bytes(),
                    PF::FORMAT_ENUM,
                    result.as_bytes_mut(),
                );
            };
            result
        }
    }
}
