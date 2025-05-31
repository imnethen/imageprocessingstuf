mod dither;
mod gaussian;
mod kuwahara;
mod line_dither;
mod util;

use util::{apply_effect, load_image, save_image};

fn main() {
    let mut img = load_image("image.png");

    let orig = apply_effect(
        |im, i, j| glam::Vec3::splat(util::luminance(im[i][j])),
        &img,
    );

    img = gaussian::dog2(0.002, &img);
    img = line_dither::line_dither(2, 256, &img, &orig);

    save_image(img, "output.png");
}
