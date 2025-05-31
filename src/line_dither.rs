use crate::{dither, util};
use glam::Vec3;
use util::{apply_effect, sample};

// img should be luminance
#[rustfmt::skip]
fn sobel(img: &Vec<Vec<Vec3>>, i: usize, j: usize) -> Vec3 {
    let i = i as isize; let j = j as isize;

    let sq = vec![
        vec![sample(img, i - 1, j - 1).x, sample(img, i - 1, j + 0).x, sample(img, i - 1, j + 1).x],
        vec![sample(img, i + 0, j - 1).x, sample(img, i + 0, j + 0).x, sample(img, i + 0, j + 1).x],
        vec![sample(img, i + 1, j - 1).x, sample(img, i + 1, j + 0).x, sample(img, i + 1, j + 1).x],
    ];

    let gx =
        3.  * sq[0][0] + 0. * sq[0][1] - 3.  * sq[0][2] +
        10. * sq[1][0] + 0. * sq[1][1] - 10. * sq[1][2] +
        3.  * sq[2][0] + 0. * sq[2][1] - 3.  * sq[2][2];

    let gy =
        3.  * sq[0][0] + 10. * sq[0][1] + 3. * sq[0][2] +
        0.  * sq[1][0] + 0.  * sq[1][1] + 0. * sq[1][2] +
        -3. * sq[2][0] - 10. * sq[2][1] - 3. * sq[2][2];

    let mag = Vec3::new(gx, gy, 0.).length();
    Vec3::splat(mag)
}

// TODO: move edge detection to its own moduel maybe ?
// TODO TODO: its like unclear which functions should be called with apply_effect and which just do their thing but idk how to not do that
pub fn edge_detect(img: &Vec<Vec<Vec3>>) -> Vec<Vec<Vec3>> {
    let mut new_img = img.clone();
    new_img = apply_effect(sobel, &new_img);

    let max = new_img
        .iter()
        .flatten()
        .map(|vec| vec.x)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    new_img = apply_effect(|im, i, j| im[i][j] / max, &new_img);

    let threshold = 0.1;
    new_img = apply_effect(
        |im, i, j| {
            if im[i][j].x > threshold {
                Vec3::splat(1.)
            } else {
                Vec3::splat(0.)
            }
        },
        &new_img,
    );
    new_img
}

fn floodfill<T: std::cmp::PartialEq + Copy>(
    img: &mut Vec<Vec<T>>,
    i: isize,
    j: isize,
    old_val: T,
    new_val: T,
) {
    if i < 0
        || j < 0
        || i >= img.len() as isize
        || j >= img[0].len() as isize
        || img[i as usize][j as usize] != old_val
    {
        return;
    }

    img[i as usize][j as usize] = new_val;
    stacker::maybe_grow(1024, 1024 * 1024, || {
        floodfill(img, i - 1, j, old_val, new_val);
        floodfill(img, i + 1, j, old_val, new_val);
        floodfill(img, i, j - 1, old_val, new_val);
        floodfill(img, i, j + 1, old_val, new_val);
    });
}

/// gen_number should be a function that generates a number from 0 to somethng
/// its needed to make direction and offset matrices in one function
fn make_direction_or_offset_matrix<T: Fn() -> u32>(
    img: &Vec<Vec<Vec3>>,
    gen_number: T,
) -> Vec<Vec<u32>> {
    let mut dirmat = vec![vec![0; img[0].len()]; img.len()];

    for i in 0..img.len() {
        for j in 0..img[i].len() {
            let clr = img[i][j].x;
            if clr > 0.99 {
                dirmat[i][j] = 1;
            }
        }
    }

    for i in 0..img.len() {
        for j in 0..img[i].len() {
            floodfill(&mut dirmat, i as isize, j as isize, 0, gen_number() + 2);
        }
    }

    dirmat
}

pub fn line_dither(num_colors: u32, mat_size: u32, img: &Vec<Vec<Vec3>>) -> Vec<Vec<Vec3>> {
    let mut new_img = img.clone();
    new_img = apply_effect(|im, i, j| Vec3::splat(util::luminance(im[i][j])), &new_img);
    new_img = edge_detect(&new_img);

    let dirmat = make_direction_or_offset_matrix(&new_img, || rand::random_bool(0.5) as u32);
    let offmat =
        make_direction_or_offset_matrix(&new_img, || rand::random_range(0..mat_size) as u32);
    new_img = offmat
        .iter()
        .map(|line| {
            line.iter()
                .map(|&clr| Vec3::splat(clr as f32 / mat_size as f32))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    new_img
}
