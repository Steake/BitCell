use slint::{Image, SharedPixelBuffer, Rgba8Pixel};

/// Render a cellular automaton grid to an image
pub fn render_grid(grid: &[Vec<u8>], width: u32, height: u32) -> Image {
    let scale = 4; // 4x4 pixels per cell (64x64 -> 256x256)
    let img_width = width * scale;
    let img_height = height * scale;
    
    let mut pixels = Vec::with_capacity((img_width * img_height) as usize);
    
    for y in 0..img_height {
        for x in 0..img_width {
            let cell_x = x / scale;
            let cell_y = y / scale;
            
            let color = if cell_y < height && cell_x < width {
                match grid[cell_y as usize][cell_x as usize] {
                    0 => Rgba8Pixel { r: 15, g: 23, b: 42, a: 255 }, // Background (Theme.background)
                    1 => Rgba8Pixel { r: 99, g: 102, b: 241, a: 255 }, // Player A (Theme.primary)
                    2 => Rgba8Pixel { r: 245, g: 158, b: 11, a: 255 }, // Player B (Theme.accent)
                    _ => Rgba8Pixel { r: 255, g: 255, b: 255, a: 255 }, // Unknown
                }
            } else {
                Rgba8Pixel { r: 0, g: 0, b: 0, a: 255 }
            };
            
            // Add grid lines
            let is_grid_line = x % scale == 0 || y % scale == 0;
            let final_color = if is_grid_line {
                Rgba8Pixel { r: 30, g: 41, b: 59, a: 255 } // Grid line color
            } else {
                color
            };
            
            pixels.push(final_color);
        }
    }
    
    // Convert Vec<Rgba8Pixel> to Vec<u8> safely
    let mut pixel_bytes = Vec::with_capacity(pixels.len() * 4);
    for pixel in &pixels {
        pixel_bytes.push(pixel.r);
        pixel_bytes.push(pixel.g);
        pixel_bytes.push(pixel.b);
        pixel_bytes.push(pixel.a);
    }
    
    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        &pixel_bytes,
        img_width,
        img_height,
    );
    Image::from_rgba8(buffer)
}

/// Generate a mock grid for testing
pub fn generate_mock_grid(width: u32, height: u32, step: u32) -> Vec<Vec<u8>> {
    let mut grid = vec![vec![0; width as usize]; height as usize];
    
    // Simple glider-like pattern that moves
    let offset = (step as usize) % (width as usize - 2);
    
    if offset + 2 < width as usize && 2 < height as usize {
        grid[0][offset + 1] = 1;
        grid[1][offset + 2] = 1;
        grid[2][offset] = 1;
        grid[2][offset + 1] = 1;
        grid[2][offset + 2] = 1;
    }
    
    grid
}
