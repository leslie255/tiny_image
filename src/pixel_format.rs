/// Convenience module for batch-importing all pixel format marker types:
///
/// ```
/// use tiny_image::pixel_formats::*;
/// ```
pub mod pixel_formats {
    pub use super::Rgb8U;
    pub use super::Rgba8U;
    pub use super::Luma8U;
    pub use super::Rgb32F;
    pub use super::Rgba32F;
    pub use super::Luma32F;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgb8U,
    Rgba8U,
    Luma8U,
    Rgb32F,
    Rgba32F,
    Luma32F,
}

impl PixelFormat {
    pub const fn is_qoi_encodeable(self) -> bool {
        match self {
            Self::Rgb8U | Self::Rgba8U => true,
            Self::Luma8U | Self::Rgb32F | Self::Rgba32F | Self::Luma32F => false,
        }
    }

    pub const fn is_png_encodeable(self) -> bool {
        match self {
            Self::Rgb8U | Self::Rgba8U | Self::Luma8U => true,
            Self::Rgb32F | Self::Rgba32F | Self::Luma32F => false,
        }
    }

    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            PixelFormat::Rgb8U => 3,
            PixelFormat::Rgba8U => 4,
            PixelFormat::Luma8U => 1,
            PixelFormat::Rgb32F => 12,
            PixelFormat::Rgba32F => 16,
            PixelFormat::Luma32F => 4,
        }
    }

    pub const fn primitives_per_pixel(self) -> usize {
        match self {
            PixelFormat::Rgb8U => 3,
            PixelFormat::Rgba8U => 4,
            PixelFormat::Luma8U => 1,
            PixelFormat::Rgb32F => 3,
            PixelFormat::Rgba32F => 4,
            PixelFormat::Luma32F => 1,
        }
    }
}

/// # Safety
///
/// - `Primitive` must be a POD type
/// - `PrimitiveArray` must be `[Repr, DEPTH]`
/// - `FORMAT_ENUM` must be correct
pub unsafe trait PixelFormatTrait: private::Sealed + Copy {
    type Primitive: Sized + Copy + 'static;

    /// Size of one pixel, in terms of the number of `Self::Repr`.
    const DEPTH: usize;

    type PrimitiveArray: Sized + Copy + AsRef<[Self::Primitive]> + 'static;

    const FORMAT_ENUM: PixelFormat;
}

#[diagnostic::on_unimplemented(
    message = "pixel format {Self} is not directly encodeable/decodeable from QOI",
    note = "QOI-compatible pixel formats are: Rgb8U, Rgba8U",
    label = "consider converting the image one of the supported pixel format before encoding/decoding"
)]
pub trait QoiCompatiblePixelFormat: PixelFormatTrait {}

#[diagnostic::on_unimplemented(
    message = "pixel format {Self} is not directly printable via Kitty Graphics Protocol",
    note = "Kitty graphics-compatible pixel formats are: Rgb8U, Rgba8U, Luma8U",
    label = "consider converting the image one of the supported pixel format before encoding/decoding"
)]
pub trait PngCompatiblePixelFormat: PixelFormatTrait {}

#[diagnostic::on_unimplemented(
    message = "pixel format {Self} is not directly printable via kitty graphics protocol",
    note = "kitty graphics-compatible pixel formats are: Rgb8U, Rgba8U",
    label = "consider converting the image to one of the supported pixel formats before printing"
)]
pub trait KittyGraphicsCompatiblePixelFormat: PixelFormatTrait {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb8U;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba8U;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Luma8U;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb32F;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba32F;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Luma32F;

mod private {
    #[diagnostic::on_unimplemented(
        message = "This trait is sealed and cannot be implemented by crates outside of `tiny_image`"
    )]
    pub trait Sealed {}
}

impl private::Sealed for Rgb8U {}
impl private::Sealed for Rgba8U {}
impl private::Sealed for Luma8U {}
impl private::Sealed for Rgb32F {}
impl private::Sealed for Rgba32F {}
impl private::Sealed for Luma32F {}

unsafe impl PixelFormatTrait for Rgb8U {
    type Primitive = u8;
    const DEPTH: usize = 3;
    type PrimitiveArray = [u8; 3];
    const FORMAT_ENUM: PixelFormat = PixelFormat::Rgb8U;
}
unsafe impl PixelFormatTrait for Rgba8U {
    type Primitive = u8;
    const DEPTH: usize = 4;
    type PrimitiveArray = [u8; 4];
    const FORMAT_ENUM: PixelFormat = PixelFormat::Rgba8U;
}
unsafe impl PixelFormatTrait for Luma8U {
    type Primitive = u8;
    const DEPTH: usize = 1;
    type PrimitiveArray = [u8; 1];
    const FORMAT_ENUM: PixelFormat = PixelFormat::Luma8U;
}
unsafe impl PixelFormatTrait for Rgb32F {
    type Primitive = f32;
    const DEPTH: usize = 3;
    type PrimitiveArray = [f32; 3];
    const FORMAT_ENUM: PixelFormat = PixelFormat::Rgb32F;
}
unsafe impl PixelFormatTrait for Rgba32F {
    type Primitive = f32;
    const DEPTH: usize = 4;
    type PrimitiveArray = [f32; 4];
    const FORMAT_ENUM: PixelFormat = PixelFormat::Rgba32F;
}
unsafe impl PixelFormatTrait for Luma32F {
    type Primitive = f32;
    const DEPTH: usize = 1;
    type PrimitiveArray = [f32; 1];
    const FORMAT_ENUM: PixelFormat = PixelFormat::Luma32F;
}

impl QoiCompatiblePixelFormat for Rgb8U {}
impl QoiCompatiblePixelFormat for Rgba8U {}

impl PngCompatiblePixelFormat for Rgb8U {}
impl PngCompatiblePixelFormat for Rgba8U {}
impl PngCompatiblePixelFormat for Luma8U {}

impl KittyGraphicsCompatiblePixelFormat for Rgb8U {}
impl KittyGraphicsCompatiblePixelFormat for Rgba8U {}
