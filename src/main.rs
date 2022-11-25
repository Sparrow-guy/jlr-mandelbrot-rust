

// Program:  JLR-Mandelbrot
// Author:  Jean-Luc Romano
// Date started:  Thursday, November 17, 2022


// ----------
// Revisions:
// ----------
// 2022-11-17:  Initial version.  (No zooming capabilities.)
// 2022-11-18:  Added threshold to escape_value() function.
// 2022-11-18:  Added ability to zoom in and out.
// 2022-11-19:  Drawing the set now happens from the center -> out.
//              (Instead of from top to bottom.)
// 2022-11-21:  Added ability to save a screenshot of current image.
// 2022-11-23:  Added the ability to specify window size with --size=SIZE.
// ----------


/*///////////////////////////////////////////////////////////////////
NOTE:  This program uses several crates in its code, so to compile
       this program you need to place the following lines:

chrono = "0.4.23"
image = "0.23"
minifb = "0.23"

       in the [dependencies] section of your "Cargo.toml" file.
*////////////////////////////////////////////////////////////////////


// The following "allow"s are for development only:
// #![allow(dead_code)]
// #![allow(unused_imports)]
// #![allow(unused_mut)]
// #![allow(unused_variables)]


// The default width and height of the display window in pixels:
const DEFAULT_WINDOW_SIZE: usize = 512;


// Defining your own color palette is pretty easy if you know the RGB
// value of each color.
//
// First, decide on the color for a point belonging to the actual set,
// and set it as MANDELBROT_SET_COLOR.
//
// Then, decide what RGB triplet gets returned for a given i (iteration value).
//
// (Note:  These u8 triplets range from 0 to 255 (inclusive).)
fn color(i: Option<usize>) -> (u8, u8, u8) {

    const MANDELBROT_SET_COLOR: (u8, u8, u8) = (0, 0, 102);  // (dark blue)
    if i == None {
        return MANDELBROT_SET_COLOR;
    }

    let i = i.unwrap();

    // If you want to write your own code that takes i as input
    // and returns a u8 RGB triplet, do it here.

    const NUM_COLORS_PER_LEG: usize = 30;
    let num_colors = NUM_COLORS_PER_LEG * 3;
    let i = i % num_colors;
    let remainder = i % NUM_COLORS_PER_LEG;

    let value1 = (NUM_COLORS_PER_LEG - remainder) * 255 / NUM_COLORS_PER_LEG;
    let value2 = remainder * 255 / NUM_COLORS_PER_LEG;

    let value1 = value1.try_into().unwrap();
    let value2 = value2.try_into().unwrap();

    let leg = i / NUM_COLORS_PER_LEG;
    match leg {
        0 => (value1, value2, 0),
        1 => (0, value1, value2),
        2 => (value2, 0, value1),
        // Should never get here, but include just in case:
        _ => { assert!(false); (0, 0, 0) }
    }
}


// A convenience function to turn RBG values
// (from 0 to 255, inclusive) into a u32 integer.
fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}


// Given a pixel coordinate, this will return the next
// pixel in a (straight-line) spiral that spirals outward.
fn next_pixel_coordinate(row: isize, column: isize) -> (isize, isize) {
    if (row, column) == (0, 0) {
        return (0, 1)  // Go right.
    }

    // In case you're wondering, I figured out the following
    // logic by drawing a straight-line spiral on grid paper.
    // No way I could have done it in my head!

    if row == column {
        if column > 0 {
            return (row, column + 1)  // Go right.
        } else {
            return (row + 1, column)  // Go down.
        }
    }

    if row == -column {
        if column > 0 {
            return (row, column - 1)  // Go left.
        } else {
            return (row, column + 1)  // Go right.
        }
    }

    if column > 0 && column > row.abs() {
        return (row - 1, column)  // Go up.
    }

    if column < 0 && -column > row.abs() {
        return (row + 1, column)  // Go down.
    }

    if row > 0 && row > column.abs() {
        return (row, column + 1)  // Go right.
    }

    if row < 0 && -row > column.abs() {
        return (row, column - 1)  // Go left.
    }

    dbg!("Bad row,column value:  {},{}", row, column);
    assert!(false);  // Should never get here, but include just in case.
    (0, 0)
}


// This is just like the fucntion next_pixel_coordinate(),
// except that it doesn't start at (0, 0), but rather at
// (center_row, center_column) -- so we can spiral around
// any arbitrary point we want.
fn next_pixel_coordinate_with_offset(row: isize, column: isize,
                                     center_row: usize, center_column: usize)
    -> (isize, isize) {
    let (row_to_use, column_to_use) = (row - center_row as isize, column - center_column as isize);
    let (returned_row, returned_column) = next_pixel_coordinate(row_to_use, column_to_use);
    return (returned_row + center_row as isize, returned_column + center_column as isize);
}


// The main Mandelbrot set calculation function.
// Given an (x, y) coordinate, it will return the number
// of iterations needed to determine that the coordinate
// is not part of the Mandelbrot set (or None if it is
// part of the set).
fn escape_value(x:f64, y:f64, threshold: Option<f64>) -> Option<usize> {
    let x0 = x;
    let y0 = y;
    let threshold = threshold.unwrap_or(0.0);

    let mut iterations = 0;
    let mut x_slow = x;
    let mut y_slow = y;
    let mut x_fast = x;
    let mut y_fast = y;

    let _start_of_loop = std::time::Instant::now();

    loop {
        let x_squared = x_fast * x_fast;
        let y_squared = y_fast * y_fast;
        if x_squared + y_squared > 4.0 {
            return Some(iterations)
        }
        let difference_of_squares = x_squared - y_squared;
        let double_the_product = 2.0 * x_fast * y_fast;
        (x_fast, y_fast) = (difference_of_squares + x0, double_the_product + y0);
        iterations += 1;

        let x_squared = x_fast * x_fast;
        let y_squared = y_fast * y_fast;
        if x_squared + y_squared > 4.0 {
            return Some(iterations)
        }
        let difference_of_squares = x_squared - y_squared;
        let double_the_product = 2.0 * x_fast * y_fast;
        (x_fast, y_fast) = (difference_of_squares + x0, double_the_product + y0);
        iterations += 1;

        let x_squared = x_slow * x_slow;
        let y_squared = y_slow * y_slow;
        let difference_of_squares = x_squared - y_squared;
        let double_the_product = 2.0 * x_slow * y_slow;
        (x_slow, y_slow) = (difference_of_squares + x0, double_the_product + y0);

        if threshold == 0.0 {  // (if no threshold was specified)
            if (x_fast, y_fast) == (x_slow, y_slow) {
                return None
            }
        } else {  // (the threshold was specified)
            if (x_fast - x_slow).abs() <= threshold && (y_fast - y_slow).abs() <= threshold {
                return None
            }
        }

        // Remove (or comment-out) the following "continue;" line
        // to allow a "time-out" if calculation gets too long:
        continue;
        #[allow(unreachable_code)]
        if iterations % 1_000_000 == 0 {
            if _start_of_loop.elapsed().as_millis() >= 1_000 {
                return None  // (Taking so much time, we'll assume it's part of the set.)
            }
        }
    }
}


// This structure contains information about the viewport
// (that is, the cartesian coordinate bounds and spans).
// It also contains the physical (width, height) of the
// window (in pixels) and the zoom_level.
//
// With the exception of width and height (in pixels)
// and the zoom_level, everything is a floating point
// number, as they refer to the mathematical measurements
// of the fractal itself.
#[allow(dead_code)]  // (There are some fields that aren't read, but might be in the future.)
#[derive(Debug)]
struct WindowAndViewportInfo {
    width: usize,  // (in pixels)
    height: usize,  // (in pixels)
    center_x: f64,
    center_y: f64,
    span: f64,
    distance_from_center_to_edge: f64,  // (half of the span)
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    delta_x: f64,
    delta_y: f64,
    zoom_level: isize,
}
impl WindowAndViewportInfo {
    fn new(width: usize, height: usize,  // (in pixels)
           center_x: f64, center_y: f64, distance_from_center_to_edge: f64,
           zoom_level: isize)
               -> WindowAndViewportInfo {

        let span = distance_from_center_to_edge * 2.0;
        let min_x = center_x - distance_from_center_to_edge;
        let max_x = center_x + distance_from_center_to_edge;
        let min_y = center_y - distance_from_center_to_edge;
        let max_y = center_y + distance_from_center_to_edge;
        let delta_x = (max_x - min_x) / width as f64;
        let delta_y = (max_y - min_y) / height as f64;

        WindowAndViewportInfo {
            width,
            height,
            center_x,
            center_y,
            span,
            distance_from_center_to_edge,
            min_x,
            max_x,
            min_y,
            max_y,
            delta_x,
            delta_y,
            zoom_level,
        }
    }
}


// The reason for the existence of this MouseInfo struct
// is because the minifb::Window class does not have a
// way to detect if a mouse button was JUST pressed or
// JUST released.  (It can detect that a mouse button
// is currently pressed down, but not if it was just
// pressed down.)
//
// So this class provides methods to read in the current
// button states, and it also provides methods that
// use logic to tell you if the buttons were just
// down or just released.
//
// To reiterate:  This struct will become obsolete once
// the minifb::Window class gets .just_released() methods
// for mouse buttons.
#[derive(Debug)]
struct MouseInfo {
    left_mouse_button_pressed: [bool; 2],
    right_mouse_button_pressed: [bool; 2],
}
#[allow(dead_code)]  // (There are methods that aren't called here, but may be in the future.)
impl MouseInfo {
    fn new() -> MouseInfo {
        MouseInfo {
            left_mouse_button_pressed: [false, false],
            right_mouse_button_pressed: [false, false],
        }
    }

    fn set_mouse_buttons_pressed(&mut self, left_mouse_button_pressed: bool,
                                            right_mouse_button_pressed: bool) {
        self.set_left_mouse_button_pressed(left_mouse_button_pressed);
        self.set_right_mouse_button_pressed(right_mouse_button_pressed);
    }

    fn set_left_mouse_button_pressed(&mut self, value: bool) {
        self.left_mouse_button_pressed[0] = self.left_mouse_button_pressed[1];
        self.left_mouse_button_pressed[1] = value;
    }

    fn set_right_mouse_button_pressed(&mut self, value: bool) {
        self.right_mouse_button_pressed[0] = self.right_mouse_button_pressed[1];
        self.right_mouse_button_pressed[1] = value;
    }

    fn left_mouse_button_currently_pressed(&self) -> bool {
        self.left_mouse_button_pressed[1]
    }

    fn right_mouse_button_currently_pressed(&self) -> bool {
        self.right_mouse_button_pressed[1]
    }

    fn left_mouse_button_just_pressed(&self) -> bool {
        !self.left_mouse_button_pressed[0] && self.left_mouse_button_pressed[1]
    }

    fn right_mouse_button_just_pressed(&self) -> bool {
        !self.right_mouse_button_pressed[0] && self.right_mouse_button_pressed[1]
    }

    fn left_mouse_button_just_released(&self) -> bool {
        self.left_mouse_button_pressed[0] && !self.left_mouse_button_pressed[1]
    }

    fn right_mouse_button_just_released(&self) -> bool {
        self.right_mouse_button_pressed[0] && !self.right_mouse_button_pressed[1]
    }
}


fn convert_row_and_column_to_x_and_y(info: &WindowAndViewportInfo,
                                     row: isize, column: isize) -> (f64, f64) {
    let x = info.min_x + info.delta_x * (column as f64 + 0.5);
    let y = info.max_y - info.delta_y * (row as f64 + 0.5);
    (x, y)
}


fn save_screenshot(filename: &str, width: usize, height: usize, image_buffer: &Vec<u32>) -> () {
    assert!(image_buffer.len() == width * height);

    let mut screenshot_buffer = image::ImageBuffer::new(width as u32, height as u32);

    for (x, y, pixel) in screenshot_buffer.enumerate_pixels_mut() {
        let (x, y): (usize, usize) = (x as usize, y as usize);  // (Convert from u32 to usize.)
        let i: usize = y * width + x;
        let image_buffer_pixel = image_buffer[i] & 0xff_ff_ff;

        let r = image_buffer_pixel >> 16 & 0xff;
        let g = image_buffer_pixel >>  8 & 0xff;
        let b = image_buffer_pixel >>  0 & 0xff;

        *pixel = image::Rgb([r as u8, g as u8, b as u8]);
    }

    screenshot_buffer.save(filename).unwrap();
    println!("Saved screenshot to a file named:  {filename}");
}


// This enum reflects the user's choices.
enum UserInput {
    Nothing,
    Quit,
    SaveScreenShot,
    ZoomIn(f64, f64),  // (x, y) of the new center.  (Where the user clicked.)
    ZoomOut(f64, f64),  // (x, y) of the new center.  (NOT where the user clicked!)
}


// Based on the Window and WindowAndViewportInfo,
// this checks to see if the user gave any input.
fn get_user_input(window: &minifb::Window,
                  info: &WindowAndViewportInfo,
                  mouse_info: &mut MouseInfo) -> UserInput {

    mouse_info.set_mouse_buttons_pressed(
                window.get_mouse_down(minifb::MouseButton::Left),
                window.get_mouse_down(minifb::MouseButton::Right));

    if !window.is_open() || window.is_key_down(minifb::Key::Escape)
                         || window.is_key_down(minifb::Key::Q) {
        return UserInput::Quit
    } else if window.is_key_released(minifb::Key::S) {
        return UserInput::SaveScreenShot
    } else if mouse_info.left_mouse_button_just_released() {  // (Left mouse button WAS down, but no longer.)
        let (column, row) = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap();
        let row = row as isize;  // (Convert to integer.)
        let column = column as isize;  // (Convert to integer.)
        let (x, y) = convert_row_and_column_to_x_and_y(&info, row, column);
        return UserInput::ZoomIn(x, y)
    } else if mouse_info.right_mouse_button_just_released() {  // (Right mouse button WAS down, but no longer.)
        let (column, row) = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap();
        let row = row as isize;  // (Convert to integer.)
        let column = column as isize;  // (Convert to integer.)
        let (x, y) = convert_row_and_column_to_x_and_y(&info, row, column);
        return UserInput::ZoomOut(2.0 * info.center_x - x, 2.0 * info.center_y - y)
    }

    return UserInput::Nothing
}


#[allow(dead_code)]
fn test_color_function() {
    let n = 0;   println!("{}: {:?}", n, color(Some(n)));
    let n = 15;  println!("{}: {:?}", n, color(Some(n)));
    let n = 30;  println!("{}: {:?}", n, color(Some(n)));
    let n = 45;  println!("{}: {:?}", n, color(Some(n)));
    let n = 60;  println!("{}: {:?}", n, color(Some(n)));
    let n = 75;  println!("{}: {:?}", n, color(Some(n)));
    let n = 90;  println!("{}: {:?}", n, color(Some(n)));
    let n = None;  println!("{:?}: {:?}", n, color(n));
}


// Returns the help text suitable for printing when the
// user specifies the --help switch.
fn help_text() -> String {
    format!("

Program:  JLR-Mandelbrot
          A Mandelbrot set viewer.

Usage:  jlr-mandelbrot [--help] [--size=WIDTH_IN_PIXELS]
Example usages:
   jlr-mandelbrot
   jlr-mandelbrot --size 256

Options:
   -h, --help
      Shows this help text and exits.
   --size=NUMBER
      Displays the image in a square window of NUMBER by NUMBER pixels.
      ({default_size} is the default.)

Once the image is displayed:
    A left-click of the mouse zooms in.
    A right-click of the mouse zooms out.
    Pressing the S key will save a screenshot in PNG format.
    Pressing the Q key will quit.
    Pressing the Escape key will also quit.

Author:  Jean-Luc Romano
e-mail:  {username}@{domain}.{suffix}

", default_size = DEFAULT_WINDOW_SIZE,
   username = "jl_post", domain = "hotmail", suffix = "com")
}


#[allow(dead_code)]
fn test_escape_value_function() {
    let (x, y) = (0.0, 0.0);  println!("{:?}: {:?}", (x, y), escape_value(x, y, None));
    let (x, y) = (-1., 0.4);  println!("{:?}: {:?}", (x, y), escape_value(x, y, None));
    let (x, y) = (0.25, 0.5);  println!("{:?}: {:?}", (x, y), escape_value(x, y, Some(0.001)));
    let (x, y) = (-1., 0.25);  println!("{:?}: {:?}", (x, y), escape_value(x, y, Some(0.001)));
    let (x, y) = (-1., -0.25);  println!("{:?}: {:?}", (x, y), escape_value(x, y, Some(0.001)));
    let (x, y) = (0.25, -0.5);  println!("{:?}: {:?}", (x, y), escape_value(x, y, Some(0.001)));
}


#[allow(dead_code)]
fn test_next_pixel_coordinate_function() {
    let (mut row, mut column) = (0, 0);  println!("{:?}", (column, row));
    for _ in 0..50 {
        (row, column) = next_pixel_coordinate(row, column);  println!("{:?}", (column, row));
    }
}


#[allow(dead_code)]
fn test_all() {
    println!();
    test_color_function();
    println!();
    test_escape_value_function();
    println!();
    test_next_pixel_coordinate_function();
    println!();
}


fn main() {
    // test_all();  // (Uncomment to do testing.)

    let mut window_size_to_use: usize = DEFAULT_WINDOW_SIZE;

    // Parse command-line arguments:
    {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut still_looking_for_options = true;
        for arg in args {
            if still_looking_for_options && arg == "--" {  // (The "--" option signifies the end of the options.)
                still_looking_for_options = false;
            } else if still_looking_for_options && (arg == "-h" || arg == "--help") {
                println!("{}", help_text());
                return ()
            } else if still_looking_for_options && arg.starts_with("--size=") {
                let size_text = &arg[7..];
                window_size_to_use = match size_text.parse() {
                    Ok(size) => size,
                    _ => {
                        println!("Error:  {arg} has an invalid value of \"{size_text}\".");
                        std::process::exit(1)
                    }
                };
                if window_size_to_use == 0 {
                    println!("Error:  The SIZE in --size=SIZE must be more than zero.");
                    std::process::exit(1)
                }
            } else if still_looking_for_options && arg == "--size" {
                println!("Error:  The --size=SIZE argument seems to be missing the \"=SIZE\" part.");
                println!("        (Did you forget the \"=\" sign?)");
                std::process::exit(1)
            } else if still_looking_for_options && arg.starts_with("--") {
                println!("Error:  Invalid option:  {arg}");
                std::process::exit(1)
            } else {
                println!("Error:  Invalid argument:  {arg}");
                std::process::exit(1)
            }
        }
    }  // (End of parsing command-line arguments.)

    println!();
    println!();
    println!("Welcome to JLR-Mandelbrot!");
    println!();
    println!("---=== A Mandelbrot set viewer ===---");
    println!();
    println!("Programmed in the Rust programming language by Jean-Luc Romano.");
    println!("(Programming work was started on Thursday, November 17, 2022.)");
    println!("Contact info:  {}@{}.{}", "jl_post", "hotmail", "com");
    println!();
    println!();
    println!("Instructions:");
    println!();
    println!(" * Left-click to zoom in.");
    println!(" * Right-click to zoom out.");
    println!(" * Press S to save a screenshot.");
    println!(" * Press the Q key or the Escape key to quit/exit the program.");
    println!();
    println!("For additional help, run this program with the --help switch.");
    println!();
    println!();

    let (width, height) = (window_size_to_use, window_size_to_use);

    let mut window = minifb::Window::new(
        "The Mandelbrot Set",
        width,
        height,
        minifb::WindowOptions::default()
    ).expect("Unable to create window.");

    // Limit to max ~60 fps update rate:
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    // window.limit_update_rate(Some(std::time::Duration::from_micros(0)));
    window.limit_update_rate(None);

    let mut image_buffer: Vec<u32> = vec![0u32; width * height];

    // let (min_x, max_x) = (-2.0, 2.0);
    // let (min_y, max_y) = (-2.0, 2.0);
    let (original_center_x, original_center_y) = (-0.5, 0.0);
    let original_distance_from_center_to_edge: f64 = 1.725;

    let mut info = WindowAndViewportInfo::new(
        width, height,  // (in pixels)
        original_center_x, original_center_y,
        original_distance_from_center_to_edge,
        0);
    let mut mouse_info = MouseInfo::new();

    let mut done = false;
    window.update_with_buffer(&image_buffer, info.width, info.height).unwrap();
    let mut user_input = get_user_input(&window, &info, &mut mouse_info);

    'main_event_loop:
    loop {
        match user_input {
            UserInput::Quit => break 'main_event_loop,
            UserInput::ZoomIn(x, y) => {
                info = WindowAndViewportInfo::new(
                    info.width, info.height,
                    x, y, info.distance_from_center_to_edge / 2.0,
                    info.zoom_level + 1);
                done = false;  // Let the drawing begin again!
                user_input = UserInput::Nothing;
                continue 'main_event_loop
            }
            UserInput::ZoomOut(x, y) => {
                info = WindowAndViewportInfo::new(
                    info.width, info.height,
                    x, y, info.distance_from_center_to_edge * 2.0,
                    info.zoom_level - 1);
                done = false;  // Let the drawing begin again!
                user_input = UserInput::Nothing;
                continue 'main_event_loop
            }
            UserInput::SaveScreenShot => {
                let now = chrono::Utc::now();
                let filename = now.format("jlr-mandelbrot.screenshot.%Y%m%d.%H%M%S.%3f.png").to_string();
                save_screenshot(&filename, info.width, info.height, &image_buffer)
            }
            _ => ()
        }

        if done {
            // Limit to max ~60 fps update rate:
            window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
            window.update_with_buffer(&image_buffer, info.width, info.height).unwrap();
            user_input = get_user_input(&window, &info, &mut mouse_info);
            continue;
        } else {
            window.limit_update_rate(None);
        }


        // If we get here, then we're generating a fractal image!

        let threshold = info.delta_x / 4.0;

        window.limit_update_rate(None);

        let start_time = std::time::Instant::now();
        let mut last_update_time = std::time::Instant::now();

        let (half_width, half_height) = (info.width / 2, info.height / 2);  // (in pixels)
        let (mut current_row, mut current_column) = (half_width as isize, half_height as isize);  // (in pixels)

        for num_pixel in 0..(info.width * info.height) {
            if num_pixel == 0 {
                (current_row, current_column) = (half_width as isize, half_height as isize);
            } else {
                loop {
                    (current_row, current_column)
                        = next_pixel_coordinate_with_offset(current_row, current_column,
                                                            half_width, half_height);
                    if current_row < 0 {
                        continue  // (Out of bounds, so try again.)
                    } else if current_column < 0 {
                        continue  // (Out of bounds, so try again.)
                    } else if current_row >= info.height as isize {
                        continue  // (Out of bounds, so try again.)
                    } else if  current_column >= info.width as isize {
                        continue  // (Out of bounds, so try again.)
                    } else {
                        break  // (Success!  We can keep this value.)
                    }
                }
            }
            let (row, column) = (current_row as usize, current_column as usize);
            // Convert row & column into x & y:
            let (x, y) = convert_row_and_column_to_x_and_y(&info, row as isize, column as isize);

            let pixel_value = escape_value(x, y, Some(threshold));
            let (r, g, b) = color(pixel_value);
            let color_as_integer = rgb_to_u32(r, g, b);

            let i = row * info.width + column;
            image_buffer[i] = color_as_integer;

            if last_update_time.elapsed().as_millis() >= 1 {
                window.update_with_buffer(&image_buffer, info.width, info.height).unwrap();
                last_update_time = std::time::Instant::now();
                user_input = get_user_input(&window, &info, &mut mouse_info);

                match user_input {
                    UserInput::Quit => break 'main_event_loop,
                    UserInput::Nothing => (),
                    _ => continue 'main_event_loop  // (Handled at the top of the loop.)
                }
            }
        }
        done = true;
        println!("Zoom level {}:  Elapsed time:  {} sec.",
                 info.zoom_level,
                 start_time.elapsed().as_micros() as f64 / 1e6);
    }  // (End of 'main_event_loop.)
}


