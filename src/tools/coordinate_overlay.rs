//! Coordinate overlay module for screenshots
//!
//! Draws visible X/Y axes, tick marks, and pixel coordinates on screenshots
//! to enable AI agents to accurately identify screen positions.

use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

/// Color for the coordinate overlay (bright cyan)
pub const OVERLAY_COLOR: Rgba<u8> = Rgba([0, 255, 255, 255]);
/// Semi-transparent color for minor elements
pub const OVERLAY_MINOR_COLOR: Rgba<u8> = Rgba([0, 255, 255, 128]);
/// Background color for labels
pub const LABEL_BG_COLOR: Rgba<u8> = Rgba([0, 0, 0, 180]);
/// Text color
pub const TEXT_COLOR: Rgba<u8> = Rgba([255, 255, 255, 255]);

/// Options for coordinate overlay
#[derive(Debug, Clone)]
pub struct CoordinateOverlayOptions {
    /// Whether to show the overlay
    pub show_overlay: bool,
    /// Interval between major ticks in pixels
    pub tick_interval: u32,
    /// Interval between minor ticks in pixels
    pub minor_tick_interval: u32,
    /// Line width for axes and major ticks
    pub line_width: u32,
}

impl Default for CoordinateOverlayOptions {
    fn default() -> Self {
        Self {
            show_overlay: false,
            tick_interval: 100,
            minor_tick_interval: 50,
            line_width: 2,
        }
    }
}

/// Apply coordinate overlay to an image
pub fn apply_coordinate_overlay(
    image: &DynamicImage,
    options: &CoordinateOverlayOptions,
) -> DynamicImage {
    if !options.show_overlay {
        return image.clone();
    }

    let (width, height) = image.dimensions();
    let mut rgba_image = image.to_rgba8();

    // Draw Y-axis (left edge)
    draw_vertical_line(&mut rgba_image, 0, 0, height, OVERLAY_COLOR, options.line_width);

    // Draw X-axis (bottom edge)
    draw_horizontal_line(&mut rgba_image, 0, height - 1, width, OVERLAY_COLOR, options.line_width);

    // Draw minor ticks on Y-axis
    let mut y = options.minor_tick_interval;
    while y < height {
        if y % options.tick_interval != 0 {
            draw_vertical_tick(&mut rgba_image, 0, y, 10, OVERLAY_MINOR_COLOR, 1);
        }
        y += options.minor_tick_interval;
    }

    // Draw minor ticks on X-axis
    let mut x = options.minor_tick_interval;
    while x < width {
        if x % options.tick_interval != 0 {
            draw_horizontal_tick(&mut rgba_image, x, height - 1, 10, OVERLAY_MINOR_COLOR, 1);
        }
        x += options.minor_tick_interval;
    }

    // Draw major ticks and labels on Y-axis
    let mut y = options.tick_interval;
    while y < height {
        draw_vertical_tick(&mut rgba_image, 0, y, 15, OVERLAY_COLOR, options.line_width);
        draw_y_label(&mut rgba_image, y, y as i32);
        y += options.tick_interval;
    }

    // Draw major ticks and labels on X-axis
    let mut x = options.tick_interval;
    while x < width {
        draw_horizontal_tick(&mut rgba_image, x, height - 1, 15, OVERLAY_COLOR, options.line_width);
        draw_x_label(&mut rgba_image, x, height, x as i32);
        x += options.tick_interval;
    }

    // Draw origin label
    draw_origin_label(&mut rgba_image);

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

fn draw_vertical_tick(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    tick_length: u32,
    color: Rgba<u8>,
    width: u32,
) {
    draw_vertical_line(image, x, y.saturating_sub(tick_length / 2), y.saturating_add(tick_length / 2), color, width);
}

fn draw_horizontal_tick(
    image: &mut RgbaImage,
    x: u32,
    y: u32,
    tick_length: u32,
    color: Rgba<u8>,
    width: u32,
) {
    draw_horizontal_line(image, x.saturating_sub(tick_length / 2), y, x.saturating_add(tick_length / 2), color, width);
}

/// Simple 5x7 pixel font for digits (0-9) and comma
/// Each row is a 5-bit value representing the pixel pattern
const FONT_DIGITS: [[u8; 7]; 10] = [
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
];

const FONT_COMMA: [u8; 7] = [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b01000];

/// Draw a character using simple pixel font
fn draw_char(image: &mut RgbaImage, x: u32, y: u32, char_idx: u8, color: Rgba<u8>) {
    let char_bits = if char_idx == 10 {
        FONT_COMMA
    } else if (char_idx as usize) < 10 {
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

/// Draw a number at position
fn draw_number(image: &mut RgbaImage, x: u32, y: u32, num: i32, color: Rgba<u8>) {
    let abs_num = num.abs();
    let digits: Vec<u8> = if abs_num == 0 {
        vec![0]
    } else {
        let mut d = Vec::new();
        let mut n = abs_num;
        while n > 0 {
            d.push((n % 10) as u8);
            n /= 10;
        }
        d.reverse();
        d
    };

    let num_digits = digits.len();
    let start_x = if num >= 0 { x } else { x + 6 }; // reserve space for minus

    // Draw minus sign if negative
    if num < 0 && start_x >= 1 {
        for col in 0..5 {
            let px = start_x - 1 + col;
            let py = y + 3;
            if px < image.width() && py < image.height() {
                image.put_pixel(px, py, color);
            }
        }
    }

    for (i, &digit) in digits.iter().enumerate() {
        let digit_x = start_x + (num_digits - 1 - i) as u32 * 6;
        if digit_x < image.width() {
            draw_char(image, digit_x, y, digit, color);
        }
    }
}

/// Draw Y-axis label (coordinate number)
fn draw_y_label(image: &mut RgbaImage, y: u32, coord: i32) {
    let label_x = 20u32;
    let label_y = y.saturating_sub(4);

    // Draw background
    for dy in 0..8u32 {
        for dx in 0..28u32 {
            let px = label_x + dx;
            let py = label_y + dy;
            if px < image.width() && py < image.height() {
                image.put_pixel(px, py, LABEL_BG_COLOR);
            }
        }
    }

    draw_number(image, label_x + 2, label_y + 1, coord, TEXT_COLOR);
}

/// Draw X-axis label (coordinate number)
fn draw_x_label(image: &mut RgbaImage, x: u32, image_height: u32, coord: i32) {
    let label_y = image_height.saturating_sub(20);
    let label_x = x.saturating_sub(10);

    // Draw background
    for dy in 0..8u32 {
        for dx in 0..28u32 {
            let px = label_x + dx;
            let py = label_y + dy;
            if px < image.width() && py < image.height() {
                image.put_pixel(px, py, LABEL_BG_COLOR);
            }
        }
    }

    draw_number(image, label_x + 2, label_y + 1, coord, TEXT_COLOR);
}

/// Draw origin label at (0, 0)
fn draw_origin_label(image: &mut RgbaImage) {
    let label_x = 20u32;
    let label_y = image.height().saturating_sub(20);

    // Draw background
    for dy in 0..8u32 {
        for dx in 0..36u32 {
            let px = label_x + dx;
            let py = label_y + dy;
            if px < image.width() && py < image.height() {
                image.put_pixel(px, py, LABEL_BG_COLOR);
            }
        }
    }

    // Draw "0,0"
    draw_char(image, label_x + 2, label_y + 1, 0, TEXT_COLOR);
    draw_char(image, label_x + 8, label_y + 1, 10, TEXT_COLOR); // comma
    draw_char(image, label_x + 14, label_y + 1, 0, TEXT_COLOR);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_options_default() {
        let options = CoordinateOverlayOptions::default();
        assert!(!options.show_overlay);
        assert_eq!(options.tick_interval, 100);
    }

    #[test]
    fn test_apply_no_overlay() {
        let img = DynamicImage::new_rgb8(100, 100);
        let options = CoordinateOverlayOptions::default();
        let result = apply_coordinate_overlay(&img, &options);
        assert_eq!(result.dimensions(), (100, 100));
    }
}
