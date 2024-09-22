#![feature(portable_simd)]

use std::simd::{ f32x4, u8x16 };
use image::{ ImageReader, ImageBuffer, RgbImage};
use std::path::Path;

fn rgb_to_yuv_simd(r: &[u8], g: &[u8], b: &[u8], y: &mut [u8], u: &mut [u8], v: &mut [u8])
{
    let chunk_size = f32x4::splat(0.0).len();
    for i in (0..r.len()).step_by(chunk_size)
    {
        let r_vec = f32x4::from_array([r[i] as f32, r[i+1] as f32, r[i+2] as f32, r[i+3] as f32]);
        let g_vec = f32x4::from_array([g[i] as f32, g[i+1] as f32, g[i+2] as f32, g[i+3] as f32]);
        let b_vec = f32x4::from_array([b[i] as f32, b[i+1] as f32, b[i+2] as f32, b[i+3] as f32]);

        let y_vec = f32x4::splat(0.299) * r_vec + f32x4::splat(0.587) * g_vec + f32x4::splat(0.114) * b_vec;
        let u_vec = f32x4::splat(128.0) + f32x4::splat(-0.168736) * r_vec + f32x4::splat(-0.331264) * g_vec + f32x4::splat(0.5) * b_vec;
        let v_vec = f32x4::splat(128.0) + f32x4::splat(0.5) * r_vec + f32x4::splat(-0.418688) * g_vec + f32x4::splat(-0.081312) * b_vec;

        for j in 0..chunk_size {
            y[i + j] = y_vec[j].max(0.0).min(255.0) as u8;
            u[i + j] = u_vec[j].max(0.0).min(255.0) as u8;
            v[i + j] = v_vec[j].max(0.0).min(255.0) as u8;
        }
    }
}

fn yuv_to_rgb_simd(y: &[u8], u: &[u8], v: &[u8], r: &mut [u8], g: &mut [u8], b: &mut [u8])
{
    let chunk_size = f32x4::splat(0.0).len();
    for i in (0..y.len()).step_by(chunk_size)
    {
        let y_vec = f32x4::from_array([y[i] as f32, y[i+1] as f32, y[i+2] as f32, y[i+3] as f32]);
        let u_vec = f32x4::from_array([u[i] as f32, u[i+1] as f32, u[i+2] as f32, u[i+3] as f32]);
        let v_vec = f32x4::from_array([v[i] as f32, v[i+1] as f32, v[i+2] as f32, v[i+3] as f32]);

        let r_vec = y_vec + f32x4::splat(1.402) * (v_vec - f32x4::splat(128.0));
        let g_vec = y_vec - f32x4::splat(0.344136) * (u_vec - f32x4::splat(128.0)) - f32x4::splat(0.714136) * (v_vec - f32x4::splat(128.0));
        let b_vec = y_vec + f32x4::splat(1.772) * (u_vec - f32x4::splat(128.0));

        for j in 0..chunk_size {
            r[i + j] = r_vec[j].max(0.0).min(255.0) as u8;
            g[i + j] = g_vec[j].max(0.0).min(255.0) as u8;
            b[i + j] = b_vec[j].max(0.0).min(255.0) as u8;
        }
    }
}

fn rgb_to_yuv(rgb_img: &RgbImage) -> RgbImage
{
    let (width, height) = rgb_img.dimensions();
    let n = (width * height) as usize;

    let mut r = vec![0u8; n];
    let mut g = vec![0u8; n];
    let mut b = vec![0u8; n];
    let mut y = vec![0u8; n];
    let mut u = vec![0u8; n];
    let mut v = vec![0u8; n];

    for (i, pixel) in rgb_img.pixels().enumerate() {
        r[i] = pixel[0];
        g[i] = pixel[1];
        b[i] = pixel[2];
    }

    rgb_to_yuv_simd(&r, &g, &b, &mut y, &mut u, &mut v);

    let mut yuv_img: RgbImage = ImageBuffer::new(width, height);
    for (i, pixel) in yuv_img.pixels_mut().enumerate() {
        pixel[0] = y[i];
        pixel[1] = u[i];
        pixel[2] = v[i];
    }
    return yuv_img;
}

fn yuv_to_rgb(yuv_img: &RgbImage) -> RgbImage
{
    let (width, height) = yuv_img.dimensions();
    let n = (width * height) as usize;

    let mut y = vec![0u8; n];
    let mut u = vec![0u8; n];
    let mut v = vec![0u8; n];
    let mut r = vec![0u8; n];
    let mut g = vec![0u8; n];
    let mut b = vec![0u8; n];

    for (i, pixel) in yuv_img.pixels().enumerate() {
        y[i] = pixel[0];
        u[i] = pixel[1];
        v[i] = pixel[2];
    }

    yuv_to_rgb_simd(&y, &u, &v, &mut r, &mut g, &mut b);

    let mut rgb_img: RgbImage = ImageBuffer::new(width, height);
    for (i, pixel) in rgb_img.pixels_mut().enumerate() {
        pixel[0] = r[i];
        pixel[1] = g[i];
        pixel[2] = b[i];
    }
    return rgb_img;
}

fn delta_down_res_predictor(yuv_img: &RgbImage, ds_factor: u32) -> (RgbImage, RgbImage)
{
    let (width, height) = yuv_img.dimensions();
    let n = (width * height) as usize;

    let ds_width = width / ds_factor as u32;
    let ds_height = height / ds_factor as u32;
    let ds_n = (ds_width * ds_height) as usize;

    let ds_img = image::imageops::resize(yuv_img, ds_width, ds_height, image::imageops::FilterType::CatmullRom);

    let mut y = vec![0u8; n];
    let mut u = vec![0u8; n];
    let mut v = vec![0u8; n];
    let mut d_y = vec![0u8; n];
    let mut d_u = vec![0u8; n];
    let mut d_v = vec![0u8; n];
    let mut ds_y = vec![0u8; ds_n];
    let mut ds_u = vec![0u8; ds_n];
    let mut ds_v = vec![0u8; ds_n];

    for (i, pixel) in yuv_img.pixels().enumerate() {
        y[i] = pixel[0];
        u[i] = pixel[1];
        v[i] = pixel[2];
    }
    for (i, pixel) in ds_img.pixels().enumerate() {
        ds_y[i] = pixel[0];
        ds_u[i] = pixel[1];
        ds_v[i] = pixel[2];
    }

    let vidx: Vec<u32> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                y * width + x
            })
        })
        .collect();

    let ds_vidx: Vec<u32> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                (y / ds_factor) * ds_width + (x / ds_factor)
            })
        })
        .collect();

    for i in 0..vidx.len() {
        d_y[vidx[i] as usize] = y[vidx[i] as usize] - ds_y[ds_vidx[i] as usize] + 128;
        d_u[vidx[i] as usize] = u[vidx[i] as usize] - ds_u[ds_vidx[i] as usize] + 128;
        d_v[vidx[i] as usize] = v[vidx[i] as usize] - ds_v[ds_vidx[i] as usize] + 128;
    }

    // Hummmmm
    // let chunk_size = u8x16::splat(0).len();

    // for i in (0..y.len()).step_by(chunk_size)
    // {
    //     let y_vec = u8x16::from_array([
    //         y[vidx[i] as usize],
    //         y[vidx[i + 1] as usize],
    //         y[vidx[i + 2] as usize],
    //         y[vidx[i + 3] as usize],
    //         y[vidx[i + 4] as usize],
    //         y[vidx[i + 5] as usize],
    //         y[vidx[i + 6] as usize],
    //         y[vidx[i + 7] as usize],
    //         y[vidx[i + 8] as usize],
    //         y[vidx[i + 9] as usize],
    //         y[vidx[i + 10] as usize],
    //         y[vidx[i + 11] as usize],
    //         y[vidx[i + 12] as usize],
    //         y[vidx[i + 13] as usize],
    //         y[vidx[i + 14] as usize],
    //         y[vidx[i + 15] as usize]]);

    //     let u_vec = u8x16::from_array([
    //         u[vidx[i] as usize],
    //         u[vidx[i + 1] as usize],
    //         u[vidx[i + 2] as usize],
    //         u[vidx[i + 3] as usize],
    //         u[vidx[i + 4] as usize],
    //         u[vidx[i + 5] as usize],
    //         u[vidx[i + 6] as usize],
    //         u[vidx[i + 7] as usize],
    //         u[vidx[i + 8] as usize],
    //         u[vidx[i + 9] as usize],
    //         u[vidx[i + 10] as usize],
    //         u[vidx[i + 11] as usize],
    //         u[vidx[i + 12] as usize],
    //         u[vidx[i + 13] as usize],
    //         u[vidx[i + 14] as usize],
    //         u[vidx[i + 15] as usize]]);

    //     let v_vec = u8x16::from_array([
    //         v[vidx[i] as usize],
    //         v[vidx[i + 1] as usize],
    //         v[vidx[i + 2] as usize],
    //         v[vidx[i + 3] as usize],
    //         v[vidx[i + 4] as usize],
    //         v[vidx[i + 5] as usize],
    //         v[vidx[i + 6] as usize],
    //         v[vidx[i + 7] as usize],
    //         v[vidx[i + 8] as usize],
    //         v[vidx[i + 9] as usize],
    //         v[vidx[i + 10] as usize],
    //         v[vidx[i + 11] as usize],
    //         v[vidx[i + 12] as usize],
    //         v[vidx[i + 13] as usize],
    //         v[vidx[i + 14] as usize],
    //         v[vidx[i + 15] as usize]]);

    //     let ds_y_vec = u8x16::from_array([
    //         ds_y[ds_vidx[i] as usize],
    //         ds_y[ds_vidx[i + 1] as usize],
    //         ds_y[ds_vidx[i + 2] as usize],
    //         ds_y[ds_vidx[i + 3] as usize],
    //         ds_y[ds_vidx[i + 4] as usize],
    //         ds_y[ds_vidx[i + 5] as usize],
    //         ds_y[ds_vidx[i + 6] as usize],
    //         ds_y[ds_vidx[i + 7] as usize],
    //         ds_y[ds_vidx[i + 8] as usize],
    //         ds_y[ds_vidx[i + 9] as usize],
    //         ds_y[ds_vidx[i + 10] as usize],
    //         ds_y[ds_vidx[i + 11] as usize],
    //         ds_y[ds_vidx[i + 12] as usize],
    //         ds_y[ds_vidx[i + 13] as usize],
    //         ds_y[ds_vidx[i + 14] as usize],
    //         ds_y[ds_vidx[i + 15] as usize]]);

    //     let ds_u_vec = u8x16::from_array([
    //         ds_u[ds_vidx[i] as usize],
    //         ds_u[ds_vidx[i + 1] as usize],
    //         ds_u[ds_vidx[i + 2] as usize],
    //         ds_u[ds_vidx[i + 3] as usize],
    //         ds_u[ds_vidx[i + 4] as usize],
    //         ds_u[ds_vidx[i + 5] as usize],
    //         ds_u[ds_vidx[i + 6] as usize],
    //         ds_u[ds_vidx[i + 7] as usize],
    //         ds_u[ds_vidx[i + 8] as usize],
    //         ds_u[ds_vidx[i + 9] as usize],
    //         ds_u[ds_vidx[i + 10] as usize],
    //         ds_u[ds_vidx[i + 11] as usize],
    //         ds_u[ds_vidx[i + 12] as usize],
    //         ds_u[ds_vidx[i + 13] as usize],
    //         ds_u[ds_vidx[i + 14] as usize],
    //         ds_u[ds_vidx[i + 15] as usize]]);

    //     let ds_v_vec = u8x16::from_array([
    //         ds_v[ds_vidx[i] as usize],
    //         ds_v[ds_vidx[i + 1] as usize],
    //         ds_v[ds_vidx[i + 2] as usize],
    //         ds_v[ds_vidx[i + 3] as usize],
    //         ds_v[ds_vidx[i + 4] as usize],
    //         ds_v[ds_vidx[i + 5] as usize],
    //         ds_v[ds_vidx[i + 6] as usize],
    //         ds_v[ds_vidx[i + 7] as usize],
    //         ds_v[ds_vidx[i + 8] as usize],
    //         ds_v[ds_vidx[i + 9] as usize],
    //         ds_v[ds_vidx[i + 10] as usize],
    //         ds_v[ds_vidx[i + 11] as usize],
    //         ds_v[ds_vidx[i + 12] as usize],
    //         ds_v[ds_vidx[i + 13] as usize],
    //         ds_v[ds_vidx[i + 14] as usize],
    //         ds_v[ds_vidx[i + 15] as usize]]);

    //     let d_y_vec = y_vec - ds_y_vec + u8x16::splat(128);
    //     let d_u_vec = u_vec - ds_u_vec + u8x16::splat(128);
    //     let d_v_vec = v_vec - ds_v_vec + u8x16::splat(128);

    //     for j in 0..chunk_size {
    //         d_y[vidx[i + j] as usize] = d_y_vec[j];
    //         d_u[vidx[i + j] as usize] = d_u_vec[j];
    //         d_v[vidx[i + j] as usize] = d_v_vec[j];
    //     }
    // }

    let mut d_img: RgbImage = ImageBuffer::new(width, height);
    for (i, pixel) in d_img.pixels_mut().enumerate() {
        pixel[0] = d_y[i];
        pixel[1] = d_u[i];
        pixel[2] = d_v[i];
    }

    return (d_img, ds_img);
}

fn undelta_down_res_predictor(d_img: &RgbImage, ds_img: &RgbImage, ds_factor: u32) -> RgbImage
{
    let (width, height) = d_img.dimensions();
    let n = (width * height) as usize;

    let ds_width = width / ds_factor as u32;
    let ds_height = height / ds_factor as u32;
    let ds_n = (ds_width * ds_height) as usize;

    let mut d_y = vec![0u8; n];
    let mut d_u = vec![0u8; n];
    let mut d_v = vec![0u8; n];
    let mut ds_y = vec![0u8; ds_n];
    let mut ds_u = vec![0u8; ds_n];
    let mut ds_v = vec![0u8; ds_n];
    let mut y = vec![0u8; n];
    let mut u = vec![0u8; n];
    let mut v = vec![0u8; n];

    for (i, pixel) in d_img.pixels().enumerate() {
        d_y[i] = pixel[0];
        d_u[i] = pixel[1];
        d_v[i] = pixel[2];
    }
    for (i, pixel) in ds_img.pixels().enumerate() {
        ds_y[i] = pixel[0];
        ds_u[i] = pixel[1];
        ds_v[i] = pixel[2];
    }

    let vidx: Vec<u32> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                y * width + x
            })
        })
        .collect();

    let ds_vidx: Vec<u32> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                (y / ds_factor) * ds_width + (x / ds_factor)
            })
        })
        .collect();

    for i in 0..vidx.len() {
        y[vidx[i] as usize] = d_y[vidx[i] as usize] + ds_y[ds_vidx[i] as usize] - 128;
        u[vidx[i] as usize] = d_u[vidx[i] as usize] + ds_u[ds_vidx[i] as usize] - 128;
        v[vidx[i] as usize] = d_v[vidx[i] as usize] + ds_v[ds_vidx[i] as usize] - 128;
    }

    let mut yuv_img: RgbImage = ImageBuffer::new(width, height);
    for (i, pixel) in yuv_img.pixels_mut().enumerate() {
        pixel[0] = y[i];
        pixel[1] = u[i];
        pixel[2] = v[i];
    }

    return yuv_img;
}

fn nephynika(input_path: &str, delta_path: &str, downres_path: &str, ds_factor: u32) {

    let img = ImageReader::open(input_path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image").to_rgb8();

    let yuv_img = rgb_to_yuv(&img);

    let (d_img, ds_img) = delta_down_res_predictor(&yuv_img, ds_factor);

    d_img
        .save(Path::new(delta_path))
        .expect("Failed to save output image");

    ds_img
        .save(Path::new(downres_path))
        .expect("Failed to save output image");

    println!("Saved the processed image to {}", delta_path);
}

fn denephynika(delta_path: &str, downres_path: &str, output_path: &str, ds_factor: u32) {

    let d_img = ImageReader::open(delta_path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image").to_rgb8();

    let ds_img = ImageReader::open(downres_path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image").to_rgb8();

    let yuv_img = undelta_down_res_predictor(&d_img, &ds_img, ds_factor);

    let img = yuv_to_rgb(&yuv_img);

    img
        .save(Path::new(output_path))
        .expect("Failed to save output image");

    println!("Saved the processed image to {}", output_path);
}

fn main() {
    nephynika("vb2n.bmp", "vb2n_delta.bmp", "vb2n_downres.bmp", 4);
    denephynika("vb2n_delta.bmp", "vb2n_downres.bmp", "vb2n_restored.bmp", 4);
}