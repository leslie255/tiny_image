use std::{array, hint::assert_unchecked, slice};

use crate::*;

unsafe fn transmute_slice<T, U>(xs: &[T]) -> &[U] {
    const {
        assert!(
            size_of::<T>().is_multiple_of(size_of::<U>())
                || size_of::<U>().is_multiple_of(size_of::<T>())
        )
    };

    if size_of::<T>() >= size_of::<U>() {
        let size_ratio = const { size_of::<T>() / size_of::<U>() };
        unsafe { slice::from_raw_parts(xs.as_ptr() as *const U, xs.len() * size_ratio) }
    } else {
        let size_ratio = const { size_of::<U>() / size_of::<T>() };
        unsafe { slice::from_raw_parts(xs.as_ptr() as *const U, xs.len() / size_ratio) }
    }
}

unsafe fn transmute_slice_mut<T, U>(xs: &mut [T]) -> &mut [U] {
    const {
        assert!(
            size_of::<T>().is_multiple_of(size_of::<U>())
                || size_of::<U>().is_multiple_of(size_of::<T>())
        )
    };

    if size_of::<T>() >= size_of::<U>() {
        let size_ratio = const { size_of::<T>() / size_of::<U>() };
        unsafe { slice::from_raw_parts_mut(xs.as_ptr() as *mut U, xs.len() * size_ratio) }
    } else {
        let size_ratio = const { size_of::<U>() / size_of::<T>() };
        unsafe { slice::from_raw_parts_mut(xs.as_ptr() as *mut U, xs.len() / size_ratio) }
    }
}

unsafe fn helper<T: Copy, U: Copy>(
    src_bytes: &[u8],
    dst_bytes: &mut [u8],
    f_convert: impl Fn(T) -> U,
) {
    let src: &[T] = unsafe { transmute_slice(src_bytes) };
    let dst: &mut [U] = unsafe { transmute_slice_mut(dst_bytes) };

    if cfg!(debug_assertions) {
        debug_assert!(src.len() == dst.len());
    } else {
        unsafe { assert_unchecked(dst.len() == src.len()) };
    }

    for (i, &x) in src.iter().enumerate() {
        dst[i] = f_convert(x);
    }
}

#[rustfmt::skip]
pub(crate) unsafe fn convert(src_format: PixelFormat, src_bytes: &[u8], dst_format: PixelFormat, dst_bytes: &mut [u8]) {
    match (src_format, dst_format) {
        (PixelFormat::Rgb8U, PixelFormat::Rgb8U) => dst_bytes.copy_from_slice(src_bytes),
        (PixelFormat::Rgb8U, PixelFormat::Rgba8U) => unsafe { rgb8u_to_rgba8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgb8U, PixelFormat::Luma8U) => unsafe { rgb8u_to_luma8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgb8U, PixelFormat::Rgb32F) => unsafe { rgb8u_to_rgb32f(src_bytes, dst_bytes) },
        (PixelFormat::Rgb8U, PixelFormat::Rgba32F) => unsafe { rgb8u_to_rgba32f(src_bytes, dst_bytes) },
        (PixelFormat::Rgb8U, PixelFormat::Luma32F) => unsafe { rgb8u_to_luma32f(src_bytes, dst_bytes) },

        (PixelFormat::Rgba8U, PixelFormat::Rgb8U) => unsafe { rgba8u_to_rgb8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgba8U, PixelFormat::Rgba8U) => dst_bytes.copy_from_slice(src_bytes),
        (PixelFormat::Rgba8U, PixelFormat::Luma8U) => unsafe { rgba8u_to_luma8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgba8U, PixelFormat::Rgb32F) => unsafe { rgba8u_to_rgb32f(src_bytes, dst_bytes) },
        (PixelFormat::Rgba8U, PixelFormat::Rgba32F) => unsafe { rgba8u_to_rgba32f(src_bytes, dst_bytes) },
        (PixelFormat::Rgba8U, PixelFormat::Luma32F) => unsafe { rgba8u_to_luma32f(src_bytes, dst_bytes) },

        (PixelFormat::Luma8U, PixelFormat::Rgb8U) => unsafe { luma8u_to_rgb8u(src_bytes, dst_bytes) },
        (PixelFormat::Luma8U, PixelFormat::Rgba8U) => unsafe { luma8u_to_rgba8u(src_bytes, dst_bytes) },
        (PixelFormat::Luma8U, PixelFormat::Luma8U) => dst_bytes.copy_from_slice(src_bytes),
        (PixelFormat::Luma8U, PixelFormat::Rgb32F) => unsafe { luma8u_to_rgb32f(src_bytes, dst_bytes) },
        (PixelFormat::Luma8U, PixelFormat::Rgba32F) => unsafe { luma8u_to_rgba32f(src_bytes, dst_bytes) },
        (PixelFormat::Luma8U, PixelFormat::Luma32F) => unsafe { luma8u_to_luma32f(src_bytes, dst_bytes) },

        (PixelFormat::Rgb32F, PixelFormat::Rgb8U) => unsafe { rgb32f_to_rgb8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgb32F, PixelFormat::Rgba8U) => unsafe { rgb32f_to_rgba8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgb32F, PixelFormat::Luma8U) => unsafe { rgb32f_to_luma8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgb32F, PixelFormat::Rgb32F) => dst_bytes.copy_from_slice(src_bytes),
        (PixelFormat::Rgb32F, PixelFormat::Rgba32F) => unsafe { rgb32f_to_rgba32f(src_bytes, dst_bytes) },
        (PixelFormat::Rgb32F, PixelFormat::Luma32F) => unsafe { rgb32f_to_luma32f(src_bytes, dst_bytes) },

        (PixelFormat::Rgba32F, PixelFormat::Rgb8U) => unsafe { rgba32f_to_rgb8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgba32F, PixelFormat::Rgba8U) => unsafe { rgba32f_to_rgba8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgba32F, PixelFormat::Luma8U) => unsafe { rgba32f_to_luma8u(src_bytes, dst_bytes) },
        (PixelFormat::Rgba32F, PixelFormat::Rgb32F) => unsafe { rgba32f_to_rgb32f(src_bytes, dst_bytes) },
        (PixelFormat::Rgba32F, PixelFormat::Rgba32F) => dst_bytes.copy_from_slice(src_bytes),
        (PixelFormat::Rgba32F, PixelFormat::Luma32F) => unsafe { rgba32f_to_luma32f(src_bytes, dst_bytes) },

        (PixelFormat::Luma32F, PixelFormat::Rgb8U) => unsafe { luma32f_to_rgb8u(src_bytes, dst_bytes) },
        (PixelFormat::Luma32F, PixelFormat::Rgba8U) => unsafe { luma32f_to_rgba8u(src_bytes, dst_bytes) },
        (PixelFormat::Luma32F, PixelFormat::Luma8U) => unsafe { luma32f_to_luma8u(src_bytes, dst_bytes) },
        (PixelFormat::Luma32F, PixelFormat::Rgb32F) => unsafe { luma32f_to_rgb32f(src_bytes, dst_bytes) },
        (PixelFormat::Luma32F, PixelFormat::Rgba32F) => unsafe { luma32f_to_rgba32f(src_bytes, dst_bytes) },
        (PixelFormat::Luma32F, PixelFormat::Luma32F) => dst_bytes.copy_from_slice(src_bytes),
    }
}

pub(crate) unsafe fn rgb8u_to_rgba8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 3], [u8; 4]>(src_bytes, dst_bytes, |[r, g, b]| [r, g, b, 255]);
    }
}

pub(crate) unsafe fn rgb8u_to_luma8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 3], u8>(src_bytes, dst_bytes, |[r, g, b]| {
            ((77 * r as u16 + 150 * g as u16 + 29 * b as u16) >> 8) as u8
        });
    }
}

pub(crate) unsafe fn rgb8u_to_rgb32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 3], [f32; 3]>(src_bytes, dst_bytes, |[r, g, b]| {
            [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
        });
    }
}

pub(crate) unsafe fn rgb8u_to_rgba32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 3], [f32; 4]>(src_bytes, dst_bytes, |[r, g, b]| {
            [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
        });
    }
}

pub(crate) unsafe fn rgb8u_to_luma32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 3], f32>(src_bytes, dst_bytes, |[r, g, b]| {
            (77.0 * r as f32 + 150.0 * g as f32 + 29.0 * b as f32) / 256.0
        });
    }
}

pub(crate) unsafe fn rgba8u_to_rgb8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 4], [u8; 3]>(src_bytes, dst_bytes, |[r, g, b, _]| [r, g, b]);
    }
}

pub(crate) unsafe fn rgba8u_to_luma8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 4], u8>(src_bytes, dst_bytes, |[r, g, b, _]| {
            ((77 * r as u16 + 150 * g as u16 + 29 * b as u16) >> 8) as u8
        });
    }
}

pub(crate) unsafe fn rgba8u_to_rgb32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 4], [f32; 3]>(src_bytes, dst_bytes, |[r, g, b, _]| {
            [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
        });
    }
}

pub(crate) unsafe fn rgba8u_to_rgba32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 4], [f32; 4]>(src_bytes, dst_bytes, |[r, g, b, a]| {
            [
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            ]
        });
    }
}

pub(crate) unsafe fn rgba8u_to_luma32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[u8; 4], f32>(src_bytes, dst_bytes, |[r, g, b, _]| {
            (77.0 * r as f32 + 150.0 * g as f32 + 29.0 * b as f32) / 256.0
        });
    }
}

pub(crate) unsafe fn luma8u_to_rgb8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<u8, [u8; 3]>(src_bytes, dst_bytes, array::repeat::<u8, 3>);
    }
}

pub(crate) unsafe fn luma8u_to_rgba8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<u8, [u8; 4]>(src_bytes, dst_bytes, |v| [v, v, v, 255]);
    }
}

pub(crate) unsafe fn luma8u_to_rgb32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<u8, [f32; 3]>(src_bytes, dst_bytes, |v| {
            let f = v as f32 / 255.0;
            [f, f, f]
        });
    }
}

pub(crate) unsafe fn luma8u_to_rgba32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<u8, [f32; 4]>(src_bytes, dst_bytes, |v| {
            let f = v as f32 / 255.0;
            [f, f, f, 1.0]
        });
    }
}

pub(crate) unsafe fn luma8u_to_luma32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<u8, f32>(src_bytes, dst_bytes, |v| v as f32 / 255.0);
    }
}

pub(crate) unsafe fn rgb32f_to_rgb8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 3], [u8; 3]>(src_bytes, dst_bytes, |[r, g, b]| {
            [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
        });
    }
}

pub(crate) unsafe fn rgb32f_to_rgba8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 3], [u8; 4]>(src_bytes, dst_bytes, |[r, g, b]| {
            [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]
        });
    }
}

pub(crate) unsafe fn rgb32f_to_luma8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 3], u8>(src_bytes, dst_bytes, |[r, g, b]| {
            let luma = 0.299 * r + 0.587 * g + 0.114 * b;
            (luma.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
        });
    }
}

pub(crate) unsafe fn rgb32f_to_rgba32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 3], [f32; 4]>(src_bytes, dst_bytes, |[r, g, b]| [r, g, b, 1.0]);
    }
}

pub(crate) unsafe fn rgb32f_to_luma32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 3], f32>(src_bytes, dst_bytes, |[r, g, b]| {
            (77.0 * r + 150.0 * g + 29.0 * b) / 256.0
        });
    }
}

pub(crate) unsafe fn rgba32f_to_rgb8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 4], [u8; 3]>(src_bytes, dst_bytes, |[r, g, b, _]| {
            [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
        });
    }
}

pub(crate) unsafe fn rgba32f_to_rgba8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 4], [u8; 4]>(src_bytes, dst_bytes, |[r, g, b, a]| {
            [
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                (a * 255.0) as u8,
            ]
        });
    }
}

pub(crate) unsafe fn rgba32f_to_luma8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 4], u8>(src_bytes, dst_bytes, |[r, g, b, _a]| {
            let luma = 0.299 * r + 0.587 * g + 0.114 * b;
            (luma.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
        });
    }
}

pub(crate) unsafe fn rgba32f_to_rgb32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 4], [f32; 3]>(src_bytes, dst_bytes, |[r, g, b, _]| [r, g, b]);
    }
}

pub(crate) unsafe fn rgba32f_to_luma32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<[f32; 4], f32>(src_bytes, dst_bytes, |[r, g, b, _]| {
            (77.0 * r + 150.0 * g + 29.0 * b) / 256.0
        });
    }
}

pub(crate) unsafe fn luma32f_to_rgb8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<f32, [u8; 3]>(src_bytes, dst_bytes, |v| {
            let u = (v * 255.0) as u8;
            [u, u, u]
        });
    }
}

pub(crate) unsafe fn luma32f_to_rgba8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<f32, [u8; 4]>(src_bytes, dst_bytes, |v| {
            let u = (v * 255.0) as u8;
            [u, u, u, 255]
        });
    }
}

pub(crate) unsafe fn luma32f_to_luma8u(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<f32, u8>(src_bytes, dst_bytes, |v| (v * 255.0) as u8);
    }
}

pub(crate) unsafe fn luma32f_to_rgb32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<f32, [f32; 3]>(src_bytes, dst_bytes, array::repeat::<f32, 3>);
    }
}

pub(crate) unsafe fn luma32f_to_rgba32f(src_bytes: &[u8], dst_bytes: &mut [u8]) {
    unsafe {
        helper::<f32, [f32; 4]>(src_bytes, dst_bytes, |v| [v, v, v, 1.0]);
    }
}
