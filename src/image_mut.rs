use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    mem::{transmute, transmute_copy},
    ops::Deref,
    ptr::write_bytes,
    slice,
};

use crate::*;

#[repr(transparent)]
pub struct ImageMut<'a, PF: PixelFormatTrait> {
    raw: ImagePtr<PF>,
    _marker: PhantomData<&'a mut [PF::Primitive]>,
}

impl<'a, PF: PixelFormatTrait> Debug for ImageMut<'a, PF> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ImageMut")
            .field("width", &self.width())
            .field("height", &self.height())
            .field("data", &self.raw.as_primitives_ptr())
            .finish()
    }
}

impl<'a, PF: PixelFormatTrait> Deref for ImageMut<'a, PF> {
    type Target = ImageRef<'a, PF>;
    fn deref(&self) -> &Self::Target {
        unsafe { transmute::<&ImageMut<PF>, &ImageRef<PF>>(self) }
    }
}

impl<'a, PF: PixelFormatTrait> AsRef<ImageRef<'a, PF>> for ImageMut<'a, PF> {
    fn as_ref(&self) -> &ImageRef<'a, PF> {
        self
    }
}

impl<'a, PF: PixelFormatTrait> ImageMut<'a, PF> {
    /// # Safety
    ///
    /// - `width` and `height` must both be non-zero
    /// - `data` must satisfy the safety contract for `std::slice::from_raw_parts_mut<'a, _>(data, len)`,
    ///   where `len` is width * height * PF::DEPTH.
    pub const unsafe fn from_raw_parts(width: u32, height: u32, data: *mut PF::Primitive) -> Self {
        Self {
            raw: unsafe { ImagePtr::from_raw_parts(width, height, data) },
            _marker: PhantomData,
        }
    }

    /// Returns `None` if:
    /// - `data` is not large enough to accommodate width * height * PF::DEPTH
    /// - `width` or `height` is zero
    pub const fn new(width: u32, height: u32, data: &'a mut [PF::Primitive]) -> Option<Self> {
        let n_bytes = width as usize * height as usize * PF::DEPTH;
        if data.len() >= n_bytes && n_bytes != 0 {
            Some(unsafe { Self::from_raw_parts(width, height, data.as_mut_ptr()) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// - `x` and `y` must be in range (`0..width` and `0..height`, respectively)
    pub const unsafe fn get_pixel_unchecked_mut(
        &mut self,
        x: u32,
        y: u32,
    ) -> &mut PF::PrimitiveArray {
        unsafe { &mut *self.raw.get_pixel_unchecked(x, y) }
    }

    pub const fn get_pixel_mut(&mut self, x: u32, y: u32) -> Option<&mut PF::PrimitiveArray> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.get_pixel_unchecked_mut(x, y) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// - `y` must be in range
    pub const unsafe fn get_row_unchecked_mut(&mut self, y: u32) -> &mut [PF::PrimitiveArray] {
        unsafe { slice::from_raw_parts_mut(self.raw.get_row_unchecked(y), self.width() as usize) }
    }

    pub const fn get_row_mut(&mut self, y: u32) -> Option<&mut [PF::PrimitiveArray]> {
        if y < self.height() {
            Some(unsafe { self.get_row_unchecked_mut(y) })
        } else {
            None
        }
    }

    pub const fn as_image_ref(&self) -> ImageRef<'a, PF> {
        unsafe { transmute_copy::<ImageMut<PF>, ImageRef<PF>>(self) }
    }

    pub const fn reborrow<'b>(&'b self) -> ImageRef<'b, PF> {
        self.as_image_ref()
    }

    pub const fn reborrow_mut(&mut self) -> ImageMut<'_, PF> {
        unsafe { transmute_copy(self) }
    }

    pub const fn width(&self) -> u32 {
        self.raw.width()
    }

    pub const fn height(&self) -> u32 {
        self.raw.height()
    }

    pub const fn as_bytes_mut(&mut self) -> &'a mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.raw.as_bytes_ptr(), self.raw.bytes_count()) }
    }

    pub const fn as_primitives_mut(&mut self) -> &'a mut [PF::Primitive] {
        unsafe {
            slice::from_raw_parts_mut(self.raw.as_primitives_ptr(), self.raw.primitives_count())
        }
    }

    pub const fn as_pixels_mut(&mut self) -> &'a mut [PF::PrimitiveArray] {
        unsafe { slice::from_raw_parts_mut(self.raw.as_pixels_ptr(), self.raw.pixels_count()) }
    }

    pub const fn fill_zeros(&mut self) {
        unsafe {
            let primitives = self.as_primitives_mut();
            write_bytes(primitives.as_mut_ptr(), 0u8, primitives.len());
        }
    }
}
