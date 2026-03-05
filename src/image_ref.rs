use std::{
    fmt::{self, Debug},
    io::{self, Write as _},
    marker::PhantomData,
    slice,
};

use base64::prelude::*;

use crate::*;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ImageRef<'a, PF: PixelFormatTrait> {
    raw: ImagePtr<PF>,
    _marker: PhantomData<&'a [PF::Primitive]>,
}

impl<'a, PF: PixelFormatTrait> Debug for ImageRef<'a, PF> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ImageRef")
            .field("width", &self.width())
            .field("height", &self.height())
            .field("data", &self.raw.as_primitives_ptr())
            .finish()
    }
}

impl<'a, PF: PixelFormatTrait> ImageRef<'a, PF> {
    /// # Safety
    ///
    /// - `width` and `height` must both be non-zero
    /// - `data` must satisfy the safety contract for `std::slice::from_raw_parts<'a, _>(data, len)`,
    ///   where `len` is width * height * PF::DEPTH
    pub const unsafe fn from_raw_parts(
        width: u32,
        height: u32,
        data: *const PF::Primitive,
    ) -> Self {
        Self {
            raw: unsafe { ImagePtr::from_raw_parts(width, height, data as *mut _) },
            _marker: PhantomData,
        }
    }

    /// Returns `None` if:
    /// - `data` is not large enough to accommodate width * height * PF::DEPTH
    /// - `width` or `height` is zero
    pub const fn new(width: u32, height: u32, data: &'a [PF::Primitive]) -> Option<Self> {
        let n_bytes = width as usize * height as usize * PF::DEPTH;
        if data.len() >= n_bytes && n_bytes != 0 {
            Some(unsafe { Self::from_raw_parts(width, height, data.as_ptr()) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// - `x` and `y` must be in range (`0..width` and `0..height`, respectively)
    pub const unsafe fn get_pixel_unchecked(&self, x: u32, y: u32) -> PF::PrimitiveArray {
        unsafe { *self.raw.get_pixel_unchecked(x, y) }
    }

    pub const fn get_pixel(&self, x: u32, y: u32) -> Option<PF::PrimitiveArray> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.get_pixel_unchecked(x, y) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// - `y` must be in range
    pub const unsafe fn get_row_unchecked(&self, y: u32) -> &[PF::PrimitiveArray] {
        unsafe { slice::from_raw_parts(self.raw.get_row_unchecked(y), self.width() as usize) }
    }

    pub const fn get_row(&self, y: u32) -> Option<&[PF::PrimitiveArray]> {
        if y < self.height() {
            Some(unsafe { self.get_row_unchecked(y) })
        } else {
            None
        }
    }

    pub const fn as_image_ptr(&self) -> ImagePtr<PF> {
        self.raw
    }

    pub const fn reborrow(&self) -> ImageRef<'_, PF> {
        *self
    }

    pub const fn width(&self) -> u32 {
        self.raw.width()
    }

    pub const fn height(&self) -> u32 {
        self.raw.height()
    }

    pub const fn as_bytes(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.raw.as_bytes_ptr(), self.raw.bytes_count()) }
    }

    pub const fn as_primitives(&self) -> &'a [PF::Primitive] {
        unsafe {
            std::slice::from_raw_parts(self.raw.as_primitives_ptr(), self.raw.primitives_count())
        }
    }

    pub const fn as_pixels(&self) -> &'a [PF::PrimitiveArray] {
        unsafe { std::slice::from_raw_parts(self.raw.as_pixels_ptr(), self.raw.pixels_count()) }
    }

    pub const fn format(&self) -> PixelFormat {
        PF::FORMAT_ENUM
    }

    pub fn encode_qoi(&self, out: &mut Vec<u8>)
    where
        PF: QoiCompatiblePixelFormat,
    {
        // Use of unwrap:
        // - ImageRef is a proof-carrying type that ensures `self.as_bytes()` is of adequate
        //   length.
        // - `PF: QoiCompatiblePixelFormat` is proof that this image is only 3 or 4 channels, with
        //    each channel being one-byte.
        //
        // Above two of the only error paths in the implementation of `qoi::Encoder::new`.
        // It is unlikely the process of **encoding** would result in any other errors than these
        // two.
        let encoder = qoi::Encoder::new(self.as_bytes(), self.width(), self.height()).unwrap();

        // Use of unwrap:
        // Since we're using Vec as the output stream, the only possible error is allocator error,
        // dealing with it is outside the scope of this project.
        encoder.encode_to_stream(out).unwrap();
    }

    pub fn encode_png(&self, out: &mut Vec<u8>)
    where
        PF: PngCompatiblePixelFormat,
    {
        use zune_png::{
            PngEncoder,
            zune_core::{bit_depth::BitDepth, colorspace::ColorSpace, options::EncoderOptions},
        };
        let colorspace = match PF::FORMAT_ENUM {
            PixelFormat::Rgb8U => ColorSpace::RGB,
            PixelFormat::Rgba8U => ColorSpace::RGBA,
            PixelFormat::Luma8U => ColorSpace::Luma,
            _ => unreachable!(),
        };
        let bit_depth = match PF::FORMAT_ENUM {
            PixelFormat::Rgb8U | PixelFormat::Rgba8U | PixelFormat::Luma8U => BitDepth::Eight,
            _ => unreachable!(),
        };
        let mut encoder = PngEncoder::new(
            self.as_bytes(),
            EncoderOptions::default()
                .set_width(self.width() as usize)
                .set_height(self.height() as usize)
                .set_colorspace(colorspace)
                .set_depth(bit_depth)
                .set_num_threads(0),
        );
        // Use of unwrap:
        // Since we're using Vec<u8> as the writer type, the only possible error is allocation
        // error, which is outside the scope of this project to deal with.
        encoder.encode(out).unwrap();
    }

    /// Print the image out into stdout via [Kitty Graphics Protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/).
    pub fn print_with_kitty_graphics(&self) -> io::Result<()>
    where
        PF: KittyGraphicsCompatiblePixelFormat,
    {
        let mut stdout = io::stdout().lock();
        let base64_string = BASE64_STANDARD.encode(self.as_bytes());
        let n_chunks = base64_string.len().div_ceil(4096);
        let base64_chunks = base64_string.as_bytes().chunks(4096);
        let f = match PF::FORMAT_ENUM {
            PixelFormat::Rgb8U => 24,
            PixelFormat::Rgba8U => 32,
            _ => unreachable!(),
        };
        for (i, chunk) in base64_chunks.enumerate() {
            match i {
                // First chunk.
                0 => {
                    let width = self.width();
                    let height = self.height();
                    let m = match n_chunks == 1 {
                        true => 0,
                        false => 1,
                    };
                    write!(&mut stdout, "\x1b_Ga=T,f={f},s={width},v={height},m={m};")?;
                }
                // Non-first chunks.
                i if i + 1 == n_chunks => write!(&mut stdout, "\x1b_Gm=0;")?,
                // Last chunk.
                _ => write!(&mut stdout, "\x1b_Gm=1;")?,
            }
            stdout.write_all(chunk)?;
            write!(&mut stdout, "\x1b\\")?;
        }
        Ok(())
    }

    pub fn convert_format_lossy<NewPF: PixelFormatTrait>(&self) -> ImageBuffer<NewPF> {
        let mut result = ImageBuffer::<NewPF>::new_zeroed(self.width(), self.height());
        unsafe {
            format_convert::convert(
                PF::FORMAT_ENUM,
                self.as_bytes(),
                NewPF::FORMAT_ENUM,
                result.as_bytes_mut(),
            );
        };
        result
    }
}
