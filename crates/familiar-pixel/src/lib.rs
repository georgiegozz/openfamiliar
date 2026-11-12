//! Local pixel-art conversion: nearest-neighbor resize, palette quantization,
//! and binary alpha. Never fetches remote assets.

use color_quant::NeuQuant;
use image::imageops::FilterType;
use image::io::Limits;
use image::{DynamicImage, ImageDecoder, ImageError, RgbaImage};
use sha2::{Digest, Sha256};
use std::io::Cursor;
use thiserror::Error;

pub const SHEET_SIZE: u32 = 256;
pub const FRAME_SIZE: u32 = 64;
pub const SHEET_COLUMNS: u32 = 4;
pub const SHEET_ROWS: u32 = 4;
pub const MAX_INPUT_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_INPUT_DIMENSION: u32 = 4096;
pub const MAX_ALLOC_BYTES: u64 = 64 * 1024 * 1024;
pub const MIN_COLORS: u32 = 16;
pub const MAX_COLORS: u32 = 256;
pub const DEFAULT_COLORS: u32 = 64;
const ALPHA_CUTOFF: u8 = 128;
const NEUQUANT_SAMPLE: i32 = 10;

#[derive(Debug, Error)]
pub enum PixelError {
    #[error("image is empty")]
    Empty,
    #[error("image exceeds the {MAX_INPUT_BYTES} byte input limit")]
    InputTooLarge,
    #[error("unsupported or invalid image")]
    InvalidImage,
    #[error("image dimensions exceed {MAX_INPUT_DIMENSION}x{MAX_INPUT_DIMENSION}")]
    DimensionLimit,
    #[error("colors must be between {MIN_COLORS} and {MAX_COLORS}")]
    ColorCount,
    #[error("failed to encode png")]
    Encode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Resize the entire image to a 256x256 contact sheet.
    ContactSheet,
    /// Resize the entire image to a single 64x64 frame.
    SingleFrame,
    /// Resize to 64x64 and tile it across a 4x4 sheet.
    TiledMascot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelOptions {
    pub colors: u32,
    pub layout: Layout,
}

impl Default for PixelOptions {
    fn default() -> Self {
        Self {
            colors: DEFAULT_COLORS,
            layout: Layout::TiledMascot,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedImage {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub color_count: usize,
    pub opaque_pixels: usize,
    pub sha256: String,
}

pub fn decode_limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_INPUT_DIMENSION);
    limits.max_image_height = Some(MAX_INPUT_DIMENSION);
    limits.max_alloc = Some(MAX_ALLOC_BYTES);
    limits
}

const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// Reject PNGs whose IHDR already declares dimensions beyond our strict Limits
/// without asking the decoder to allocate a bitmap.
fn reject_oversized_png_header(bytes: &[u8]) -> Result<(), PixelError> {
    if bytes.len() < 24 || !bytes.starts_with(PNG_SIGNATURE) {
        return Ok(());
    }
    // 8 sig + 4 len + 4 "IHDR" + 8 width/height
    if &bytes[12..16] != b"IHDR" {
        return Ok(());
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if width > MAX_INPUT_DIMENSION || height > MAX_INPUT_DIMENSION {
        return Err(PixelError::DimensionLimit);
    }
    Ok(())
}

pub fn import_image(bytes: &[u8], options: PixelOptions) -> Result<ImportedImage, PixelError> {
    if bytes.is_empty() {
        return Err(PixelError::Empty);
    }
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(PixelError::InputTooLarge);
    }
    if !(MIN_COLORS..=MAX_COLORS).contains(&options.colors) {
        return Err(PixelError::ColorCount);
    }
    reject_oversized_png_header(bytes)?;

    let image = decode_bounded(bytes)?;
    let rgba = resize_for_layout(image, options.layout);
    let quantized = quantize_binary_alpha(&rgba, options.colors as usize);
    let png = encode_png(&quantized)?;
    let (color_count, opaque_pixels) = inspect(&quantized);
    let sha256 = sha256_hex(&png);
    Ok(ImportedImage {
        width: quantized.width(),
        height: quantized.height(),
        png,
        color_count,
        opaque_pixels,
        sha256,
    })
}

fn decode_bounded(bytes: &[u8]) -> Result<DynamicImage, PixelError> {
    let cursor = Cursor::new(bytes);
    let mut reader = image::io::Reader::new(cursor)
        .with_guessed_format()
        .map_err(|_| PixelError::InvalidImage)?;
    if reader.format().is_none() {
        return Err(PixelError::InvalidImage);
    }
    reader.limits(decode_limits());
    match reader.decode() {
        Ok(image) => Ok(image),
        Err(ImageError::Limits(_)) => Err(PixelError::DimensionLimit),
        Err(_) => Err(PixelError::InvalidImage),
    }
}

/// Peek at declared decoder dimensions after applying `Limits`.
/// Used by tests to prove oversized headers are rejected before allocation.
pub fn peek_dimensions(bytes: &[u8]) -> Result<(u32, u32), PixelError> {
    if bytes.is_empty() {
        return Err(PixelError::Empty);
    }
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(PixelError::InputTooLarge);
    }
    reject_oversized_png_header(bytes)?;
    let cursor = Cursor::new(bytes);
    let mut decoder =
        image::codecs::png::PngDecoder::new(cursor).map_err(|_| PixelError::InvalidImage)?;
    decoder
        .set_limits(decode_limits())
        .map_err(|_| PixelError::DimensionLimit)?;
    Ok(decoder.dimensions())
}

fn resize_for_layout(image: DynamicImage, layout: Layout) -> RgbaImage {
    let rgba = image.to_rgba8();
    match layout {
        Layout::ContactSheet => {
            image::imageops::resize(&rgba, SHEET_SIZE, SHEET_SIZE, FilterType::Nearest)
        }
        Layout::SingleFrame => {
            image::imageops::resize(&rgba, FRAME_SIZE, FRAME_SIZE, FilterType::Nearest)
        }
        Layout::TiledMascot => {
            let frame = image::imageops::resize(&rgba, FRAME_SIZE, FRAME_SIZE, FilterType::Nearest);
            let mut sheet = RgbaImage::new(SHEET_SIZE, SHEET_SIZE);
            for row in 0..SHEET_ROWS {
                for col in 0..SHEET_COLUMNS {
                    image::imageops::replace(
                        &mut sheet,
                        &frame,
                        (col * FRAME_SIZE) as i64,
                        (row * FRAME_SIZE) as i64,
                    );
                }
            }
            sheet
        }
    }
}

fn quantize_binary_alpha(source: &RgbaImage, colors: usize) -> RgbaImage {
    let mut raw = source.as_raw().clone();
    for px in raw.chunks_exact_mut(4) {
        if px[3] < ALPHA_CUTOFF {
            px[0] = 0;
            px[1] = 0;
            px[2] = 0;
            px[3] = 0;
        } else {
            px[3] = 255;
        }
    }
    let quant = NeuQuant::new(NEUQUANT_SAMPLE, colors, &raw);
    let mut out = raw;
    for px in out.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        let idx = quant.index_of(px);
        let mapped = quant.color_map_rgba();
        let base = idx * 4;
        px[0] = mapped[base];
        px[1] = mapped[base + 1];
        px[2] = mapped[base + 2];
        px[3] = 255;
    }
    RgbaImage::from_raw(source.width(), source.height(), out)
        .expect("quantized buffer matches source dimensions")
}

fn encode_png(image: &RgbaImage) -> Result<Vec<u8>, PixelError> {
    let mut out = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut out);
    image
        .write_with_encoder(encoder)
        .map_err(|_| PixelError::Encode)?;
    Ok(out)
}

fn inspect(image: &RgbaImage) -> (usize, usize) {
    use std::collections::BTreeSet;
    let mut colors = BTreeSet::new();
    let mut opaque = 0usize;
    for px in image.pixels() {
        if px.0[3] == 255 {
            opaque += 1;
            colors.insert((px.0[0], px.0[1], px.0[2]));
        } else {
            colors.insert((0, 0, 0));
        }
    }
    (colors.len(), opaque)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn sha256_of(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

/// Minimal PNG whose IHDR declares 60000x60000 without pixel payload.
pub fn oversized_png_header() -> Vec<u8> {
    // Precomputed: 8-bit RGBA IHDR 60000x60000 + IEND. No IDAT.
    hex::decode("89504e470d0a1a0a0000000d494844520000ea600000ea60080600000080d275420000000049454e44ae426082")
        .expect("static hex")
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid_png(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
        let mut img = RgbaImage::new(width, height);
        for px in img.pixels_mut() {
            *px = Rgba(color);
        }
        encode_png(&img).unwrap()
    }

    fn checker_png() -> Vec<u8> {
        let mut img = RgbaImage::new(32, 32);
        for (x, y, px) in img.enumerate_pixels_mut() {
            if (x / 4 + y / 4) % 2 == 0 {
                *px = Rgba([255, 0, 0, 255]);
            } else if x < 8 && y < 8 {
                *px = Rgba([0, 0, 255, 40]);
            } else {
                *px = Rgba([0, 255, 0, 255]);
            }
        }
        encode_png(&img).unwrap()
    }

    #[test]
    fn empty_input_is_rejected() {
        let err = import_image(&[], PixelOptions::default()).unwrap_err();
        assert!(matches!(err, PixelError::Empty));
    }

    #[test]
    fn oversized_byte_payload_is_rejected() {
        let huge = vec![0u8; MAX_INPUT_BYTES + 1];
        let err = import_image(&huge, PixelOptions::default()).unwrap_err();
        assert!(matches!(err, PixelError::InputTooLarge));
    }

    #[test]
    fn declared_60000_png_is_rejected_by_limits() {
        let bomb = oversized_png_header();
        assert!(bomb.len() < 100, "fixture must not allocate a bitmap");
        let err = peek_dimensions(&bomb).unwrap_err();
        assert!(matches!(err, PixelError::DimensionLimit));
        let err = import_image(&bomb, PixelOptions::default()).unwrap_err();
        assert!(
            matches!(err, PixelError::DimensionLimit | PixelError::InvalidImage),
            "{err:?}"
        );
    }

    #[test]
    fn contact_sheet_is_256_and_deterministic() {
        let png = checker_png();
        let opts = PixelOptions {
            colors: 32,
            layout: Layout::ContactSheet,
        };
        let first = import_image(&png, opts).unwrap();
        let second = import_image(&png, opts).unwrap();
        assert_eq!(first.width, SHEET_SIZE);
        assert_eq!(first.height, SHEET_SIZE);
        assert_eq!(first.sha256, second.sha256);
        assert_eq!(first.png, second.png);
        assert!(first.color_count <= 32);
        assert!(first.opaque_pixels > 0);
        assert_eq!(first.sha256, sha256_of(&first.png));
        assert_binary_alpha(&first.png);
    }

    #[test]
    fn single_frame_is_64() {
        let png = solid_png(120, 80, [10, 20, 30, 255]);
        let out = import_image(
            &png,
            PixelOptions {
                colors: 16,
                layout: Layout::SingleFrame,
            },
        )
        .unwrap();
        assert_eq!(out.width, FRAME_SIZE);
        assert_eq!(out.height, FRAME_SIZE);
        assert!(out.color_count <= 16);
        assert_binary_alpha(&out.png);
    }

    #[test]
    fn tiled_mascot_repeats_64_frame() {
        let png = solid_png(50, 50, [200, 100, 50, 255]);
        let out = import_image(
            &png,
            PixelOptions {
                colors: 16,
                layout: Layout::TiledMascot,
            },
        )
        .unwrap();
        assert_eq!(out.width, SHEET_SIZE);
        assert_eq!(out.height, SHEET_SIZE);
        assert_eq!(out.opaque_pixels, (SHEET_SIZE * SHEET_SIZE) as usize);
        assert_binary_alpha(&out.png);
    }

    #[test]
    fn color_count_bounds() {
        let png = solid_png(8, 8, [1, 2, 3, 255]);
        assert!(matches!(
            import_image(
                &png,
                PixelOptions {
                    colors: 15,
                    layout: Layout::SingleFrame
                }
            ),
            Err(PixelError::ColorCount)
        ));
        assert!(matches!(
            import_image(
                &png,
                PixelOptions {
                    colors: 257,
                    layout: Layout::SingleFrame
                }
            ),
            Err(PixelError::ColorCount)
        ));
    }

    fn assert_binary_alpha(png: &[u8]) {
        let decoded = image::load_from_memory(png).unwrap().to_rgba8();
        for px in decoded.pixels() {
            assert!(px.0[3] == 0 || px.0[3] == 255, "partial alpha {}", px.0[3]);
        }
    }
}
