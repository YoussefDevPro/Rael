//! A simple terminal rendering engine for creating text-based UIs and games.

#![warn(missing_docs)]

/// Represents an RGB color with red, green, and blue components.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// The red component of the color (0-255).
    pub r: u8,
    /// The green component of the color (0-255).
    pub g: u8,
    /// The blue component of the color (0-255).
    pub b: u8,
}

/// Represents a single half-block pixel with a specific color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalPixel {
    /// The color of this half-block pixel.
    pub color: Color,
}

/// Represents a single terminal character cell after compositing half-block pixels.
/// This is used for differential rendering, storing the final top and bottom colors
/// that will be displayed in a single terminal character cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompositedCell {
    /// The color of the top half of the terminal character cell.
    pub top_color: Color,
    /// The color of the bottom half of the terminal character cell.
    pub bottom_color: Color,
}

/// A canvas for drawing to the terminal, like a digital picasso.
///
/// The `Canvas` represents a 3D grid of `TerminalPixel`s that can be drawn to.
/// Each terminal character cell is composed of two half-block pixels (top and bottom).
/// The `z` coordinate in `set_pixel` determines the layering (depth).
/// It uses a differential rendering approach to only update the parts of the screen that have changed.
pub struct Canvas {
    /// The width of the canvas in terminal character columns.
    pub width: usize,
    /// The height of the canvas in terminal character rows.
    pub height: usize,
    /// Stores the 3D grid of half-block pixels. Indexed by (x, y_half_block, z_layer).
    pixels: Vec<TerminalPixel>,
    /// Stores the 2D grid of currently composited terminal cells. Used for rendering.
    composited_cells: Vec<CompositedCell>,
    /// Stores the 2D grid of previously composited terminal cells. Used for differential rendering.
    previous_composited_cells: Vec<CompositedCell>,
    /// The default color used for clearing the canvas and for transparent pixels.
    pub default_color: Color,
    /// The maximum number of z-layers supported by the canvas.
    max_z_layers: usize,
}

impl Canvas {
    const DEFAULT_MAX_Z_LAYERS: usize = 10;

    /// Creates a new `Canvas` with the given width, height, and default color.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the canvas in terminal character columns.
    /// * `height` - The height of the canvas in terminal character rows.
    /// * `default_color` - The default background color for the canvas.
    ///
    /// # Returns
    ///
    /// A new `Canvas` instance.
    pub fn new(width: usize, height: usize, default_color: Color) -> Self {
        let initial_pixel = TerminalPixel {
            color: default_color,
        };

        let initial_composited_cell = CompositedCell {
            top_color: default_color,
            bottom_color: default_color,
        };

        let opposite_color = Color {
            r: 255 - default_color.r,
            g: 255 - default_color.g,
            b: 255 - default_color.b,
        };
        let different_composited_cell = CompositedCell {
            top_color: opposite_color,
            bottom_color: opposite_color,
        };

        let total_half_block_pixels = width * height * 2 * Self::DEFAULT_MAX_Z_LAYERS;
        let total_terminal_cells = width * height;

        Self {
            width,
            height,
            pixels: vec![initial_pixel; total_half_block_pixels],
            composited_cells: vec![initial_composited_cell; total_terminal_cells],
            previous_composited_cells: vec![different_composited_cell; total_terminal_cells],
            default_color,
            max_z_layers: Self::DEFAULT_MAX_Z_LAYERS,
        }
    }

    /// Clears the entire canvas to the `default_color`.
    /// All half-block pixels across all z-layers are reset to the `default_color`.
    pub fn clear(&mut self) {
        let initial_pixel = TerminalPixel {
            color: self.default_color,
        };
        for pixel in self.pixels.iter_mut() {
            *pixel = initial_pixel;
        }
    }

    fn get_index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= self.width { return None; }
        if y >= self.height * 2 { return None; } // y is now half-block row
        if z >= self.max_z_layers { return None; }

        Some(x + (y * self.width) + (z * self.width * self.height * 2))
    }

    /// Sets a half-block pixel at the specified (x, y) coordinate and z-layer with the given color.
    ///
    /// # Arguments
    ///
    /// * `x` - The terminal column coordinate (0-indexed).
    /// * `y` - The half-block row coordinate (0-indexed).
    ///         - `y = 0` corresponds to the top half of the first terminal cell row.
    ///         - `y = 1` corresponds to the bottom half of the first terminal cell row.
    ///         - `y = 2` corresponds to the top half of the second terminal cell row, and so on.
    /// * `z` - The z-layer (depth) of the pixel. Higher `z` values are drawn on top of lower `z` values.
    /// * `color` - The `Color` to set for the pixel.
    pub fn set_pixel(&mut self, x: usize, y: usize, z: usize, color: Color) {
        if let Some(index) = self.get_index(x, y, z) {
            let pixel = &mut self.pixels[index];
            pixel.color = color;
        }
    }

    /// Renders the current state of the canvas to a string containing ANSI escape codes.
    ///
    /// This function composites all z-layers for each terminal character cell to determine
    /// the final top and bottom half-block colors. It then compares this composited state
    /// with the previous frame's state and returns a string containing only the necessary
    /// ANSI escape codes to update the terminal, optimizing for minimal output.
    ///
    /// # Returns
    ///
    /// A `String` containing ANSI escape codes to update the terminal.
    pub fn render(&mut self) -> String {
        let mut buffer = String::new();
        for terminal_cell_y in 0..self.height {
            for terminal_cell_x in 0..self.width {
                let top_half_pixel_y = terminal_cell_y * 2;
                let bottom_half_pixel_y = terminal_cell_y * 2 + 1;

                let mut current_top_color = self.default_color;
                let mut current_bottom_color = self.default_color;

                // Find the highest z-layer color for the top half-block
                for z in (0..self.max_z_layers).rev() {
                    if let Some(index) = self.get_index(terminal_cell_x, top_half_pixel_y, z) {
                        let pixel = &self.pixels[index];
                        if pixel.color != self.default_color {
                            current_top_color = pixel.color;
                            break;
                        }
                    }
                }

                // Find the highest z-layer color for the bottom half-block
                for z in (0..self.max_z_layers).rev() {
                    if let Some(index) = self.get_index(terminal_cell_x, bottom_half_pixel_y, z) {
                        let pixel = &self.pixels[index];
                        if pixel.color != self.default_color {
                            current_bottom_color = pixel.color;
                            break;
                        }
                    }
                }

                // Create a temporary CompositedCell for comparison with previous frame
                let current_composited_cell = CompositedCell {
                    top_color: current_top_color,
                    bottom_color: current_bottom_color,
                };

                let terminal_cell_index = terminal_cell_y * self.width + terminal_cell_x;

                // Compare with previous composited cell
                if current_composited_cell != self.previous_composited_cells[terminal_cell_index] {
                    buffer.push_str(&format!("\u{1b}[{};{}H", terminal_cell_y + 1, terminal_cell_x + 1));
                    buffer.push_str(&format!(
                        "\u{1b}[48;2;{};{};{}m\u{1b}[38;2;{};{};{}m▄",
                        current_top_color.r,
                        current_top_color.g,
                        current_top_color.b,
                        current_bottom_color.r,
                        current_bottom_color.g,
                        current_bottom_color.b
                    ));
                }
                // Update composited_cells with the current composited cell
                self.composited_cells[terminal_cell_index] = current_composited_cell;
            }
        }
        self.previous_composited_cells = self.composited_cells.clone();
        buffer
    }
}
