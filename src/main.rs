

// Program:  JLR-Mandelbrot
// Author:  Jean-Luc Romano
// Date started:  Thursday, November 17, 2022


// ----------
// Revisions:
// ----------
// 2022-11-17:  Initial version.  (No zooming capabilities.)
// 2022-11-18:  Added threshold to calculate_escape_value() function.
// 2022-11-18:  Added ability to zoom in and out.
// 2022-11-19:  Drawing the set now happens from the center -> out,
//              instead of from top to bottom.
// 2022-11-21:  Added ability to save a screenshot of current image.
// 2022-11-23:  Added the ability to specify window size with --size=NUMBER.
// 2022-11-28:  Added the --bailout=NUMBER switch.
// 2022-11-30:  Added printing of coordinates (to stdout) with the C key.
// 2022-12-01:  Added the --julia=X,Y switch.
// ----------


/*///////////////////////////////////////////////////////////////////
NOTE:  This program uses several crates, so to compile
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


// The Float type defines the type of floating-point values
// to use when calculating the fractal.  It should really
// be set to the biggest float type available (which is
// f64 today, but might be f128 tomorrow).  But if you're
// curious, you can change it to f32 for comparison purposes.
type Float = f64;


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
        return MANDELBROT_SET_COLOR
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

    let value1 = value1.try_into().unwrap();  // (Converts from usize to u8.)
    let value2 = value2.try_into().unwrap();  // (Converts from usize to u8.)

    let leg = i / NUM_COLORS_PER_LEG;
    match leg {
        0 => (value1, value2, 0),
        1 => (0, value1, value2),
        2 => (value2, 0, value1),
        // Should never get here, but include just in case:
        _ => panic!("Reached state that should never have been reached."),
    }
}


// A convenience function to turn RBG values
// (from 0 to 255, inclusive) into a u32 integer.
fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}


// This structure is an iterator that returns pixel coordinates
// (row, column) starting at the specified (start_row, start_column)
// and continuing outward in a swirl.  Its iterator should never
// return None.
struct RowAndColumnIterator {
    started: bool,
    start_row: isize,
    start_column: isize,
    row: isize,
    column: isize,
}
impl RowAndColumnIterator {
    fn new(start_row: isize, start_column: isize) -> Self {
        Self {
            started: false,
            start_row: start_row,
            start_column: start_column,
            row: 0,
            column: 0,
        }
    }
}
impl Iterator for RowAndColumnIterator {
    // We will be returning (row, column) tuples (wrappen in Some()):
    type Item = (isize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            (self.row, self.column) = (0, 0);
            self.started = true;
        } else {
            let (row, column) = (&mut self.row, &mut self.column);
            // The variables row and column are now references
            // to self.row and self.column.

            // In case you're wondering, I figured out the following
            // logic by drawing a straight-line spiral on grid paper.
            // No way I could have done it in my head!

            if (*row, *column) == (0, 0) {
                (*row, *column) = (0, 1);  // Go right.
            } else if *row == *column {
                if *column > 0 {
                    (*row, *column) = (*row, *column + 1);  // Go right.
                } else {
                    (*row, *column) = (*row + 1, *column);  // Go down.
                }
            } else if *row == -*column {
                if *column > 0 {
                    (*row, *column) = (*row, *column - 1);  // Go left.
                } else {
                    (*row, *column) = (*row, *column + 1);  // Go right.
                }
            } else if *column > 0 && *column > row.abs() {
                (*row, *column) = (*row - 1, *column);  // Go up.
            } else if *column < 0 && -*column > row.abs() {
                (*row, *column) = (*row + 1, *column);  // Go down.
            } else if *row > 0 && *row > column.abs() {
                (*row, *column) = (*row, *column + 1);  // Go right.
            } else if *row < 0 && -*row > column.abs() {
                (*row, *column) = (*row, *column - 1);  // Go left.
            } else {
                panic!("Reached state that should never have been reached.");
            }
        }

        Some((self.row + self.start_row, self.column + self.start_column))
    }
}



// The main Mandelbrot set calculation function.
// Given an (x, y) coordinate, it will return the number
// of iterations needed to determine that the coordinate
// is not part of the Mandelbrot set (or None if it is
// part of the set).
//
// Note:  Using a value of None for c will make it be
//        set to the passed-in (x,y), which is ideal
//        for calculating the Mandelbrot set.
//        For Julia sets, where c is the same for
//        each and every coordinate, c can be passed
//        in as Some((some_x as Float, some_y as Float)).
//
// The threshold specifies what's considered "close enough"
// for x and y values when detecting cycles.  (Half the
// length of a pixel is probably good enough.)
//
// The bailout value is the maximum number of times
// Znext = Z + c
// gets carried out (not counting the times for
// cycle detection).
fn calculate_escape_value(x: Float, y: Float,
                          c: Option<(Float, Float)>,
                          threshold: Option<Float>,
                          bailout: Option<usize>) -> Option<usize> {
    let (c_x, c_y) = c.unwrap_or((x, y));
    let threshold = threshold.unwrap_or(0.0);

    let mut iterations = 0;
    let (mut x_slow, mut y_slow) = (x, y);
    let (mut x_fast, mut y_fast) = (x, y);

    let _start_of_loop = std::time::Instant::now();

    loop {
        let (x_squared, y_squared) = (x_fast * x_fast, y_fast * y_fast);
        if x_squared + y_squared > 4.0 {
            return Some(iterations)
        }
        let difference_of_squares = x_squared - y_squared;
        let double_the_product = 2.0 * x_fast * y_fast;
        (x_fast, y_fast) = (difference_of_squares + c_x, double_the_product + c_y);
        // Check to see if we've encountered this point before:
        if threshold == 0.0 {  // (if no threshold was specified)
            if (x_fast, y_fast) == (x_slow, y_slow) {
                return None
            }
        } else {  // (the threshold was specified)
            if (x_fast - x_slow).abs() <= threshold && (y_fast - y_slow).abs() <= threshold {
                return None
            }
        }
        iterations += 1;
        if let Some(bailout_to_use) = bailout {
            if iterations == bailout_to_use {
                return None
            }
        }

        let (x_squared, y_squared) = (x_fast * x_fast, y_fast * y_fast);
        if x_squared + y_squared > 4.0 {
            return Some(iterations)
        }
        let difference_of_squares = x_squared - y_squared;
        let double_the_product = 2.0 * x_fast * y_fast;
        (x_fast, y_fast) = (difference_of_squares + c_x, double_the_product + c_y);
        // Check to see if we've encountered this point before:
        if threshold == 0.0 {  // (if no threshold was specified)
            if (x_fast, y_fast) == (x_slow, y_slow) {
                return None
            }
        } else {  // (the threshold was specified)
            if (x_fast - x_slow).abs() <= threshold && (y_fast - y_slow).abs() <= threshold {
                return None
            }
        }
        iterations += 1;
        if let Some(bailout_to_use) = bailout {
            if iterations == bailout_to_use {
                return None
            }
        }

        let (x_squared, y_squared) = (x_slow * x_slow, y_slow * y_slow);
        let difference_of_squares = x_squared - y_squared;
        let double_the_product = 2.0 * x_slow * y_slow;
        (x_slow, y_slow) = (difference_of_squares + c_x, double_the_product + c_y);
        // Check to see if we've encountered this point before:
        if threshold == 0.0 {  // (if no threshold was specified)
            if (x_fast, y_fast) == (x_slow, y_slow) {
                return None
            }
        } else {  // (the threshold was specified)
            if (x_fast - x_slow).abs() <= threshold && (y_fast - y_slow).abs() <= threshold {
                return None
            }
        }
        // Do not increment the iterations variable here,
        // as we only do so after advancing the "fast" point cycle.

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
    center_x: Float,
    center_y: Float,
    span: Float,
    distance_from_center_to_edge: Float,  // (half of the span)
    min_x: Float,
    max_x: Float,
    min_y: Float,
    max_y: Float,
    delta_x: Float,
    delta_y: Float,
    zoom_level: isize,
}
impl WindowAndViewportInfo {
    fn new(width: usize, height: usize,  // (in pixels)
           center_x: Float, center_y: Float, distance_from_center_to_edge: Float,
           zoom_level: isize)
               -> Self {

        let span = distance_from_center_to_edge * 2.0;
        let min_x = center_x - distance_from_center_to_edge;
        let max_x = center_x + distance_from_center_to_edge;
        let min_y = center_y - distance_from_center_to_edge;
        let max_y = center_y + distance_from_center_to_edge;
        let delta_x = (max_x - min_x) / width as Float;
        let delta_y = (max_y - min_y) / height as Float;

        Self {
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
    fn new() -> Self {
        Self {
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


// Converts a row&column coordinate (with row=0 & column=0 as the center
// of upper-right pixel) to the Mandelbrot's domain's x,y coordinate:
fn convert_row_and_column_to_x_and_y(info: &WindowAndViewportInfo,
                                     row: Float, column: Float) -> (Float, Float) {
    let x = info.min_x + info.delta_x * (column + 0.5);
    let y = info.max_y - info.delta_y * (row + 0.5);
    (x, y)
}


// Saves a screenshot to disk with the given filename.
// (The image_buffer must have a length of width x height.)
fn save_screenshot_to_filename(image_buffer: &Vec<u32>, width: usize, height: usize, filename: &str) -> () {
    // Verify that the length of the image_buffer
    // equals the width x height.  Otherwise, things
    // will break spectacularly:
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


// Saves a screenshot to disk with a calculated filename.
// (The image_buffer must have a length of width x height.)
fn save_screenshot(image_buffer: &Vec<u32>, width: usize, height: usize) -> () {
    let now = chrono::Utc::now();
    let filename = now.format("jlr-mandelbrot.screenshot.%Y%m%d.%H%M%S.%3f.png").to_string();
    save_screenshot_to_filename(&image_buffer, width, height, &filename)
}


// Prints screen coordinates and mouse coordinates to the console.
fn print_coordinates(window: &minifb::Window, info: &WindowAndViewportInfo) {
    let upper_left = (info.min_x, info.max_y);
    let upper_right = (info.max_x, info.max_y);
    let center = (info.center_x, info.center_y);
    let lower_left = (info.min_x, info.min_y);
    let lower_right = (info.max_x, info.min_y);
    let (mouse_column, mouse_row) = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap();
    let mouse_cursor = convert_row_and_column_to_x_and_y(
                           &info,
                           mouse_row as Float, mouse_column as Float);
    let mouse_cursor = (mouse_cursor.0, mouse_cursor.1);
    // We want to round the numbers to use only a specified
    // number of digits of precision, so that they don't take
    // up too much of the line:
    let round_tuple_of_floats = |p: (Float, Float), decimal_places: isize| -> (Float, Float) {
        let p0 = p.0 * (10.0 as Float).powi(decimal_places as i32);
        let p0 = p0.round();
        let p0 = p0 / (10.0 as Float).powi(decimal_places as i32);
        let p1 = p.1 * (10.0 as Float).powi(decimal_places as i32);
        let p1 = p1.round();
        let p1 = p1 / (10.0 as Float).powi(decimal_places as i32);
        (p0, p1)
    };
    let decimal_places = 7;
    print!("Screen coordinates:
--------------------------------------------------------------
|{: <29}  {: >29}|
|{: ^60}|
|{: <29}  {: >29}|
--------------------------------------------------------------
Mouse coordinates:  {}
",
    format!("{:?}", round_tuple_of_floats(upper_left, decimal_places)),
    format!("{:?}", round_tuple_of_floats(upper_right, decimal_places)),
    format!("{:?}", round_tuple_of_floats(center, decimal_places)),
    format!("{:?}", round_tuple_of_floats(lower_left, decimal_places)),
    format!("{:?}", round_tuple_of_floats(lower_right, decimal_places)),
    format!("{:?}", round_tuple_of_floats(mouse_cursor, decimal_places)));
}


// This enum reflects the user's choices.
enum UserInput {
    Nothing,
    Quit,
    SaveScreenShot,
    ShowCoordinates,
    ZoomIn(Float, Float),  // (x, y) of the new center.  (Where the user clicked.)
    ZoomOut(Float, Float),  // (x, y) of the new center.  (NOT where the user clicked!)
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
    } else if window.is_key_released(minifb::Key::S) {  // S => Save ScreenShot
        return UserInput::SaveScreenShot
    } else if window.is_key_released(minifb::Key::C) {  // C => Coordinates
        return UserInput::ShowCoordinates
    } else if mouse_info.left_mouse_button_just_released() {  // (Left mouse button WAS down, but no longer.)
        let (column, row) = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap();
        let (x, y) = convert_row_and_column_to_x_and_y(&info, row as Float, column as Float);
        return UserInput::ZoomIn(x, y)
    } else if mouse_info.right_mouse_button_just_released() {  // (Right mouse button WAS down, but no longer.)
        let (column, row) = window.get_mouse_pos(minifb::MouseMode::Pass).unwrap();
        let (x, y) = convert_row_and_column_to_x_and_y(&info, row as Float, column as Float);
        return UserInput::ZoomOut(2.0 * info.center_x - x, 2.0 * info.center_y - y)
    }

    return UserInput::Nothing
}


#[allow(dead_code)]
fn test_color_function() {
    println!();
    println!("Testing the color() function:");
    let n = 0;   println!("{}: {:?}", n, color(Some(n)));
    let n = 15;  println!("{}: {:?}", n, color(Some(n)));
    let n = 30;  println!("{}: {:?}", n, color(Some(n)));
    let n = 45;  println!("{}: {:?}", n, color(Some(n)));
    let n = 60;  println!("{}: {:?}", n, color(Some(n)));
    let n = 75;  println!("{}: {:?}", n, color(Some(n)));
    let n = 90;  println!("{}: {:?}", n, color(Some(n)));
    let n = None;  println!("{:?}: {:?}", n, color(n));
    println!();
}


// Returns the help text suitable for printing when the
// user specifies the --help switch.
fn help_text() -> String {
    format!("

Program:  JLR-Mandelbrot
          A Mandelbrot set viewer.

Usage:  jlr-mandelbrot [options]
Example usages:
   jlr-mandelbrot
   jlr-mandelbrot --size=256
   jlr-mandelbrot --bailout=150
   jlr-mandelbrot --julia=-0.835,-0.232

Options:
   -h, --help
      Shows this help text and exits.
   --size=NUMBER
      Displays the image in a square window of NUMBER by NUMBER pixels.
      ({default_size} is the default.)
   --bailout=NUMBER
      Uses a bailout number, or a maximum number of iterations.
      If this number is reached, then a point is considered to
      be part of the set.  (A bailout number is not used by default.)
   --julia=X,Y
      Instead of a Mandelbrot set, a Julia set will be generated
      using X+Yi as the value for c.

Once the image is displayed:
   A left-click of the mouse zooms in.
   A right-click of the mouse zooms out.
   Pressing the C key will print coordinates to the console.
   Pressing the S key will save a screenshot in PNG format.
   Pressing the Q key will quit.
   Pressing the Escape key will also quit.

Author:  Jean-Luc Romano
e-mail:  {username}@{domain}.{suffix}

", default_size = DEFAULT_WINDOW_SIZE,
   username = "jl_post", domain = "hotmail", suffix = "com")
}


#[allow(dead_code)]
fn test_calculate_escape_value_function() {
    println!();
    println!("Testing the calculate_escape_value() function:");
    let (x, y) = (0.0, 0.0);  println!("{:?}: {:?}", (x, y), calculate_escape_value(x, y, None, None, None));
    let (x, y) = (-1., 0.4);  println!("{:?}: {:?}", (x, y), calculate_escape_value(x, y, None, None, None));
    let (x, y) = (0.25, 0.5);  println!("{:?}: {:?}", (x, y), calculate_escape_value(x, y, None, Some(0.001), None));
    let (x, y) = (-1., 0.25);  println!("{:?}: {:?}", (x, y), calculate_escape_value(x, y, None, Some(0.001), None));
    let (x, y) = (-1., -0.25);  println!("{:?}: {:?}", (x, y), calculate_escape_value(x, y, None, Some(0.001), None));
    let (x, y) = (0.25, -0.5);  println!("{:?}: {:?}", (x, y), calculate_escape_value(x, y, None, Some(0.001), None));
    println!();
}


#[allow(dead_code)]
fn test_row_and_column_iterator() {
    println!();
    println!("Testing the RowAndColumnIterator:");
    let mut row_and_column_iterator = RowAndColumnIterator::new(0, 0);
    for _ in 0..50 {
        println!("{:?}", row_and_column_iterator.next().unwrap());
    }
    println!();
}


#[allow(dead_code)]
fn test_all() {
    println!();
    test_color_function();
    println!();
    test_calculate_escape_value_function();
    println!();
    test_row_and_column_iterator();
    println!();
}


fn main() {
    // These are "practically global" variables, in that
    // they're used (and sometimes changed) all throughout
    // the main() function:
    let mut window_size_to_use: usize = DEFAULT_WINDOW_SIZE;
    let mut bailout_value_to_use: Option<usize> = None;
    let mut c: Option<(Float, Float)> = None;  // Sometimes known as (x0, y0).
    let mut original_center_to_use: (Float, Float) = (-0.5, 0.0);
    let original_distance_from_center_to_edge: Float = 1.725;

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
            } else if still_looking_for_options && arg == "--test" {
                // --test is an undocumented option;
                // it is only used for diagnostic purposes.
                test_all();
                return ()
            } else if still_looking_for_options && arg.starts_with("--size=") {
                let prefix_length = "--size=".len();
                let size_text = &arg[prefix_length..];
                window_size_to_use = match size_text.parse() {
                    Ok(size) => size,
                    _ => {
                        println!("Error:  {arg} has an invalid value of \"{size_text}\".");
                        std::process::exit(1)
                    }
                };
                if window_size_to_use == 0 {
                    println!("Error:  The NUMBER in --size=NUMBER must be more than zero.");
                    std::process::exit(1)
                }
            } else if still_looking_for_options && arg == "--size" {
                println!("Error:  The --size=NUMBER argument seems to be missing the \"=NUMBER\" part.");
                println!("        (Did you forget the \"=\" sign?)");
                std::process::exit(1)
            } else if still_looking_for_options && arg.starts_with("--bailout=") {
                let prefix_length = "--bailout=".len();
                let bailout_text = &arg[prefix_length..];
                bailout_value_to_use = match bailout_text.parse() {
                    Ok(size) => Some(size),
                    _ => {
                        println!("Error:  {arg} has an invalid value of \"{bailout_text}\".");
                        std::process::exit(1)
                    }
                };
            } else if still_looking_for_options && arg == "--bailout" {
                println!("Error:  The --bailout=NUMBER argument seems to be missing the \"=NUMBER\" part.");
                println!("        (Did you forget the \"=\" sign?)");
                std::process::exit(1)
            } else if still_looking_for_options && arg.starts_with("--julia=") {
                let prefix_length = "--julia=".len();
                let julia_text = &arg[prefix_length..];
                let text_values: Vec<_> = julia_text.split(",").collect();
                if text_values.len() != 2 {
                    println!("Error:  The X,Y value in --julia=X,Y ({julia_text}) needs exactly one comma.");
                    std::process::exit(1)
                }
                let (x_text, y_text) = (text_values[0], text_values[1]);
                let x_result = x_text.parse::<Float>();
                if x_result.is_err() {
                    println!("Error:  The X value in --julia=X,Y ({julia_text}) is not a valid number.");
                    std::process::exit(1)
                }
                let y_result = y_text.parse::<Float>();
                if y_result.is_err() {
                    println!("Error:  The Y value in --julia=X,Y ({julia_text}) is not a valid number.");
                    std::process::exit(1)
                }
                c = Some((x_result.unwrap(), y_result.unwrap()));
                original_center_to_use = (0.0, 0.0);  // We'll start centered for Julia sets.
            } else if still_looking_for_options && arg == "--julia" {
                println!("Error:  The --julia=X,Y argument seems to be missing the \"=X,Y\" part.");
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
    println!(" * Press C to print coordinates (to this console).");
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

    // Use this to limit to max ~60 fps update rate:
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    // Use this to update with no delay:
    window.limit_update_rate(None);

    let mut image_buffer: Vec<u32> = vec![0u32; width * height];

    let (original_center_x, original_center_y) = original_center_to_use;

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
            UserInput::SaveScreenShot => save_screenshot(&image_buffer, info.width, info.height),
            UserInput::ShowCoordinates => print_coordinates(&window, &info),
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
            _ => ()
        }

        if done {
            // Limit to max ~60 fps update rate:
            window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

            // Refresh the screen and get window inputs:
            window.update_with_buffer(&image_buffer, info.width, info.height).unwrap();

            // Examine the window to determine the user's input:
            user_input = get_user_input(&window, &info, &mut mouse_info);

            continue;  // Since we're done drawing the frame, don't draw it again.
        }


        // If we get here, then we're generating a fractal image!

        let threshold = info.delta_x / 4.0;

        window.limit_update_rate(None);

        let start_time = std::time::Instant::now();
        let mut last_update_time = std::time::Instant::now();

        // Create an iterator that will return pixel coordinates,
        // swirling outward from the center of the window:
        let (half_width, half_height) = (info.width / 2, info.height / 2);  // (in pixels)
        let mut row_and_column_iterator = RowAndColumnIterator::new(half_width as isize,
                                                                    half_height as isize);

        // Fill out every pixel in the image_buffer:
        for _ in 0..(info.width * info.height) {
            // Find the coordinate (as (row, column))
            // of the next pixel to operate on:
            let (row, column): (usize, usize) = loop {
                let (current_row, current_column) = row_and_column_iterator.next().unwrap();
                // Check to see if the (current_row, current_column)
                // pixel coordinate is in the window.  If not, keep
                // looping until we find one that is in the window:
                if current_row < 0 {
                    continue  // (Out of bounds, so try again.)
                } else if current_column < 0 {
                    continue  // (Out of bounds, so try again.)
                } else if current_row >= info.height as isize {
                    continue  // (Out of bounds, so try again.)
                } else if  current_column >= info.width as isize {
                    continue  // (Out of bounds, so try again.)
                } else {  // (Success!  We can keep this value.)
                    break (current_row.try_into().unwrap(), current_column.try_into().unwrap())
                }
            };
            // Convert row & column into x & y:
            let (x, y) = convert_row_and_column_to_x_and_y(&info, row as Float, column as Float);

            // Is (x, y) part of the set?  Let's find out.
            // And whatever the answer, find the color to
            // plot at the pixel's row & column of the
            // image_buffer:
            let escape_value = calculate_escape_value(x, y, c, Some(threshold), bailout_value_to_use);
            let (r, g, b) = color(escape_value);
            let color_as_integer = rgb_to_u32(r, g, b);

            // Set the pixel (at the row & column) of the
            // image_buffer to the color we just calculated:
            let i = row * info.width + column;
            image_buffer[i] = color_as_integer;

            // Periodically refresh the image and get user input:
            if last_update_time.elapsed().as_millis() >= 1 {
                window.update_with_buffer(&image_buffer, info.width, info.height).unwrap();
                last_update_time = std::time::Instant::now();
                user_input = get_user_input(&window, &info, &mut mouse_info);

                match user_input {
                    UserInput::Nothing => (),
                    UserInput::Quit => break 'main_event_loop,
                    UserInput::SaveScreenShot => save_screenshot(&image_buffer, info.width, info.height),
                    UserInput::ShowCoordinates => print_coordinates(&window, &info),
                    _ => continue 'main_event_loop  // (The rest are handled at the top of the loop.)
                }
            }
        }
        done = true;
        println!("Zoom level {}:  Elapsed time:  {} sec.",
                 info.zoom_level,
                 start_time.elapsed().as_micros() as Float / 1e6);
    }  // (End of 'main_event_loop.)
}


