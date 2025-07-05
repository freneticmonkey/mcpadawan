use crate::error::ConversionError;
use core_graphics::image::CGImage;
use image::RgbaImage;

pub struct ImageConverter {
    reuse_buffer: Option<Vec<u8>>,
}

impl ImageConverter {
    pub fn new() -> Self {
        Self { reuse_buffer: None }
    }

    pub fn cgimage_to_rgba(&mut self, cg_image_ptr: *mut CGImage) -> Result<RgbaImage, ConversionError> {
        if cg_image_ptr.is_null() {
            return Err(ConversionError::NoDataProvider);
        }

        // For now, return a simple error - we'll need to use lower-level Core Graphics APIs
        // to properly extract data from CGImage. This is a simplified placeholder.
        Err(ConversionError::UnsupportedFormat {
            bits_per_pixel: 32,
            bits_per_component: 8,
        })
    }


    fn convert_bgra_to_rgba(
        &self,
        src: &[u8],
        width: usize,
        height: usize,
        bytes_per_row: usize,
        dst: &mut Vec<u8>,
    ) -> Result<(), ConversionError> {
        dst.reserve_exact(width * height * 4);

        for y in 0..height {
            let row_start = y * bytes_per_row;
            
            for x in 0..width {
                let src_idx = row_start + x * 4;
                
                if src_idx + 3 >= src.len() {
                    return Err(ConversionError::BufferTooSmall);
                }

                dst.push(src[src_idx + 2]); // R
                dst.push(src[src_idx + 1]); // G
                dst.push(src[src_idx]);     // B
                dst.push(src[src_idx + 3]); // A
            }
        }

        Ok(())
    }

    fn copy_rgba(
        &self,
        src: &[u8],
        width: usize,
        height: usize,
        bytes_per_row: usize,
        dst: &mut Vec<u8>,
    ) -> Result<(), ConversionError> {
        dst.reserve_exact(width * height * 4);

        if bytes_per_row == width * 4 {
            let total_bytes = width * height * 4;
            if src.len() < total_bytes {
                return Err(ConversionError::BufferTooSmall);
            }
            dst.extend_from_slice(&src[..total_bytes]);
        } else {
            for y in 0..height {
                let row_start = y * bytes_per_row;
                let row_end = row_start + width * 4;
                
                if row_end > src.len() {
                    return Err(ConversionError::BufferTooSmall);
                }
                
                dst.extend_from_slice(&src[row_start..row_end]);
            }
        }

        Ok(())
    }

    fn convert_rgb_to_rgba(
        &self,
        src: &[u8],
        width: usize,
        height: usize,
        bytes_per_row: usize,
        dst: &mut Vec<u8>,
    ) -> Result<(), ConversionError> {
        dst.reserve_exact(width * height * 4);

        for y in 0..height {
            let row_start = y * bytes_per_row;
            
            for x in 0..width {
                let src_idx = row_start + x * 3;
                
                if src_idx + 2 >= src.len() {
                    return Err(ConversionError::BufferTooSmall);
                }

                dst.push(src[src_idx]);     // R
                dst.push(src[src_idx + 1]); // G
                dst.push(src[src_idx + 2]); // B
                dst.push(255);              // A
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_reuse() {
        let mut converter = ImageConverter::new();
        assert!(converter.reuse_buffer.is_none());
        
        converter.reuse_buffer = Some(Vec::with_capacity(1024));
        let buffer = converter.reuse_buffer.take().unwrap();
        assert_eq!(buffer.capacity(), 1024);
    }
}