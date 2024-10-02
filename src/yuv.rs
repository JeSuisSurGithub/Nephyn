use image::{ ImageBuffer, RgbImage };

///////////////////////////////////////////////////////////////////////////////

// https://stackoverflow.com/questions/10566668/lossless-rgb-to-ycbcr-transformation/12146329#12146329

fn rgb_to_gcbcr(red: u8, green: u8, blue: u8) -> (u8, u8, u8) {
    let cb = blue.wrapping_sub(green);
    let cr = red.wrapping_sub(green);
    (green, cb, cr)
}

fn gcbcr_to_rgb(green: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    let blue = green.wrapping_add(cb);
    let red = green.wrapping_add(cr);
    (red, green, blue)
}
///////////////////////////////////////////////////////////////////////////////


pub fn rgb_to_yuv(rgb_img: &RgbImage) -> RgbImage
{
    let (width, height) = rgb_img.dimensions();
    let mut yuv_img: RgbImage = ImageBuffer::new(width, height);

    for py in 0..height {
        for px in 0..width {
            let rgb_px = rgb_img.get_pixel(px, py);
            let yuv_px = yuv_img.get_pixel_mut(px, py);

            // let (y, u, v) = rgb_to_ycocg24(rgb_px[0], rgb_px[1], rgb_px[2]);
            // yuv_px[0] = y;
            // yuv_px[1] = u as u8;
            // yuv_px[2] = v as u8;

            let (y, u, v) = rgb_to_gcbcr(rgb_px[0], rgb_px[1], rgb_px[2]);
            yuv_px[0] = y;
            yuv_px[1] = u;
            yuv_px[2] = v;
        }
    }
    yuv_img
}

pub fn yuv_to_rgb(yuv_img: &RgbImage) -> RgbImage
{
    let (width, height) = yuv_img.dimensions();
    let mut rgb_image: RgbImage = ImageBuffer::new(width, height);

    for py in 0..height {
        for px in 0..width {
            let yuv_px = yuv_img.get_pixel(px, py);
            let rgb_px = rgb_image.get_pixel_mut(px, py);

            // let (r, g, b) = ycocg24_to_rgb(yuv_px[0], yuv_px[1] as i8, yuv_px[2] as i8);
            // rgb_px[0] = r;
            // rgb_px[1] = g;
            // rgb_px[2] = b;

            let (r, g, b) = gcbcr_to_rgb(yuv_px[0], yuv_px[1], yuv_px[2]);
            rgb_px[0] = r;
            rgb_px[1] = g;
            rgb_px[2] = b;
        }
    }
    rgb_image
}