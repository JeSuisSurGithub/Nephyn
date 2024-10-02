#![feature(portable_simd)]

mod ddrp;
mod lzw;
mod yuv;

use crate::ddrp::{delta_down_res_predictor, dedelta_down_res_predictor};
use crate::lzw::{lzw_decode, lzw_encode};
use crate::yuv::{rgb_to_yuv, yuv_to_rgb};

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;
use image::ImageReader;

use tempfile::NamedTempFile;

fn nephynika(input_path: &str, delta_path: &str, downres_path: &str, ds_factor: u32) {

    let img = ImageReader::open(input_path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image").to_rgb8();

    // let yuv_img = rgb_to_yuv(&img);

    let (d_img, ds_img) = delta_down_res_predictor(&img, ds_factor);

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

    let yuv_img = dedelta_down_res_predictor(&d_img, &ds_img, ds_factor);

    // let img = yuv_to_rgb(&yuv_img);

    yuv_img
        .save(Path::new(output_path))
        .expect("Failed to save output image");

    println!("Saved the processed image to {}", output_path);
}

fn main()
{
    nephynika("experiments/sirin_020.bmp", "experiments/delta2.bmp", "experiments/dr2.bmp", 4);
    denephynika("experiments/delta2.bmp", "experiments/dr2.bmp", "experiments/restore2.bmp", 4);
    // {
    //     let start = Instant::now();

    //     let infile = File::open("experiments/sirin_delta.bmp").unwrap();
    //     let mut outfile = File::create("experiments/sirin_delta.lzw").unwrap();
    //     lzw_encode(&mut infile.take(u64::MAX), &mut outfile).unwrap();

    //     let duration = start.elapsed();
    //     println!("Time taken: {:?}", duration);
    // }
    // {
    //     let start = Instant::now();

    //     let infile = File::open("experiments/sirin_delta.lzw").unwrap();
    //     let mut outfile = File::create("experiments/sirin_delta_restored.bmp").unwrap();

    //     lzw_decode(&mut infile.take(u64::MAX), &mut outfile).unwrap();

    //     let duration = start.elapsed();
    //     println!("Time taken: {:?}", duration);
    // }
}