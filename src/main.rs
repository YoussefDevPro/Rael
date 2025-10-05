use crossterm::{
    cursor::{Hide, Show},
    event::{poll, read, Event, KeyCode},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rael::{Canvas, Color};
use std::io::{stdout, Write};
use std::time::Duration;

// A simple struct to ensure cleanup is performed when it's dropped.
struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        _ = execute!(stdout(), LeaveAlternateScreen, Show);
        _ = disable_raw_mode();
    }
}

fn main() -> std::io::Result<()> {
    let _clean_up = CleanUp;
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let (width, height) = crossterm::terminal::size()?;
    let mut canvas = Canvas::new(
        width as usize,
        height as usize,
        Color { r: 0, g: 0, b: 0 }, // Default background color
    );

    let floor_y_terminal_cell = (height * 4 / 5) as usize;

    let mut frame = 0;

    // --- Main Game Loop ---
    loop {
        // --- Input Handling ---
        if poll(Duration::from_millis(0))? {
            if let Ok(Event::Key(key_event)) = read() {
                if key_event.code == KeyCode::Char('q') {
                    break; // Exit loop on 'q'
                }
            }
        }

        // --- Update and Draw Scene ---
        canvas.clear(); // Clear canvas to default color

        // Draw the floor
        for terminal_cell_y in floor_y_terminal_cell..height as usize {
            for x in 0..width as usize {
                let color = Color {
                    r: 50,
                    g: 50,
                    b: 50,
                };
                // Top half of the terminal cell
                canvas.set_pixel(x, terminal_cell_y * 2, 0, color);
                // Bottom half of the terminal cell
                canvas.set_pixel(x, terminal_cell_y * 2 + 1, 0, color);
            }
        }

        // Draw overlapping blocks with different z-layers

        // Block 1 (Red, z=1)
        let block1_color = Color { r: 255, g: 0, b: 0 };
        let block1_base_x = width as usize / 4;
        let block1_x_offset = ((frame as f32 * 0.03).sin() * 5.0) as isize;
        let block1_x = (block1_base_x as isize + block1_x_offset).max(0).min((width - 8) as isize) as usize;
        let block1_base_y = floor_y_terminal_cell - 10;
        let block1_y_offset = ((frame as f32 * 0.05).cos() * 3.0) as isize;
        let block1_y = (block1_base_y as isize + block1_y_offset).max(0).min((height - 8) as isize) as usize;
        let block1_size = 8;
        for y_offset in 0..block1_size {
            for x_offset in 0..block1_size {
                canvas.set_pixel(block1_x + x_offset, (block1_y + y_offset) * 2, 1, block1_color); // Top half
                canvas.set_pixel(block1_x + x_offset, (block1_y + y_offset) * 2 + 1, 1, block1_color); // Bottom half
            }
        }

        // Block 2 (Green, z=2) - overlaps Block 1
        let block2_color = Color { r: 0, g: 255, b: 0 };
        let block2_base_x = width as usize / 4 + 4;
        let block2_x_offset = ((frame as f32 * 0.04).cos() * 7.0) as isize;
        let block2_x = (block2_base_x as isize + block2_x_offset).max(0).min((width - 8) as isize) as usize;
        let block2_base_y = floor_y_terminal_cell - 8;
        let block2_y_offset = ((frame as f32 * 0.06).sin() * 4.0) as isize;
        let block2_y = (block2_base_y as isize + block2_y_offset).max(0).min((height - 8) as isize) as usize;
        let block2_size = 8;
        for y_offset in 0..block2_size {
            for x_offset in 0..block2_size {
                canvas.set_pixel(block2_x + x_offset, (block2_y + y_offset) * 2, 2, block2_color); // Top half
                canvas.set_pixel(block2_x + x_offset, (block2_y + y_offset) * 2 + 1, 2, block2_color); // Bottom half
            }
        }

        // Block 3 (Blue, z=3) - overlaps Block 2
        let block3_color = Color { r: 0, g: 0, b: 255 };
        let block3_base_x = width as usize / 4 + 8;
        let block3_x_offset = ((frame as f32 * 0.05).sin() * 6.0) as isize;
        let block3_x = (block3_base_x as isize + block3_x_offset).max(0).min((width - 8) as isize) as usize;
        let block3_base_y = floor_y_terminal_cell - 6;
        let block3_y_offset = ((frame as f32 * 0.07).cos() * 5.0) as isize;
        let block3_y = (block3_base_y as isize + block3_y_offset).max(0).min((height - 8) as isize) as usize;
        let block3_size = 8;
        for y_offset in 0..block3_size {
            for x_offset in 0..block3_size {
                canvas.set_pixel(block3_x + x_offset, (block3_y + y_offset) * 2, 3, block3_color); // Top half
                canvas.set_pixel(block3_x + x_offset, (block3_y + y_offset) * 2 + 1, 3, block3_color); // Bottom half
            }
        }

        // Render the canvas to the terminal
        let output = canvas.render();
        execute!(stdout, Print(output))?;
        stdout.flush()?;

        frame += 1;
        std::thread::sleep(Duration::from_millis(16)); // Aim for ~60 FPS
    }

    Ok(())
}