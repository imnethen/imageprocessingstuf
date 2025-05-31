use crate::util;
use glam::Vec3;

pub fn bayer_matrix() -> Vec<Vec<f32>> {
    vec![
        0, 32, 8, 40, 2, 34, 10, 42, // :3
        48, 16, 56, 24, 50, 18, 58, 26, // :3
        12, 44, 4, 36, 14, 46, 6, 38, // :3
        60, 28, 52, 20, 62, 30, 54, 22, // :3
        3, 35, 11, 43, 1, 33, 9, 41, // :3
        51, 19, 59, 27, 49, 17, 57, 25, // :3
        15, 47, 7, 39, 13, 45, 5, 37, // :3
        63, 31, 55, 23, 61, 29, 53, 21, // :3
    ]
    .iter()
    .map(|&c| (c as f32) / 64. - (63. / 128.))
    .collect::<Vec<_>>()
    .chunks(8)
    .map(|c| c.into())
    .collect::<Vec<_>>()
}

pub fn dither_color(
    num_colors: u32,
    spread: f32, // do i need this
    mat: &Vec<Vec<f32>>,
    color: Vec3,
    color_i: usize,
    color_j: usize,
) -> Vec3 {
    let new_color = color + spread * mat[color_i % mat.len()][color_j % mat.len()];
    util::quantize(num_colors, new_color)
}

pub fn dither_image(
    num_colors: u32,
    spread: f32, // h
    mat: Vec<Vec<f32>>,
) -> impl Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3 {
    move |img: &Vec<Vec<Vec3>>, i: usize, j: usize| {
        dither_color(num_colors, spread, &mat, img[i][j], i, j)
    }
}
