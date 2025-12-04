use qrcodegen::{QrCode, QrCodeEcc};
use slint::{Image, SharedPixelBuffer, Rgba8Pixel};

/// Generate a QR code image from text
pub fn generate_qr_code(text: &str) -> Image {
    let qr = QrCode::encode_text(text, QrCodeEcc::Medium).unwrap();
    let size = qr.size() as u32;
    
    // Scale up the QR code to make it visible (e.g., 4x per module)
    let scale = 4;
    let img_size = size * scale;
    
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(img_size, img_size);
    
    for y in 0..size {
        for x in 0..size {
            let color = if qr.get_module(x as i32, y as i32) {
                Rgba8Pixel { r: 0, g: 0, b: 0, a: 255 } // Black
            } else {
                Rgba8Pixel { r: 255, g: 255, b: 255, a: 255 } // White
            };
            
            // Fill scaled block
            for dy in 0..scale {
                for dx in 0..scale {
                    let px = x * scale + dx;
                    let py = y * scale + dy;
                    let offset = (py * img_size + px) as usize;
                    // Safe because we allocated correctly
                    // Using unsafe for direct buffer access would be faster but this is fine
                    // Slint's SharedPixelBuffer doesn't expose direct slice access easily in safe Rust 
                    // without cloning, so we construct it via make_mut_slice if possible or just rebuild
                }
            }
        }
    }
    
    // Simpler approach: Create raw buffer
    let mut pixels = Vec::with_capacity((img_size * img_size) as usize);
    for y in 0..img_size {
        for x in 0..img_size {
            let module_x = x / scale;
            let module_y = y / scale;
            if qr.get_module(module_x as i32, module_y as i32) {
                pixels.push(Rgba8Pixel { r: 0, g: 0, b: 0, a: 255 });
            } else {
                pixels.push(Rgba8Pixel { r: 255, g: 255, b: 255, a: 255 });
            }
        }
    }
    
    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        unsafe { std::slice::from_raw_parts(pixels.as_ptr() as *const u8, pixels.len() * 4) },
        img_size,
        img_size,
    );
    Image::from_rgba8(buffer)
}
