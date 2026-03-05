use std::{
    alloc::{self, Layout},
    fmt::Debug,
    io,
    mem::forget,
    mem::{transmute, transmute_copy},
    ptr::copy_nonoverlapping,
    slice,
};

use crate::*;

#[repr(transparent)]
pub struct ImageBuffer<PF: PixelFormatTrait> {
    raw: ImagePtr<PF>,
}

impl<PF: PixelFormatTrait> Drop for ImageBuffer<PF> {
    fn drop(&mut self) {
        unsafe { Self::dealloc(self.raw) };
    }
}

impl<PF: PixelFormatTrait> Clone for ImageBuffer<PF> {
    fn clone(&self) -> Self {
        Self::from_image_ref(self.as_image_ref())
    }

    fn clone_from(&mut self, source: &Self) {
        if self.width() == source.width() && self.height() == source.height() {
            unsafe {
                copy_nonoverlapping::<PF::Primitive>(
                    source.raw.as_primitives_ptr(), // src
                    self.raw.as_primitives_ptr(),   // dst
                    self.raw.primitives_count(),    // count
                );
            }
        } else {
            *self = Self::clone(source);
        }
    }
}

impl<PF: PixelFormatTrait> Debug for ImageBuffer<PF> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageBuffer")
            .field("width", &self.width())
            .field("height", &self.height())
            .field("data", &self.raw.as_primitives_ptr())
            .finish()
    }
}

impl<PF: PixelFormatTrait> ImageBuffer<PF> {
    /// # Safety
    ///
    /// - `width` and `height` must be both non-zero.
    /// - `buffer` must be owned logically
    /// - `buffer` must be a value returned by the global allocator
    /// - `buffer` must not have any extra capacity at its tail
    pub unsafe fn from_raw_parts(width: u32, height: u32, buffer: *mut PF::Primitive) -> Self {
        debug_assert!(width != 0);
        debug_assert!(height != 0);
        Self {
            raw: unsafe { ImagePtr::from_raw_parts(width, height, buffer) },
        }
    }

    /// # Safety
    ///
    /// - `width` and `height` must be both non-zero.
    /// - buffer must be owned logically
    /// - buffer must be a pointer returned by the global allocator
    /// - buffer must not have any extra capacity at its tail
    pub unsafe fn from_image_ptr(image_ptr: ImagePtr<PF>) -> Self {
        Self { raw: image_ptr }
    }

    /// # Panics
    ///
    /// - if either `width` and `height` are zero
    pub fn new_zeroed(width: u32, height: u32) -> Self {
        assert!(width != 0);
        assert!(height != 0);
        Self {
            raw: Self::alloc(width, height, true),
        }
    }

    /// # Panics
    ///
    /// - if either `width` and `height` are zero
    pub fn from_fn(
        width: u32,
        height: u32,
        mut f_pixel: impl FnMut(u32, u32) -> PF::PrimitiveArray,
    ) -> Self {
        assert!(width != 0);
        assert!(height != 0);
        let raw = Self::alloc(width, height, false);
        for x in 0..width {
            for y in 0..height {
                unsafe { *raw.get_pixel_unchecked(x, y) = f_pixel(x, y) };
            }
        }
        Self { raw }
    }

    pub fn from_image_ref(image: ImageRef<PF>) -> Self {
        let raw = Self::alloc(image.width(), image.height(), false);
        unsafe {
            copy_nonoverlapping::<PF::Primitive>(
                image.as_image_ptr().as_primitives_ptr(), // src
                raw.as_primitives_ptr(),                  // dst
                raw.primitives_count(),                   // count
            );
        }
        Self { raw }
    }

    pub fn as_image_ref(&self) -> ImageRef<'_, PF> {
        unsafe { transmute_copy(self) }
    }

    pub fn as_image_mut(&mut self) -> ImageMut<'_, PF> {
        unsafe { transmute_copy(self) }
    }

    pub fn as_image_ptr(&self) -> ImagePtr<PF> {
        self.raw
    }

    /// # Safety
    ///
    /// - `x` and `y` must be in range (`0..width` and `0..height`, respectively)
    pub unsafe fn get_pixel_unchecked(&self, x: u32, y: u32) -> PF::PrimitiveArray {
        unsafe { *self.raw.get_pixel_unchecked(x, y) }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Option<PF::PrimitiveArray> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.get_pixel_unchecked(x, y) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// - `y` must be in range
    pub unsafe fn get_row_unchecked(&self, y: u32) -> &[PF::PrimitiveArray] {
        unsafe { slice::from_raw_parts(self.raw.get_row_unchecked(y), self.width() as usize) }
    }

    pub fn get_row(&self, y: u32) -> Option<&[PF::PrimitiveArray]> {
        if y < self.height() {
            Some(unsafe { self.get_row_unchecked(y) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// - `x` and `y` must be in range (`0..width` and `0..height`, respectively)
    pub unsafe fn get_pixel_unchecked_mut(&mut self, x: u32, y: u32) -> &mut PF::PrimitiveArray {
        unsafe { &mut *self.raw.get_pixel_unchecked(x, y) }
    }

    pub fn get_pixel_mut(&mut self, x: u32, y: u32) -> Option<&mut PF::PrimitiveArray> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.get_pixel_unchecked_mut(x, y) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// - `y` must be in range
    pub unsafe fn get_row_unchecked_mut(&mut self, y: u32) -> &mut [PF::PrimitiveArray] {
        unsafe { slice::from_raw_parts_mut(self.raw.get_row_unchecked(y), self.width() as usize) }
    }

    pub fn get_row_mut(&mut self, y: u32) -> Option<&mut [PF::PrimitiveArray]> {
        if y < self.height() {
            Some(unsafe { self.get_row_unchecked_mut(y) })
        } else {
            None
        }
    }

    pub fn width(&self) -> u32 {
        self.raw.width()
    }

    pub fn height(&self) -> u32 {
        self.raw.height()
    }

    pub fn as_bytes(&self) -> &'_ [u8] {
        unsafe { slice::from_raw_parts(self.raw.as_bytes_ptr(), self.raw.bytes_count()) }
    }

    pub fn as_primitives(&self) -> &'_ [PF::Primitive] {
        unsafe { slice::from_raw_parts(self.raw.as_primitives_ptr(), self.raw.primitives_count()) }
    }

    pub fn as_pixels(&self) -> &'_ [PF::PrimitiveArray] {
        unsafe { slice::from_raw_parts(self.raw.as_pixels_ptr(), self.raw.pixels_count()) }
    }

    pub fn as_bytes_mut(&mut self) -> &'_ mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.raw.as_bytes_ptr(), self.raw.bytes_count()) }
    }

    pub fn as_primitives_mut(&mut self) -> &'_ mut [PF::Primitive] {
        unsafe {
            slice::from_raw_parts_mut(self.raw.as_primitives_ptr(), self.raw.primitives_count())
        }
    }

    pub fn as_pixels_mut(&mut self) -> &'_ mut [PF::PrimitiveArray] {
        unsafe { slice::from_raw_parts_mut(self.raw.as_pixels_ptr(), self.raw.pixels_count()) }
    }

    pub fn into_bytes_vec(self) -> Vec<u8> {
        let vec = unsafe {
            Vec::from_raw_parts(
                self.raw.as_bytes_ptr(),
                self.raw.bytes_count(),
                self.raw.bytes_count(),
            )
        };
        forget(self);
        vec
    }

    pub fn into_primitives_vec(self) -> Vec<PF::Primitive> {
        let vec = unsafe {
            Vec::from_raw_parts(
                self.raw.as_primitives_ptr(),
                self.raw.primitives_count(),
                self.raw.primitives_count(),
            )
        };
        forget(self);
        vec
    }

    pub fn into_pixels_vec(self) -> Vec<PF::PrimitiveArray> {
        let vec = unsafe {
            Vec::from_raw_parts(
                self.raw.as_pixels_ptr(),
                self.raw.pixels_count(),
                self.raw.pixels_count(),
            )
        };
        forget(self);
        vec
    }

    fn alloc(width: u32, height: u32, zeroed: bool) -> ImagePtr<PF> {
        let n_pixels = width as usize * height as usize;
        let layout = Layout::array::<PF::Primitive>(n_pixels * PF::DEPTH).unwrap();
        let buffer = match zeroed {
            true => unsafe { alloc::alloc_zeroed(layout) },
            false => unsafe { alloc::alloc(layout) },
        };
        assert!(
            !buffer.is_null(),
            "allocation failed in ImagePtr<{:?}>::alloc",
            PF::FORMAT_ENUM
        );
        unsafe { ImagePtr::from_raw_parts(width, height, buffer as *mut _) }
    }

    /// # Safety
    ///
    /// - `raw` cannot have any spare capacity at the end
    unsafe fn dealloc(raw: ImagePtr<PF>) {
        let layout = Layout::array::<PF::Primitive>(raw.pixels_count() * PF::DEPTH).unwrap();
        unsafe {
            alloc::dealloc(raw.as_bytes_ptr(), layout);
        };
    }

    pub fn encode_qoi(&self, out: &mut Vec<u8>)
    where
        PF: QoiCompatiblePixelFormat,
    {
        self.as_image_ref().encode_qoi(out);
    }

    pub fn encode_png(&self, out: &mut Vec<u8>)
    where
        PF: PngCompatiblePixelFormat,
    {
        self.as_image_ref().encode_png(out);
    }

    pub fn print_with_kitty_graphics(&self) -> io::Result<()>
    where
        PF: KittyGraphicsCompatiblePixelFormat,
    {
        self.as_image_ref().print_with_kitty_graphics()
    }

    pub fn convert_format_lossy<NewPF: PixelFormatTrait>(&self) -> ImageBuffer<NewPF> {
        self.as_image_ref().convert_format_lossy::<NewPF>()
    }

    /// Convert to the specified pixel format. If not already that format.
    pub fn into_format_lossy<NewPF: PixelFormatTrait>(self) -> ImageBuffer<NewPF> {
        if PF::FORMAT_ENUM == NewPF::FORMAT_ENUM {
            unsafe { transmute::<ImageBuffer<PF>, ImageBuffer<NewPF>>(self) }
        } else {
            self.convert_format_lossy()
        }
    }
}
