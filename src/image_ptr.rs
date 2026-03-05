use std::{mem::transmute, ptr::NonNull};

use crate::*;

/// Like `ImageRef` and `ImageMut`, but without lifetimes (obviously dereferencing/indexing is then
/// unsafe).
/// The non-zero-ness of width/height still hold true for `ImagePtr`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImagePtr<PF: PixelFormatTrait> {
    width: u32,
    height: u32,
    data: NonNull<PF::Primitive>,
}

unsafe impl<PF: PixelFormatTrait> Send for ImagePtr<PF> {}
unsafe impl<PF: PixelFormatTrait> Sync for ImagePtr<PF> {}

impl<PF: PixelFormatTrait> ImagePtr<PF> {
    /// # Safety
    ///
    /// - `width` and `height` must both be non-zero
    /// - `data` must be non-null
    pub const unsafe fn from_raw_parts(width: u32, height: u32, data: *mut PF::Primitive) -> Self {
        debug_assert!(width != 0);
        debug_assert!(height != 0);

        Self {
            width,
            height,
            data: unsafe { NonNull::new_unchecked(data as *mut _) },
        }
    }

    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn bytes_count(&self) -> usize {
        self.primitives_count() * size_of::<PF::Primitive>()
    }

    pub const fn as_bytes_ptr(&self) -> *mut u8 {
        self.data.as_ptr() as *mut u8
    }

    pub const fn primitives_count(&self) -> usize {
        self.pixels_count() * PF::DEPTH
    }

    pub const fn as_primitives_ptr(&self) -> *mut PF::Primitive {
        self.data.as_ptr()
    }

    pub const fn pixels_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    pub const fn as_pixels_ptr(&self) -> *mut PF::PrimitiveArray {
        self.data.as_ptr() as *mut PF::PrimitiveArray
    }

    /// # Safety
    ///
    /// - `data` must satisfy the safety contract of `std::slice::from_raw_parts<'a>(data, _)`
    pub const unsafe fn as_image_ref<'a>(self) -> ImageRef<'a, PF> {
        unsafe { transmute::<ImagePtr<PF>, ImageRef<PF>>(self) }
    }

    /// # Safety
    ///
    /// - `data` must satisfy the safety contract of `std::slice::from_raw_parts_mut<'a>(data, _)`
    pub const unsafe fn as_image_mut<'a>(self) -> ImageMut<'a, PF> {
        unsafe { transmute::<ImagePtr<PF>, ImageMut<PF>>(self) }
    }

    /// # Safety
    ///
    /// - `x` and `y` must be in range (`0..width` and `0..height`, respectively)
    pub const unsafe fn get_pixel_unchecked(&self, x: u32, y: u32) -> *mut PF::PrimitiveArray {
        let index = y as usize * self.width() as usize + x as usize;
        unsafe { self.as_pixels_ptr().add(index) }
    }

    /// # Safety
    ///
    /// - `y` must be in range
    pub const unsafe fn get_row_unchecked(&self, y: u32) -> *mut PF::PrimitiveArray {
        let offset = y as usize * self.width() as usize;
        unsafe { self.as_pixels_ptr().add(offset) }
    }
}
