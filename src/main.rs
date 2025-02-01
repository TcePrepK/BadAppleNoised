use std::fs;
use std::path::Path;
use std::process::Command;
use image::{DynamicImage, GenericImageView, Rgb, RgbImage};
use rand::{Rng, SeedableRng};
use rand::rngs::{StdRng};

const INPUT_VIDEO: &str = "BadApple.mp4";
const FRAMES_DIR: &str = "frames";
const MODIFIED_FRAMES_DIR: &str = "modified_frames";
const OUTPUT_VIDEO: &str = "output.mp4";

fn main() {
    // Delete previous output
    fs::remove_file(OUTPUT_VIDEO).unwrap_or_default();

    // Optionally, clean up the frames directory
    fs::remove_dir_all(FRAMES_DIR).unwrap_or_default();
    fs::remove_dir_all(MODIFIED_FRAMES_DIR).unwrap_or_default();
    println!("Cleaned up frames directory.");

    // Create the directory for the frames
    fs::create_dir_all(FRAMES_DIR).unwrap();
    fs::create_dir_all(MODIFIED_FRAMES_DIR).unwrap();

    // Step 1: Extract frames from the original video using ffmpeg
    println!("Extracting frames from the original video...");
    let ffmpeg_extract = Command::new("ffmpeg")
        .arg("-i")
        .arg(INPUT_VIDEO)
        .arg("-vf")
        .arg("fps=30") // Extract at N frames per second
        .arg("-start_number")
        .arg("0") // Start at frame 0
        .arg("-vsync") // Prevent frame dropping
        .arg("vfr")
        .arg(format!("{}/frame_%04d.png", FRAMES_DIR))
        .output()
        .expect("Failed to execute ffmpeg");

    // Check if ffmpeg succeeded
    if !ffmpeg_extract.status.success() {
        eprintln!("Error extracting frames:");
        eprintln!("{}", String::from_utf8_lossy(&ffmpeg_extract.stderr));
        return;
    }

    // Step 2: Process each frame
    println!("Processing frames...");
    let mut frame_count = 0;
    while Path::new(&format!("{}/frame_{:04}.png", FRAMES_DIR, frame_count)).exists() {
        // Load the frame
        let frame_path = format!("{}/frame_{:04}.png", FRAMES_DIR, frame_count);
        let img = image::open(&frame_path).unwrap();

        // Modify the frame (e.g., apply random colors to each pixel)
        let modified_img = modify_frame(frame_count, &img);

        // Save the modified frame
        let modified_frame_path = format!("{}/frame_{:04}.png", MODIFIED_FRAMES_DIR, frame_count);
        modified_img.save(&modified_frame_path).unwrap();

        frame_count += 1;
    }
    println!("Processed {} frames.", frame_count);

    // Step 3: Compile modified frames into a new video using ffmpeg
    println!("Compiling modified frames into a new video...");
    let ffmpeg_compile = Command::new("ffmpeg")
        .arg("-framerate")
        .arg("30") // Match the original frame rate
        .arg("-i")
        .arg(format!("{}/frame_%04d.png", MODIFIED_FRAMES_DIR))
        .arg("-c:v")
        .arg("libx264")
        .arg("-crf")
        .arg("23")
        .arg("-preset")
        .arg("medium")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(OUTPUT_VIDEO)
        .output()
        .expect("Failed to execute ffmpeg");

    if ffmpeg_compile.status.success() {
        println!("Video successfully created: {}", OUTPUT_VIDEO);
    } else {
        eprintln!("Error creating video:");
        eprintln!("{}", String::from_utf8_lossy(&ffmpeg_compile.stderr));
    }
}

/// Modify a frame (e.g., apply random colors to each pixel)
fn modify_frame(frame: u32, img: &DynamicImage) -> RgbImage {
    let (width, height) = img.dimensions();

    let mut random_white = StdRng::seed_from_u64(0);
    let mut random_black = StdRng::seed_from_u64(12415);

    let mut modified_img = RgbImage::new(width, height);

    for _ in 0..frame {
        for _ in 0..width {
            random_white.random_bool(0.5);
        }
    }

    for (x, y, pixel) in img.to_rgb8().enumerate_pixels() {
        // let white_red = random_white.random_range(0..255);
        // let white_green = random_white.random_range(0..255);
        // let white_blue = random_white.random_range(0..255);
        // let white = Rgb([white_red, white_green, white_blue]);
        //
        // let black_red = random_black.random_range(0..255);
        // let black_green = random_black.random_range(0..255);
        // let black_blue = random_black.random_range(0..255);
        // let black = Rgb([black_red, black_green, black_blue]);

        let white_col = if random_white.random_bool(0.5) { 0 } else { 255 };
        let black_col = if random_black.random_bool(0.5) { 0 } else { 255 };

        let white = Rgb([white_col, white_col, white_col]);
        let black = Rgb([black_col, black_col, black_col]);

        if pixel[0] == 0 {
            modified_img.put_pixel(x, y, black);
        } else {
            modified_img.put_pixel(x, y, white);
        }
    }

    modified_img
}