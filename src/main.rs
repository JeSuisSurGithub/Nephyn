#![feature(portable_simd)]

use std::simd::f32x4;
use image::{ ImageReader, ImageBuffer, RgbImage};
use std::path::Path;

fn rgb_to_ycbcr_simd(r: &[u8], g: &[u8], b: &[u8], y: &mut [u8], cb: &mut [u8], cr: &mut [u8]) {
    assert_eq!(r.len(), g.len());
    assert_eq!(g.len(), b.len());
    assert_eq!(b.len(), y.len());
    assert_eq!(y.len(), cb.len());
    assert_eq!(cb.len(), cr.len());

    let chunk_size = f32x4::splat(0.0).len();
    for i in (0..r.len()).step_by(chunk_size) {
        let r_vec = f32x4::from_array([r[i] as f32, r[i+1] as f32, r[i+2] as f32, r[i+3] as f32]);
        let g_vec = f32x4::from_array([g[i] as f32, g[i+1] as f32, g[i+2] as f32, g[i+3] as f32]);
        let b_vec = f32x4::from_array([b[i] as f32, b[i+1] as f32, b[i+2] as f32, b[i+3] as f32]);

        let y_vec = r_vec * f32x4::splat(0.299) + g_vec * f32x4::splat(0.587) + b_vec * f32x4::splat(0.114);
        let cb_vec = f32x4::splat(128.0) + r_vec * f32x4::splat(-0.168736) + g_vec * f32x4::splat(-0.331264) + b_vec * f32x4::splat(0.5);
        let cr_vec = f32x4::splat(128.0) + r_vec * f32x4::splat(0.5) + g_vec * f32x4::splat(-0.418688) + b_vec * f32x4::splat(-0.081312);

        for j in 0..chunk_size {
            y[i + j] = y_vec[j].max(0.0).min(255.0) as u8;
            cb[i + j] = cb_vec[j].max(0.0).min(255.0) as u8;
            cr[i + j] = cr_vec[j].max(0.0).min(255.0) as u8;
        }
    }
}

fn process_image(input_path: &str, output_path: &str) {
    let img = ImageReader::open(input_path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image")
        .to_rgb8(); // Ensure it's RGB8 (3 bytes per pixel)

    let (width, height) = img.dimensions();
    let num_pixels = (width * height) as usize;

    let mut r = vec![0u8; num_pixels];
    let mut g = vec![0u8; num_pixels];
    let mut b = vec![0u8; num_pixels];

    for (i, pixel) in img.pixels().enumerate() {
        r[i] = pixel[0];
        g[i] = pixel[1];
        b[i] = pixel[2];
    }

    let mut y = vec![0u8; num_pixels];
    let mut cb = vec![0u8; num_pixels];
    let mut cr = vec![0u8; num_pixels];

    rgb_to_ycbcr_simd(&r, &g, &b, &mut y, &mut cb, &mut cr);

    let mut img_out: RgbImage = ImageBuffer::new(width, height);
    for (i, pixel) in img_out.pixels_mut().enumerate() {
        pixel[0] = y[i];
        pixel[1] = cb[i];
        pixel[2] = cr[i];
    }

    // Step 5: Save the resulting image
    img_out
        .save(Path::new(output_path))
        .expect("Failed to save output image");

    println!("Saved the processed image to {}", output_path);
}

fn main() {
    // Input and output file paths
    let input_path = "chao.png";
    let output_path = "chao_ycbcr.png";

    // Process the image: load, convert, and save
    process_image(input_path, output_path);
}