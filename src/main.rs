// #![feature(portable_simd)]

// use std::simd::f32x4;
// use image::{ Imagebuf, ImageReader, RgbImage };
// use std::path::Path;
// use std::env;
// use std::process;
// use std::process::Command;
// use std::io::{Write};
// use tempfile::NamedTempFile;

// fn rgb_to_yuv_simd(r: &[u8], g: &[u8], b: &[u8], y: &mut [u8], u: &mut [u8], v: &mut [u8])
// {
//     let chunk_size = f32x4::splat(0.0).len();
//     for i in (0..r.len()).step_by(chunk_size)
//     {
//         let r_vec = f32x4::from_array([r[i] as f32, r[i+1] as f32, r[i+2] as f32, r[i+3] as f32]);
//         let g_vec = f32x4::from_array([g[i] as f32, g[i+1] as f32, g[i+2] as f32, g[i+3] as f32]);
//         let b_vec = f32x4::from_array([b[i] as f32, b[i+1] as f32, b[i+2] as f32, b[i+3] as f32]);

//         let y_vec = f32x4::splat(0.299) * r_vec + f32x4::splat(0.587) * g_vec + f32x4::splat(0.114) * b_vec;
//         let u_vec = f32x4::splat(128.0) + f32x4::splat(-0.168736) * r_vec + f32x4::splat(-0.331264) * g_vec + f32x4::splat(0.5) * b_vec;
//         let v_vec = f32x4::splat(128.0) + f32x4::splat(0.5) * r_vec + f32x4::splat(-0.418688) * g_vec + f32x4::splat(-0.081312) * b_vec;

//         for j in 0..chunk_size {
//             y[i + j] = y_vec[j].max(0.0).min(255.0) as u8;
//             u[i + j] = u_vec[j].max(0.0).min(255.0) as u8;
//             v[i + j] = v_vec[j].max(0.0).min(255.0) as u8;
//         }
//     }
// }

// fn yuv_to_rgb_simd(y: &[u8], u: &[u8], v: &[u8], r: &mut [u8], g: &mut [u8], b: &mut [u8])
// {
//     let chunk_size = f32x4::splat(0.0).len();
//     for i in (0..y.len()).step_by(chunk_size)
//     {
//         let y_vec = f32x4::from_array([y[i] as f32, y[i+1] as f32, y[i+2] as f32, y[i+3] as f32]);
//         let u_vec = f32x4::from_array([u[i] as f32, u[i+1] as f32, u[i+2] as f32, u[i+3] as f32]);
//         let v_vec = f32x4::from_array([v[i] as f32, v[i+1] as f32, v[i+2] as f32, v[i+3] as f32]);

//         let r_vec = y_vec + f32x4::splat(1.402) * (v_vec - f32x4::splat(128.0));
//         let g_vec = y_vec - f32x4::splat(0.344136) * (u_vec - f32x4::splat(128.0)) - f32x4::splat(0.714136) * (v_vec - f32x4::splat(128.0));
//         let b_vec = y_vec + f32x4::splat(1.772) * (u_vec - f32x4::splat(128.0));

//         for j in 0..chunk_size {
//             r[i + j] = r_vec[j].max(0.0).min(255.0) as u8;
//             g[i + j] = g_vec[j].max(0.0).min(255.0) as u8;
//             b[i + j] = b_vec[j].max(0.0).min(255.0) as u8;
//         }
//     }
// }

// fn rgb_to_yuv(rgb_img: &RgbImage) -> RgbImage
// {
//     let (width, height) = rgb_img.dimensions();
//     let n = (width * height) as usize;

//     let mut r = vec![0u8; n];
//     let mut g = vec![0u8; n];
//     let mut b = vec![0u8; n];
//     let mut y = vec![0u8; n];
//     let mut u = vec![0u8; n];
//     let mut v = vec![0u8; n];

//     for (i, pixel) in rgb_img.pixels().enumerate() {
//         r[i] = pixel[0];
//         g[i] = pixel[1];
//         b[i] = pixel[2];
//     }

//     rgb_to_yuv_simd(&r, &g, &b, &mut y, &mut u, &mut v);

//     let mut yuv_img: RgbImage = Imagebuf::new(width, height);
//     for (i, pixel) in yuv_img.pixels_mut().enumerate() {
//         pixel[0] = y[i];
//         pixel[1] = u[i];
//         pixel[2] = v[i];
//     }
//     return yuv_img;
// }

// fn yuv_to_rgb(yuv_img: &RgbImage) -> RgbImage
// {
//     let (width, height) = yuv_img.dimensions();
//     let n = (width * height) as usize;

//     let mut y = vec![0u8; n];
//     let mut u = vec![0u8; n];
//     let mut v = vec![0u8; n];
//     let mut r = vec![0u8; n];
//     let mut g = vec![0u8; n];
//     let mut b = vec![0u8; n];

//     for (i, pixel) in yuv_img.pixels().enumerate() {
//         y[i] = pixel[0];
//         u[i] = pixel[1];
//         v[i] = pixel[2];
//     }

//     yuv_to_rgb_simd(&y, &u, &v, &mut r, &mut g, &mut b);

//     let mut rgb_img: RgbImage = Imagebuf::new(width, height);
//     for (i, pixel) in rgb_img.pixels_mut().enumerate() {
//         pixel[0] = r[i];
//         pixel[1] = g[i];
//         pixel[2] = b[i];
//     }
//     return rgb_img;
// }

// fn delta_down_res_predictor(yuv_img: &RgbImage, ds_factor: u32) -> (RgbImage, RgbImage)
// {
//     let (width, height) = yuv_img.dimensions();
//     let n = (width * height) as usize;

//     let ds_width = width / ds_factor as u32;
//     let ds_height = height / ds_factor as u32;
//     let ds_n = (ds_width * ds_height) as usize;

//     let ds_img = image::imageops::resize(yuv_img, ds_width, ds_height, image::imageops::FilterType::CatmullRom);

//     let mut y = vec![0u8; n];
//     let mut u = vec![0u8; n];
//     let mut v = vec![0u8; n];
//     let mut d_y = vec![0u8; n];
//     let mut d_u = vec![0u8; n];
//     let mut d_v = vec![0u8; n];
//     let mut ds_y = vec![0u8; ds_n];
//     let mut ds_u = vec![0u8; ds_n];
//     let mut ds_v = vec![0u8; ds_n];

//     for (i, pixel) in yuv_img.pixels().enumerate() {
//         y[i] = pixel[0];
//         u[i] = pixel[1];
//         v[i] = pixel[2];
//     }
//     for (i, pixel) in ds_img.pixels().enumerate() {
//         ds_y[i] = pixel[0];
//         ds_u[i] = pixel[1];
//         ds_v[i] = pixel[2];
//     }

//     let vidx: Vec<u32> = (0..height)
//         .flat_map(|y| {
//             (0..width).map(move |x| {
//                 (y * width + x).clamp(0, n as u32 - 1)
//             })
//         })
//         .collect();

//     let ds_vidx: Vec<u32> = (0..height)
//         .flat_map(|y| {
//             (0..width).map(move |x| {
//                 ((y / ds_factor) * ds_width + (x / ds_factor)).clamp(0, ds_n as u32 - 1)
//             })
//         })
//         .collect();

//     for i in 0..vidx.len() {
//         d_y[vidx[i] as usize] = y[vidx[i] as usize] - ds_y[ds_vidx[i] as usize] + 128;
//         d_u[vidx[i] as usize] = u[vidx[i] as usize] - ds_u[ds_vidx[i] as usize] + 128;
//         d_v[vidx[i] as usize] = v[vidx[i] as usize] - ds_v[ds_vidx[i] as usize] + 128;
//     }

//     let mut d_img: RgbImage = Imagebuf::new(width, height);
//     for (i, pixel) in d_img.pixels_mut().enumerate() {
//         pixel[0] = d_y[i];
//         pixel[1] = d_u[i];
//         pixel[2] = d_v[i];
//     }

//     return (d_img, ds_img);
// }

// fn dedelta_down_res_predictor(d_img: &RgbImage, ds_img: &RgbImage, ds_factor: u32) -> RgbImage
// {
//     let (width, height) = d_img.dimensions();
//     let n = (width * height) as usize;

//     let ds_width = width / ds_factor as u32;
//     let ds_height = height / ds_factor as u32;
//     let ds_n = (ds_width * ds_height) as usize;

//     let mut d_y = vec![0u8; n];
//     let mut d_u = vec![0u8; n];
//     let mut d_v = vec![0u8; n];
//     let mut ds_y = vec![0u8; ds_n];
//     let mut ds_u = vec![0u8; ds_n];
//     let mut ds_v = vec![0u8; ds_n];
//     let mut y = vec![0u8; n];
//     let mut u = vec![0u8; n];
//     let mut v = vec![0u8; n];

//     for (i, pixel) in d_img.pixels().enumerate() {
//         d_y[i] = pixel[0];
//         d_u[i] = pixel[1];
//         d_v[i] = pixel[2];
//     }
//     for (i, pixel) in ds_img.pixels().enumerate() {
//         ds_y[i] = pixel[0];
//         ds_u[i] = pixel[1];
//         ds_v[i] = pixel[2];
//     }

//     let vidx: Vec<u32> = (0..height)
//         .flat_map(|y| {
//             (0..width).map(move |x| {
//                 (y * width + x).clamp(0, n as u32 - 1)
//             })
//         })
//         .collect();

//     let ds_vidx: Vec<u32> = (0..height)
//         .flat_map(|y| {
//             (0..width).map(move |x| {
//                 ((y / ds_factor) * ds_width + (x / ds_factor)).clamp(0, ds_n as u32 - 1)
//             })
//         })
//         .collect();

//     for i in 0..vidx.len() {
//         y[vidx[i] as usize] = d_y[vidx[i] as usize] + ds_y[ds_vidx[i] as usize] - 128;
//         u[vidx[i] as usize] = d_u[vidx[i] as usize] + ds_u[ds_vidx[i] as usize] - 128;
//         v[vidx[i] as usize] = d_v[vidx[i] as usize] + ds_v[ds_vidx[i] as usize] - 128;
//     }

//     let mut yuv_img: RgbImage = Imagebuf::new(width, height);
//     for (i, pixel) in yuv_img.pixels_mut().enumerate() {
//         pixel[0] = y[i];
//         pixel[1] = u[i];
//         pixel[2] = v[i];
//     }

//     return yuv_img;
// }

// fn nephynika(input_path: &str, delta_path: &str, downres_path: &str, ds_factor: u32) {

//     let img = ImageReader::open(input_path)
//         .expect("Failed to open image")
//         .decode()
//         .expect("Failed to decode image").to_rgb8();

//     let yuv_img = rgb_to_yuv(&img);

//     let (d_img, ds_img) = delta_down_res_predictor(&yuv_img, ds_factor);

//     d_img
//         .save(Path::new(delta_path))
//         .expect("Failed to save output image");

//     ds_img
//         .save(Path::new(downres_path))
//         .expect("Failed to save output image");

//     ////println!("Saved the processed image to {}", delta_path);
// }

// fn denephynika(delta_path: &str, downres_path: &str, output_path: &str, ds_factor: u32) {

//     let d_img = ImageReader::open(delta_path)
//         .expect("Failed to open image")
//         .decode()
//         .expect("Failed to decode image").to_rgb8();

//     let ds_img = ImageReader::open(downres_path)
//         .expect("Failed to open image")
//         .decode()
//         .expect("Failed to decode image").to_rgb8();

//     let yuv_img = dedelta_down_res_predictor(&d_img, &ds_img, ds_factor);

//     let img = yuv_to_rgb(&yuv_img);

//     img
//         .save(Path::new(output_path))
//         .expect("Failed to save output image");

//     ////println!("Saved the processed image to {}", output_path);
// }

// fn main() {
//     let args: Vec<String> = env::args().skip(1).collect();

//     if args.len() != 3 {
//         process::exit(0);
//     }

//     nephynika(args[0].as_str(), args[1].as_str(), args[2].as_str(), 4);
//     denephynika(args[1].as_str(), args[2].as_str(), args[0].as_str(), 4);
// }

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::time::Instant;
use lazy_static::lazy_static;
use std::hash::BuildHasherDefault;
use ahash::AHasher;

const MAX_DICT_SIZE: usize = 65535;
const CLEAR_CODE: u16 = 65535;

lazy_static! {
    static ref ENCODE_DICT: HashMap<Vec<u8>, u16, BuildHasherDefault<AHasher>> = {
        let mut dict = HashMap::with_capacity_and_hasher(MAX_DICT_SIZE, BuildHasherDefault::default());
        for i in 0..=255 {
            dict.insert(vec![i as u8], i as u16);
        }
        dict
    };
}

lazy_static! {
    static ref DECODE_TABLE: Vec<Vec<u8>> = {
        let mut table = vec![vec![]; MAX_DICT_SIZE];

        for i in 0..=255 {
            table[i] = vec![i as u8];
        }
        table
    };
}

struct LZWEncodeCtx {
    dict: HashMap<Vec<u8>, u16, BuildHasherDefault<AHasher>>,
    cur_code: u16,
    bit_width: u8,
    bit_buf: u64,
    bit_buf_len: u8,
    next_growth: u32,
}
struct LZWDecodeCtx {
    table: Vec<Vec<u8>>,
    cur_code: u16,
    bit_width: u8,
    bit_buf: u64,
    bit_buf_len: u8,
    next_growth: u32,
}

impl LZWEncodeCtx {
    fn new() -> LZWEncodeCtx {
        LZWEncodeCtx {
            dict: ENCODE_DICT.clone(),
            cur_code: 256,
            bit_width: 9,
            bit_buf: 0,
            bit_buf_len: 0,
            next_growth: 512,
        }
    }

    fn reset(&mut self) {
        self.dict = ENCODE_DICT.clone();
        self.cur_code = 256;
        self.bit_width = 9;
        self.next_growth = 512;
    }
}

impl LZWDecodeCtx {
    fn new() -> LZWDecodeCtx {
        LZWDecodeCtx {
            table: DECODE_TABLE.clone(),
            cur_code: 256,
            bit_width: 9,
            bit_buf: 0,
            bit_buf_len: 0,
            next_growth: 511,
        }
    }

    fn reset(&mut self) {
        self.table = DECODE_TABLE.clone();
        self.cur_code = 256;
        self.bit_width = 9;
        self.next_growth = 511;
    }
}

fn lzw_encode(input: &mut dyn Read, output: &mut dyn Write) -> Result<(), io::Error>
{
    let mut state: LZWEncodeCtx = LZWEncodeCtx::new();

    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);

    let mut byte = [0u8; 1];
    let mut cur_data = Vec::with_capacity(16);

    while reader.read_exact(&mut byte).is_ok() {
        cur_data.push(byte[0]);

        if !state.dict.contains_key(&cur_data) {
            let mut prev_cur_data = cur_data.clone();
            prev_cur_data.pop();
            write_code(state.dict[&prev_cur_data], 32, &mut state, &mut writer)?;

            if state.cur_code != CLEAR_CODE {
                if (state.cur_code as u32) == state.next_growth {
                    state.bit_width += 1;
                    state.next_growth *= 2;
                }

                state.dict.insert(cur_data.clone(), state.cur_code);
                state.cur_code += 1;
            } else {
                write_code(CLEAR_CODE, 32, &mut state, &mut writer)?;
                state.reset();
            }

            cur_data.clear();
            cur_data.push(byte[0]);
        }
    }

    if !cur_data.is_empty() {
        write_code(state.dict[&cur_data], 0, &mut state, &mut writer)?;
    }

    if state.bit_buf > 0 {
        writer.write_all(&[(state.bit_buf << (8 - state.bit_buf_len)) as u8])?;
    }

    writer.flush()?;
    Ok(())
}

fn write_code(code: u16, threshold: u8, state: &mut LZWEncodeCtx, writer: &mut BufWriter<&mut dyn Write>) -> Result<(), io::Error>
{
    state.bit_buf = (state.bit_buf << state.bit_width) | code as u64;
    state.bit_buf_len += state.bit_width;

    if state.bit_buf_len > threshold
    {
        while state.bit_buf_len >= 8 {
            writer.write_all(&[(state.bit_buf >> (state.bit_buf_len - 8)) as u8])?;
            state.bit_buf_len -= 8;
        }
    }
    Ok(())
}

fn lzw_decode(input: &mut dyn Read, output: &mut dyn Write) -> io::Result<()>
{
    let mut state = LZWDecodeCtx::new();

    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);

    let mut prev_code: Option<u16> = None;
    let mut byte = [0u8; 1];

    while reader.read_exact(&mut byte).is_ok() {
        state.bit_buf = (state.bit_buf << 8) | byte[0] as u64;
        state.bit_buf_len += 8;

        while state.bit_buf_len >= state.bit_width {
            let code = (state.bit_buf >> (state.bit_buf_len - state.bit_width)) as u16;
            state.bit_buf_len -= state.bit_width;
            state.bit_buf &= (1 << state.bit_buf_len) - 1;

            if code == CLEAR_CODE {
                state.reset();
                prev_code = None;
                continue;
            }

            let cur_data = if code < state.cur_code {
                state.table[code as usize].clone()
            } else if let Some(prev) = prev_code {
                let mut cur_data = state.table[prev as usize].clone();
                cur_data.push(cur_data[0]);
                cur_data
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid LZW code"));
            };

            writer.write_all(&cur_data)?;

            if let Some(prev) = prev_code
            {
                let mut new_cur_data = state.table[prev as usize].clone();
                new_cur_data.push(cur_data[0]);

                state.table[state.cur_code as usize] = new_cur_data;

                if (state.cur_code as u32) == state.next_growth {
                    state.bit_width += 1;
                    state.next_growth = 2u32.pow(state.bit_width as u32) - 1;
                }

                state.cur_code += 1;
            }

            prev_code = Some(code);
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), io::Error>
{
    {
        let start = Instant::now();

        let infile = File::open("experiments/sirin_delta.bmp")?;
        let mut outfile = File::create("experiments/sirin_delta.lzw")?;
        lzw_encode(&mut infile.take(u64::MAX), &mut outfile)?;

        let duration = start.elapsed();
        println!("Time taken: {:?}", duration);
    }
    {
        let start = Instant::now();

        let infile = File::open("experiments/sirin_delta.lzw")?;
        let mut outfile = File::create("experiments/sirin_delta_restored.bmp")?;

        lzw_decode(&mut infile.take(u64::MAX), &mut outfile)?;

        let duration = start.elapsed();
        println!("Time taken: {:?}", duration);
    }

    Ok(())
}