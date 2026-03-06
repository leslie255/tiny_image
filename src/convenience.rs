use std::{
    error::Error,
    fmt::{self, Display},
    fs::File,
    io::{self, Read},
    path::Path,
};

use crate::*;

#[derive(Debug)]
pub enum OpenError {
    IoError(io::Error),
    QoiDecodeError(QoiDecodeError),
    PngDecodeError(PngDecodeError),
    UnknownFileFormat,
}
impl From<io::Error> for OpenError {
    fn from(v: io::Error) -> Self {
        Self::IoError(v)
    }
}
impl From<QoiDecodeError> for OpenError {
    fn from(v: QoiDecodeError) -> Self {
        Self::QoiDecodeError(v)
    }
}
impl From<PngDecodeError> for OpenError {
    fn from(v: PngDecodeError) -> Self {
        Self::PngDecodeError(v)
    }
}
impl Display for OpenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IoError(error) => Display::fmt(error, f),
            Self::QoiDecodeError(error) => Display::fmt(error, f),
            Self::PngDecodeError(error) => Display::fmt(error, f),
            Self::UnknownFileFormat => write!(f, "unknown file format"),
        }
    }
}
impl Error for OpenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IoError(error) => Some(error),
            Self::QoiDecodeError(error) => Some(error),
            Self::PngDecodeError(error) => Some(error),
            _ => None,
        }
    }
}

pub fn open<P: AsRef<Path>>(path: P) -> Result<AnyImageBuffer, OpenError> {
    open_(path.as_ref())
}

fn open_(path: &Path) -> Result<AnyImageBuffer, OpenError> {
    const QOI_MAGIC_NUMBER: &[u8] = b"qoif";
    const PNG_MAGIC_NUMBER: &[u8] = b"\x89PNG\r\n\x1A\n";

    let mut file = File::open(path)?;

    let header = {
        let mut buffer = [0u8; 8];
        file.read_exact(&mut buffer[..])?;
        buffer
    };

    if header.starts_with(QOI_MAGIC_NUMBER) {
        let mut file_data = Vec::<u8>::from(&header[..]);
        file.read_to_end(&mut file_data)?;
        Ok(AnyImageBuffer::decode_from_qoi(&file_data[..])?)
    } else if header.starts_with(PNG_MAGIC_NUMBER) {
        let mut file_data = Vec::<u8>::from(&header[..]);
        file.read_to_end(&mut file_data)?;
        Ok(AnyImageBuffer::decode_from_png(&file_data[..])?)
    } else {
        Err(OpenError::UnknownFileFormat)
    }
}
