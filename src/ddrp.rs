use image::{ ImageBuffer, RgbImage };

pub fn delta_down_res_predictor(yuv_img: &RgbImage, ds_factor: u32) -> (RgbImage, RgbImage)
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
                (y * width + x).clamp(0, n as u32 - 1)
            })
        })
        .collect();

    let ds_vidx: Vec<u32> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                ((y / ds_factor) * ds_width + (x / ds_factor)).clamp(0, ds_n as u32 - 1)
            })
        })
        .collect();

    for i in 0..vidx.len() {
        d_y[vidx[i] as usize] = y[vidx[i] as usize] - ds_y[ds_vidx[i] as usize] + 128;
        d_u[vidx[i] as usize] = u[vidx[i] as usize] - ds_u[ds_vidx[i] as usize] + 128;
        d_v[vidx[i] as usize] = v[vidx[i] as usize] - ds_v[ds_vidx[i] as usize] + 128;
    }

    let mut d_img: RgbImage = ImageBuffer::new(width, height);
    for (i, pixel) in d_img.pixels_mut().enumerate() {
        pixel[0] = d_y[i];
        pixel[1] = d_u[i];
        pixel[2] = d_v[i];
    }

    return (d_img, ds_img);
}

pub fn dedelta_down_res_predictor(d_img: &RgbImage, ds_img: &RgbImage, ds_factor: u32) -> RgbImage
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
                (y * width + x).clamp(0, n as u32 - 1)
            })
        })
        .collect();

    let ds_vidx: Vec<u32> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                ((y / ds_factor) * ds_width + (x / ds_factor)).clamp(0, ds_n as u32 - 1)
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