//! A small library for working with images.
//!
//! The library provides types `ImageBuffer`, `ImageRef`, `ImageMut`, `ImagePtr` for representing
//! image buffers.
//!
//! Conceptually, if we pretend there is a dynamically-sized type `Image`:
//!
//! - `ImageBuffer` is `Box<Image>`
//! - `ImageRef<'a>` is `&'a Image`
//! - `ImageMut<'a>` is `&'a mut Image`
//! - `ImagePtr` is `NonNull<Image>`
//!
//! Such DST unfortunately cannot exist in current versions of Rust, because entirely custom
//! pointer metadata is still a work-in-progress feature. (One may be able to hack together such
//! DST type with `#[feature(ptr_metadata)]`, but things like `std::size_of_val` would break
//! completely for such type).
//!
//! This limitation implicates that some implicit coersions inserted by the compiler for the normal
//! `&T` and `&mut T` types has to now be performed manually. Namely, re-borrowing needs to be
//! manually specified with `.reborrow()` and `.reborrow_mut()`.
//!
//! All image types have statically ensured non-zero width and height. Although the width and
//! height values are internally represented with `u32`, as oppose to `NonZeroU32`. The pointer to
//! the data buffer, however, is `NonNull`.
//!
//! `ImageBuffer`, `ImageRef`, `ImageMut` are `#[repr(transparent)]` wrappers of `ImagePtr`. So
//! they can be `transmute`'d to/from each other for those who desire to do so.

mod any_image_buffer;
mod image_buffer;
mod image_mut;
mod image_ptr;
mod image_ref;
mod pixel_format;

pub use any_image_buffer::*;
pub use image_buffer::*;
pub use image_mut::*;
pub use image_ptr::*;
pub use image_ref::*;
pub use pixel_format::*;

pub(crate) mod format_convert;
