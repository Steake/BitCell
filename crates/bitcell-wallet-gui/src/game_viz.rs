use slint::{Image, SharedPixelBuffer, Rgba8Pixel};

/// Render a cellular automaton grid to an image
pub fn render_grid(grid: &[Vec<u8>], width: u32, height: u32) -> Image {
    let scale = 4; // 4x4 pixels per cell (64x64 -> 256x256)
    let img_width = width * scale;
    let img_height = height * scale;
    
    // Create pixel data safely as raw bytes
    let mut pixel_bytes = Vec::with_capacity((img_width * img_height * 4) as usize);
    
    for y in 0..img_height {
        for x in 0..img_width {
            let cell_x = x / scale;
            let cell_y = y / scale;
            
            // Determine base color
            let (r, g, b) = if cell_y < height && cell_x < width {
                match grid[cell_y as usize][cell_x as usize] {
                    0 => (15, 23, 42),      // Background (Theme.background)
                    1 => (99, 102, 241),    // Player A (Theme.primary)
                    2 => (245, 158, 11),    // Player B (Theme.accent)
                    _ => (255, 255, 255),   // Unknown
                }
            } else {
                (0, 0, 0)
            };
            
            // Add grid lines
            let is_grid_line = x % scale == 0 || y % scale == 0;
            let (final_r, final_g, final_b) = if is_grid_line {
                (30, 41, 59) // Grid line color
            } else {
                (r, g, b)
            };
            
            pixel_bytes.push(final_r);
            pixel_bytes.push(final_g);
            pixel_bytes.push(final_b);
            pixel_bytes.push(255); // alpha
        }
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
