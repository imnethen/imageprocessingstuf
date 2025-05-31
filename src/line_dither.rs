use crate::{dither, util};
use glam::Vec3;
use util::{apply_effect, sample};

/// img should be luminance
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
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
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

/// (i - j) % m, needed to not overflow i - j into negatives
fn submod(i: usize, j: usize, m: usize) -> usize {
    let s = (i as isize - j as isize) % m as isize;
    if s >= 0 {
        s as usize
    } else {
        (s + m as isize) as usize
    }
}

fn make_dithering_matrices(size: usize) -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
    let mut line_order = (0..size).collect::<Vec<_>>();

    use rand::seq::SliceRandom;
    line_order.shuffle(&mut rand::rng());

    let mut ditmat1 = vec![vec![0.; size]; size];
    let mut ditmat2 = vec![vec![0.; size]; size];

    for i in 0..size {
        for j in 0..size {
            ditmat1[i][j] =
                (line_order[(i + j) % size] * size + i) as f32 / (size * size) as f32 - 0.5;
            ditmat2[i][j] =
                (line_order[submod(i, j, size)] * size + i) as f32 / (size * size) as f32 - 0.5;
        }
    }

    (ditmat1, ditmat2)
}

pub fn line_dither(
    num_colors: u32,
    mat_size: usize,
    dog: &Vec<Vec<Vec3>>,
    orig: &Vec<Vec<Vec3>>,
) -> Vec<Vec<Vec3>> {
    let mut new_img = dog.clone();

    new_img = edge_detect(&new_img);

    let direction_matrix =
        make_direction_or_offset_matrix(&new_img, || rand::random_bool(0.5) as u32);
    let offset_matrix =
        make_direction_or_offset_matrix(&new_img, || rand::random_range(0..mat_size) as u32);

    let (dithering_matrix1, dithering_matrix2) = make_dithering_matrices(mat_size);

    for i in 0..dog.len() {
        for j in 0..dog[i].len() {
            new_img[i][j] = match direction_matrix[i][j] {
                1 | 2 => dither::dither_color(
                    num_colors,
                    1.,
                    &dithering_matrix1,
                    orig[i][j],
                    i,
                    j + offset_matrix[i][j] as usize,
                ),
                3 => dither::dither_color(
                    num_colors,
                    1.,
                    &dithering_matrix2,
                    orig[i][j],
                    i,
                    j + offset_matrix[i][j] as usize,
                ),
                _ => unreachable!(),
            };
        }
    }

    new_img
}
