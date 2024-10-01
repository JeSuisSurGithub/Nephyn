use std::simd::f32x4;
use image::{ ImageBuffer, RgbImage };

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

pub fn rgb_to_yuv(rgb_img: &RgbImage) -> RgbImage
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

pub fn yuv_to_rgb(yuv_img: &RgbImage) -> RgbImage
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