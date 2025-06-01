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

    img = gaussian::dog2(0.004, &img);
    // img = apply_effect(max_blur, &img);
    // img = line_dither::edge_detect(&img);
    img = line_dither::line_dither(2, 256, &img, &orig);

    save_image(img, "output.png");
}

use glam::Vec3;
fn max_blur(img: &Vec<Vec<Vec3>>, i: usize, j: usize) -> Vec3 {
    let i = i as isize;
    let j = j as isize;
    let mut res = Vec3::splat(0.);

    for oi in -2..2 {
        for oj in -2..2 {
            let clr = util::sample(img, i + oi, j + oj);
            res = res.max(clr);
        }
    }

    res
}
