use crate::util;
use glam::Vec3;

// returns average color and std deviation
pub fn sample_square(
    img: &Vec<Vec<Vec3>>,
    x1: isize,
    y1: isize,
    x2: isize,
    y2: isize,
) -> (Vec3, f32) {
    let num_samples = ((x2 - x1) * (y2 - y1)) as f32;

    let mut luminance_sum = 0.;
    let mut luminance_sum2 = 0.;
    let mut color_sum = Vec3::splat(0.);

    for i in y1..y2 {
        for j in x1..x2 {
            let color = util::sample(img, i, j);
            color_sum += color;

            let lum = util::luminance(color);
            luminance_sum += lum;
            luminance_sum2 += lum * lum;
        }
    }

    let mean = luminance_sum / num_samples;
    let std = f32::abs(luminance_sum2 / num_samples - mean * mean);

    (color_sum / num_samples, std)
}

pub fn square_kuwahara(kernel_size: isize) -> impl Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3 {
    move |img: &Vec<Vec<Vec3>>, i: usize, j: usize| {
        let i = i as isize;
        let j = j as isize;

        let windows = vec![
            vec![-kernel_size, -kernel_size, 0, 0],
            vec![0, -kernel_size, kernel_size, 0],
            vec![-kernel_size, 0, 0, kernel_size],
            vec![0, 0, kernel_size, kernel_size],
        ];

        let samples = windows
            .iter()
            .map(|w| sample_square(img, j + w[0], i + w[1], j + w[2], i + w[3]))
            .collect::<Vec<_>>();

        let mut min_color = Vec3::splat(0.);
        let mut min_stdev = 1e9;

        for (color, stdev) in samples {
            if stdev < min_stdev {
                min_stdev = stdev;
                min_color = color;
            }
        }

        min_color
    }
}
