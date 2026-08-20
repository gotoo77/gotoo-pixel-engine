use std::{fmt, io::Cursor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    width: u32,
    height: u32,
    rgba8: Vec<u8>,
}

impl Image {
    pub fn from_rgba8(width: u32, height: u32, rgba8: Vec<u8>) -> Result<Self, ImageError> {
        let expected_len = pixel_len(width, height)?;
        if rgba8.len() != expected_len {
            return Err(ImageError::new(format!(
                "RGBA buffer length mismatch: expected {expected_len} bytes, got {}",
                rgba8.len()
            )));
        }

        Ok(Self {
            width,
            height,
            rgba8,
        })
    }

    pub fn decode_png(bytes: &[u8]) -> Result<Self, ImageError> {
        let mut decoder = png::Decoder::new(Cursor::new(bytes));
        decoder.set_transformations(png::Transformations::normalize_to_color8());
        let mut reader = decoder
            .read_info()
            .map_err(|error| ImageError::new(format!("PNG decode failed: {error}")))?;
        let output_len = reader
            .output_buffer_size()
            .ok_or_else(|| ImageError::new("PNG output buffer would exceed decoder limits"))?;
        let mut output = vec![0; output_len];
        let info = reader
            .next_frame(&mut output)
            .map_err(|error| ImageError::new(format!("PNG decode failed: {error}")))?;
        let bytes = &output[..info.buffer_size()];
        let rgba8 = normalize_png_frame_to_rgba8(bytes, &info)?;

        Self::from_rgba8(info.width, info.height, rgba8)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn as_rgba8(&self) -> &[u8] {
        &self.rgba8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl ImageRegion {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageError {
    message: String,
}

impl ImageError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ImageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for ImageError {}

fn normalize_png_frame_to_rgba8(
    bytes: &[u8],
    info: &png::OutputInfo,
) -> Result<Vec<u8>, ImageError> {
    if info.bit_depth != png::BitDepth::Eight {
        return Err(ImageError::new(format!(
            "unsupported PNG bit depth after normalization: {:?}",
            info.bit_depth
        )));
    }

    let pixel_count = info.width as usize * info.height as usize;
    let mut rgba8 = Vec::with_capacity(pixel_count * 4);

    match info.color_type {
        png::ColorType::Rgba => rgba8.extend_from_slice(bytes),
        png::ColorType::Rgb => {
            for rgb in bytes.chunks_exact(3) {
                rgba8.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
        png::ColorType::Grayscale => {
            for &value in bytes {
                rgba8.extend_from_slice(&[value, value, value, 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for gray_alpha in bytes.chunks_exact(2) {
                rgba8.extend_from_slice(&[
                    gray_alpha[0],
                    gray_alpha[0],
                    gray_alpha[0],
                    gray_alpha[1],
                ]);
            }
        }
        png::ColorType::Indexed => {
            return Err(ImageError::new(
                "indexed PNG did not expand to RGB/RGBA during decode",
            ));
        }
    }

    let expected_len = pixel_len(info.width, info.height)?;
    if rgba8.len() != expected_len {
        return Err(ImageError::new(format!(
            "decoded PNG size mismatch: expected {expected_len} RGBA bytes, got {}",
            rgba8.len()
        )));
    }

    Ok(rgba8)
}

fn pixel_len(width: u32, height: u32) -> Result<usize, ImageError> {
    let pixels = width
        .checked_mul(height)
        .ok_or_else(|| ImageError::new("image dimensions overflow"))?;
    pixels
        .checked_mul(4)
        .map(|len| len as usize)
        .ok_or_else(|| ImageError::new("image byte length overflow"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_rgba_buffers_with_wrong_length() {
        let error = Image::from_rgba8(2, 2, vec![0; 15]).unwrap_err();

        assert!(error.to_string().contains("length mismatch"));
    }

    #[test]
    fn decodes_rgba_png_bytes() {
        let bytes = tiny_rgba_png();
        let image = Image::decode_png(&bytes).expect("valid PNG should decode");

        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 1);
        assert_eq!(image.as_rgba8(), &[255, 0, 0, 255, 0, 128, 255, 128]);
    }

    #[test]
    fn invalid_png_reports_decode_error() {
        let error = Image::decode_png(b"not a png").unwrap_err();

        assert!(error.to_string().contains("PNG decode failed"));
    }

    fn tiny_rgba_png() -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, 2, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().expect("header");
            writer
                .write_image_data(&[255, 0, 0, 255, 0, 128, 255, 128])
                .expect("image data");
        }
        bytes
    }
}
