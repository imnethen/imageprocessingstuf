use crate::util;
use glam::Vec3;

fn gaussian(sigma: f32, pos: f32) -> f32 {
    1. / f32::sqrt(std::f32::consts::TAU * sigma * sigma)
        * f32::exp(-(pos * pos) / (2. * sigma * sigma))
}

pub fn gaussian_blur(
    kernel_size: u32,
    sigma: f32,
    vertical: bool,
) -> impl Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3 {
    move |img: &Vec<Vec<Vec3>>, i: usize, j: usize| {
        let i = i as isize;
        let j = j as isize;

        let mut color = Vec3::new(0., 0., 0.);
        let mut kernel_sum = 0.;

        for x in -(kernel_size as isize)..(kernel_size as isize) {
            let c = util::sample(
                img,
                i + x * (vertical as isize),
                j + x * (!vertical as isize),
            );
            let c = Vec3::splat(util::luminance(c));
            let gauss = gaussian(sigma, x as f32);

            color += c * gauss;
            kernel_sum += gauss;
        }

        color / kernel_sum
    }
}

pub fn gaussian_threshold(t: f32, phi: f32) -> impl Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3 {
    move |img: &Vec<Vec<Vec3>>, i: usize, j: usize| {
        img[i][j].map(|c| {
            if c < t {
                1. + f32::tanh(phi * (c - t))
                // quantize_color(8, Vec3::splat(c / t)).x
            } else {
                1.
            }
        })
    }
}

pub fn dog1(img: &Vec<Vec<Vec3>>) -> Vec<Vec<Vec3>> {
    use util::apply_effect;
    let orig = img.clone();

    let mut blurred_1 = apply_effect(gaussian_blur(5, 2., false), &img);
    blurred_1 = apply_effect(gaussian_blur(5, 2., true), &blurred_1);

    let mut blurred_2 = apply_effect(gaussian_blur(5, 3.2, false), &img);
    blurred_2 = apply_effect(gaussian_blur(5, 3.2, true), &blurred_2);

    let mut new_img = img.clone();
    new_img = apply_effect(|_, i, j| blurred_1[i][j] - blurred_2[i][j], &new_img);
    new_img = apply_effect(|im, i, j| im[i][j].map(|c| f32::clamp(c, 0., 1.)), &new_img);
    new_img = apply_effect(gaussian_threshold(0.01, 50.), &new_img);
    new_img = apply_effect(|im, i, j| im[i][j] * 2.0 - 1., &new_img);

    new_img = apply_effect(|im, i, j| im[i][j] * orig[i][j], &new_img);

    new_img
}
