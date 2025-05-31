mod dither;
mod gaussian;
mod kuwahara;
mod line_dither;
mod util;

use util::{apply_effect, load_image, save_image};

fn main() {
    let mut img = load_image("image.png");

    // img = gaussian::dog1(&img);
    img = line_dither::line_dither(2, 8, &img);

    save_image(img, "output.png");
}
