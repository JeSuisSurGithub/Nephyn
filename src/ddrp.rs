use image::{ ImageBuffer, RgbImage };

const DEFAULT_FILTER: image::imageops::FilterType = image::imageops::FilterType::Nearest;

pub fn delta_down_res_predictor(yuv_img: &RgbImage, ds_factor: u32) -> (RgbImage, RgbImage)
{
    let (width, height) = yuv_img.dimensions();

    let ds_width = width / ds_factor as u32;
    let ds_height = height / ds_factor as u32;

    let ds_img = image::imageops::resize(yuv_img, ds_width, ds_height, DEFAULT_FILTER);

    let mut d_img: RgbImage = ImageBuffer::new(width, height);

    for py in 0..height {
        for px in 0..width {
            let pixel = yuv_img.get_pixel(px, py);
            let ds_pixel = ds_img.get_pixel((px / ds_factor).clamp(0, ds_width - 1), (py / ds_factor).clamp(0, ds_height - 1));
            let d_pixel = d_img.get_pixel_mut(px, py);
            d_pixel[0] = pixel[0].wrapping_sub(ds_pixel[0]).wrapping_add(128);
            d_pixel[1] = pixel[1].wrapping_sub(ds_pixel[1]).wrapping_add(128);
            d_pixel[2] = pixel[2].wrapping_sub(ds_pixel[2]).wrapping_add(128);
        }
    }

    return (d_img, ds_img);
}

pub fn dedelta_down_res_predictor(d_img: &RgbImage, ds_img: &RgbImage, ds_factor: u32) -> RgbImage
{
    let (width, height) = d_img.dimensions();

    let ds_width = width / ds_factor as u32;
    let ds_height = height / ds_factor as u32;

    let mut yuv_img: RgbImage = ImageBuffer::new(width, height);

    for py in 0..height {
        for px in 0..width {
            let d_pixel = d_img.get_pixel(px, py);
            let ds_pixel = ds_img.get_pixel((px / ds_factor).clamp(0, ds_width - 1), (py / ds_factor).clamp(0, ds_height - 1));
            let pixel = yuv_img.get_pixel_mut(px, py);
            pixel[0] = d_pixel[0].wrapping_add(ds_pixel[0]).wrapping_sub(128);
            pixel[1] = d_pixel[1].wrapping_add(ds_pixel[1]).wrapping_sub(128);
            pixel[2] = d_pixel[2].wrapping_add(ds_pixel[2]).wrapping_sub(128);
        }
    }

    return yuv_img;
}