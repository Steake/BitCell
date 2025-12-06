use qrcodegen::{QrCode, QrCodeEcc};
use slint::{Image, SharedPixelBuffer, Rgba8Pixel};

/// Generate a QR code image from text
pub fn generate_qr_code(text: &str) -> Image {
    let qr = QrCode::encode_text(text, QrCodeEcc::Medium).unwrap();
    let size = qr.size() as u32;
    
    // Scale up the QR code to make it visible (e.g., 4x per module)
    let scale = 4;
    let img_size = size * scale;
    
    // Create pixel data safely
    let mut pixel_bytes = Vec::with_capacity((img_size * img_size * 4) as usize);
    for y in 0..img_size {
        for x in 0..img_size {
            let module_x = x / scale;
            let module_y = y / scale;
            if qr.get_module(module_x as i32, module_y as i32) {
                // Black module
                pixel_bytes.push(0);   // r
                pixel_bytes.push(0);   // g
                pixel_bytes.push(0);   // b
                pixel_bytes.push(255); // a
            } else {
                // White module
                pixel_bytes.push(255); // r
                pixel_bytes.push(255); // g
                pixel_bytes.push(255); // b
                pixel_bytes.push(255); // a
            }
        }
    }
    
    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &pixel_bytes,
        img_size,
        img_size,
    );
    Image::from_rgba8(buffer)
}
