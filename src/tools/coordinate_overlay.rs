//! Grid overlay module for screenshots
//!
//! Draws hierarchical grid overlay for AI to identify screen positions.
//! Level 1: 6x4 grid (A1 ~ F4)
//! Level 2: 3x3 sub-grid (a1 ~ c3) within each cell

use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

/// Color for Level 1 grid (blue)
pub const GRID_COLOR_L1: Rgba<u8> = Rgba([0, 120, 255, 255]);
/// Color for Level 2 grid (gray)
pub const GRID_COLOR_L2: Rgba<u8> = Rgba([150, 150, 150, 200]);
/// Background color for cell labels
pub const LABEL_BG_COLOR: Rgba<u8> = Rgba([0, 0, 0, 200]);
/// Text color for grid labels
pub const TEXT_COLOR: Rgba<u8> = Rgba([255, 255, 255, 255]);
/// Highlight color for a specific cell
pub const HIGHLIGHT_COLOR: Rgba<u8> = Rgba([255, 50, 50, 180]);

/// Options for grid overlay
#[derive(Debug, Clone)]
pub struct CoordinateOverlayOptions {
    /// Whether to show the overlay
    pub show_overlay: bool,
    /// Level 1 grid columns (default: 6 for A-F)
    pub grid_cols: u32,
    /// Level 1 grid rows (default: 4 for 1-4)
    pub grid_rows: u32,
    /// Line width for grid lines
    pub line_width: u32,
    /// Highlight a specific cell (e.g., "B3" or "B3-b2")
    pub highlight_cell: Option<String>,
}

impl Default for CoordinateOverlayOptions {
    fn default() -> Self {
        Self {
            show_overlay: false,
            grid_cols: 6,
            grid_rows: 4,
            line_width: 2,
            highlight_cell: None,
        }
    }
}

/// Column labels for grid (A-F for 6 columns)
fn get_col_label(col: u32) -> char {
    (b'A' + col as u8) as char
}

/// Row labels for grid (1-4 for 4 rows)
fn get_row_label(row: u32) -> u32 {
    row + 1
}

/// Apply grid overlay to an image
pub fn apply_coordinate_overlay(
    image: &DynamicImage,
    options: &CoordinateOverlayOptions,
) -> DynamicImage {
    if !options.show_overlay {
        return image.clone();
    }

    let (width, height) = image.dimensions();
    let mut rgba_image = image.to_rgba8();

    // Calculate cell dimensions
    let cell_width = width / options.grid_cols;
    let cell_height = height / options.grid_rows;

    // Draw Level 1 grid lines
    // Vertical lines
    for col in 0..=options.grid_cols {
        let x = col * cell_width;
        draw_vertical_line(&mut rgba_image, x, 0, height, GRID_COLOR_L1, options.line_width);
    }

    // Horizontal lines
    for row in 0..=options.grid_rows {
        let y = row * cell_height;
        draw_horizontal_line(&mut rgba_image, 0, y, width, GRID_COLOR_L1, options.line_width);
    }

    // Draw cell IDs at top-left corner of each cell
    for row in 0..options.grid_rows {
        for col in 0..options.grid_cols {
            let cell_id = format!("{}{}", get_col_label(col), get_row_label(row));
            let x = col * cell_width + 5;
            let y = row * cell_height + 5;
            draw_cell_label(&mut rgba_image, x, y, &cell_id);
        }
    }

    // Draw grid dimensions info at bottom
    let info = format!("Grid: {}x{} | Cell: {}x{}", options.grid_cols, options.grid_rows, cell_width, cell_height);
    draw_info_label(&mut rgba_image, width, height, &info);

    DynamicImage::ImageRgba8(rgba_image)
}

fn draw_vertical_line(
    image: &mut RgbaImage,
    x: u32,
    y_start: u32,
    y_end: u32,
    color: Rgba<u8>,
    width: u32,
) {
    for y in y_start..y_end {
        for w in 0..width {
            if x + w < image.width() && y < image.height() {
                image.put_pixel(x + w, y, color);
            }
        }
    }
}

fn draw_horizontal_line(
    image: &mut RgbaImage,
    x_start: u32,
    y: u32,
    x_end: u32,
    color: Rgba<u8>,
    width: u32,
) {
    for x in x_start..x_end {
        for w in 0..width {
            if x < image.width() && y + w < image.height() {
                image.put_pixel(x, y + w, color);
            }
        }
    }
}

/// Simple 5x7 pixel font for digits (0-9) and letters (A-F)
const FONT_DIGITS: [[u8; 7]; 16] = [
    [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110], // 0
    [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110], // 1
    [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111], // 2
    [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110], // 3
    [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010], // 4
    [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110], // 5
    [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110], // 6
    [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000], // 7
    [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110], // 8
    [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100], // 9
    [0b01110, 0b00000, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110], // A (10)
    [0b11100, 0b10010, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110], // B (11)
    [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110], // C (12)
    [0b11100, 0b10010, 0b10001, 0b10001, 0b10001, 0b10010, 0b11100], // D (13)
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111], // E (14)
    [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000], // F (15)
];

const FONT_COMMA: [u8; 7] = [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b01000];

/// Draw a character using simple pixel font
fn draw_char(image: &mut RgbaImage, x: u32, y: u32, char_idx: u8, color: Rgba<u8>) {
    let char_bits = if char_idx == 10 {
        FONT_COMMA
    } else if (char_idx as usize) < 16 {
        FONT_DIGITS[char_idx as usize]
    } else {
        return;
    };

    for row in 0u32..7 {
        let bits = char_bits[row as usize];
        for col in 0..5 {
            if (bits >> (4 - col)) & 1 == 1 {
                let px = x + col;
                let py = y + row;
                if px < image.width() && py < image.height() {
                    image.put_pixel(px, py, color);
                }
            }
        }
    }
}

/// Draw a cell label (e.g., "B3", "A1")
fn draw_cell_label(image: &mut RgbaImage, x: u32, y: u32, label: &str) {
    // Draw background
    let bg_width = 28u32;
    let bg_height = 10u32;
    for dy in 0..bg_height {
        for dx in 0..bg_width {
            let px = x + dx;
            let py = y + dy;
            if px < image.width() && py < image.height() {
                image.put_pixel(px, py, LABEL_BG_COLOR);
            }
        }
    }

    // Draw label characters
    let chars: Vec<char> = label.chars().collect();
    let mut offset_x = x + 2;

    for ch in chars {
        let char_idx = if ch.is_ascii_digit() {
            ch.to_digit(10).unwrap() as u8
        } else if ch.is_ascii_uppercase() {
            (ch as u8 - b'A') + 10
        } else if ch == ',' {
            10 // comma
        } else {
            continue;
        };

        draw_char(image, offset_x, y + 2, char_idx, TEXT_COLOR);
        offset_x += 7;
    }
}

/// Draw info label at bottom of image
fn draw_info_label(image: &mut RgbaImage, img_width: u32, img_height: u32, info: &str) {
    let y = img_height - 15;
    let x = 5u32;

    // Draw background
    let bg_width = (info.len() as u32) * 7 + 4;
    for dy in 0..12u32 {
        for dx in 0..bg_width {
            let px = x + dx;
            let py = y + dy;
            if px < image.width() && py < image.height() {
                image.put_pixel(px, py, LABEL_BG_COLOR);
            }
        }
    }

    // Draw info text (simplified - just ASCII)
    let chars: Vec<char> = info.chars().collect();
    let mut offset_x = x + 2;

    for ch in chars {
        let char_idx = if ch.is_ascii_digit() {
            ch.to_digit(10).unwrap() as u8
        } else if ch.is_ascii_uppercase() {
            (ch as u8 - b'A') + 10
        } else if ch == 'x' {
            10 // use comma as 'x' substitute
        } else {
            offset_x += 5;
            continue;
        };
        draw_char(image, offset_x, y + 3, char_idx, TEXT_COLOR);
        offset_x += 7;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_options_default() {
        let options = CoordinateOverlayOptions::default();
        assert!(!options.show_overlay);
        assert_eq!(options.grid_cols, 6);
        assert_eq!(options.grid_rows, 4);
    }

    #[test]
    fn test_apply_no_overlay() {
        let img = DynamicImage::new_rgb8(100, 100);
        let options = CoordinateOverlayOptions::default();
        let result = apply_coordinate_overlay(&img, &options);
        assert_eq!(result.dimensions(), (100, 100));
    }

    #[test]
    fn test_col_labels() {
        assert_eq!(get_col_label(0), 'A');
        assert_eq!(get_col_label(5), 'F');
    }

    #[test]
    fn test_row_labels() {
        assert_eq!(get_row_label(0), 1);
        assert_eq!(get_row_label(3), 4);
    }
}
