mod ddrp;
mod lzw;
mod yuv;

use crate::ddrp::{delta_down_res_predictor, dedelta_down_res_predictor};
use crate::lzw::{lzw_decode, lzw_encode};
use crate::yuv::{rgb_to_yuv, yuv_to_rgb};

use std::env;
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use std::time::Instant;
use std::path::Path;
use std::mem;

use image::{ ImageReader, ImageBuffer, RgbImage };

#[repr(C)]
#[derive(Default)]
struct NpnkHeader {
    width: u32,
    height: u32,
    ds_size: u64,
    d_size: u64,
}

fn write_header(file: &mut File, header: &NpnkHeader) -> io::Result<()> {
    let bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            header as *const _ as *const u8,
            mem::size_of::<NpnkHeader>(),
        )
    };
    file.write_all(bytes)?;
    Ok(())
}

fn read_header(file: &mut File) -> io::Result<NpnkHeader> {
    let mut header = NpnkHeader::default();
    let bytes = unsafe {
        std::slice::from_raw_parts_mut(
            &mut header as *mut _ as *mut u8,
            mem::size_of::<NpnkHeader>(),
        )
    };

    file.read_exact(bytes)?;
    Ok(header)
}

fn nephyn(input_path: &str, output_path: &str, ds_factor: u32) -> Result<(), io::Error> {

    let start = Instant::now();

    let img = ImageReader::open(input_path)
        .expect("Failed to open image")
        .decode()
        .expect("Failed to decode image").to_rgb8();

    let yuv_img = rgb_to_yuv(&img);

    let (d_img, ds_img) = delta_down_res_predictor(&yuv_img, ds_factor);

    let mut ds_img_buf: Vec<u8> = ds_img.into_vec();
    let mut d_img_buf: Vec<u8> = d_img.into_vec();
    let mut lzw_ds_img_buf: Vec<u8> = Vec::new();
    let mut lzw_d_img_buf: Vec<u8> = Vec::new();

    lzw_encode(&mut Cursor::new(&mut ds_img_buf), &mut Cursor::new(&mut lzw_ds_img_buf))?;
    lzw_encode(&mut Cursor::new(&mut d_img_buf), &mut Cursor::new(&mut lzw_d_img_buf))?;

    let mut npnk = File::create(output_path)?;
    let header = NpnkHeader{
        width: img.width(),
        height: img.height(),
        ds_size: lzw_ds_img_buf.len() as u64,
        d_size: lzw_d_img_buf.len() as u64
    };

    write_header(&mut npnk, &header)?;
    npnk.write_all(&lzw_ds_img_buf)?;
    npnk.write_all(&lzw_d_img_buf)?;

    println!("Finished in {:?}", start.elapsed());
    println!("Saved converted image to {}", output_path);
    println!("Compression rate in reference to bitmap size {}",
        (header.ds_size + header.d_size) as f32 / (header.width * header.height * 3) as f32);

    Ok(())
}

fn denephyn(input_path: &str, output_path: &str, ds_factor: u32)  -> Result<(), io::Error> {

    let start = Instant::now();

    let mut npnk = File::open(input_path)?;
    let header = read_header(&mut npnk)?;

    let mut lzw_ds_img_buf: Vec<u8> = vec![0u8 ; header.ds_size as usize];
    let mut lzw_d_img_buf: Vec<u8>  = vec![0u8 ; header.d_size as usize];
    let mut ds_img_buf: Vec<u8> = Vec::new();
    let mut d_img_buf: Vec<u8> = Vec::new();

    npnk.read_exact(&mut lzw_ds_img_buf)?;
    npnk.read_exact(&mut lzw_d_img_buf)?;

    lzw_decode(&mut Cursor::new(&mut lzw_ds_img_buf), &mut Cursor::new(&mut ds_img_buf))?;
    lzw_decode(&mut Cursor::new(&mut lzw_d_img_buf), &mut Cursor::new(&mut d_img_buf))?;

    let ds_img: RgbImage =
        ImageBuffer::from_raw(header.width / ds_factor, header.height / ds_factor, ds_img_buf)
        .expect("File corrupted");
    let d_img: RgbImage =
        ImageBuffer::from_raw(header.width, header.height, d_img_buf)
        .expect("File corrupted");

    let yuv_img = dedelta_down_res_predictor(&d_img, &ds_img, ds_factor);

    let img = yuv_to_rgb(&yuv_img);

    img
        .save(Path::new(output_path))
        .expect("Failed to save output image");

    println!("Finished in {:?}", start.elapsed());
    println!("Saved converted image to {}", output_path);

    Ok(())
}

fn print_help(va: &Vec<String>) {
    println!("Usage:");
    println!("\tAny image file to npnk: {} npnk [image_path] [npnk_path]", va[0]);
    println!("\tnpnk to any image file: {} denpnk [npnk_path] [image_path]", va[0]);
}

fn main()
{
    let va: Vec<String> = env::args().collect();
    if va.len() != 4 {
        print_help(&va);
        std::process::exit(1);
    }
    if va[1].as_str() == "npnk" {
        nephyn(va[2].as_str(), va[3].as_str(), 4)
            .expect("Unsuccessful conversion");
    } else if va[1].as_str() == "denpnk" {
        denephyn(va[2].as_str(), va[3].as_str(), 4)
            .expect("Unsuccessful conversion");
    } else {
        print_help(&va);
    }
    std::process::exit(0);
}