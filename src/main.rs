use glam::Vec3;

fn load_image(path: &str) -> Vec<Vec<Vec3>> {
    let img_buf = image::open(path).unwrap().into_rgb8();
    let width = img_buf.width() as usize;
    let img_vec = img_buf.into_vec();

    let img_2dvec = img_vec
        .chunks(3)
        .map(|c| Vec3::new(c[0] as f32 / 255., c[1] as f32 / 255., c[2] as f32 / 255.))
        .collect::<Vec<_>>()
        .chunks(width)
        .map(|c| c.into())
        .collect::<Vec<_>>();

    img_2dvec
}

fn save_image(img: Vec<Vec<Vec3>>, path: &str) {
    let flat_img = img
        .iter()
        .flatten()
        .map(|v| {
            vec![
                (v.x * 255.).round() as u8,
                (v.y * 255.).round() as u8,
                (v.z * 255.).round() as u8,
            ]
        })
        .flatten()
        .collect::<Vec<_>>();

    image::save_buffer(
        path,
        flat_img.as_slice(),
        img[0].len() as u32,
        img.len() as u32,
        image::ExtendedColorType::Rgb8,
    )
    .unwrap();
}

fn apply_effect<T: Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3>(
    effect: T,
    img: &Vec<Vec<Vec3>>,
) -> Vec<Vec<Vec3>> {
    let mut img2 = img.clone();
    for i in 0..img.len() {
        for j in 0..img[0].len() {
            img2[i][j] = effect(img, i, j);
        }
    }

    img2
}

fn main() {
    let mut img = load_image("image.png");
    let orig = img.clone();
    // img = apply_effect(square_kuwahara(4), &img);

    // img = apply_effect(square_kuwahara(4), &img);
    // img = apply_effect(dither(2, 1.), &img);

    let mut blurred_1 = apply_effect(gaussian_blur(5, 2., false), &img);
    blurred_1 = apply_effect(gaussian_blur(5, 2., true), &blurred_1);

    let mut blurred_2 = apply_effect(gaussian_blur(5, 3.2, false), &img);
    blurred_2 = apply_effect(gaussian_blur(5, 3.2, true), &blurred_2);

    img = apply_effect(|_, i, j| blurred_1[i][j] - blurred_2[i][j], &img);
    img = apply_effect(|im, i, j| im[i][j].map(|c| f32::clamp(c, 0., 1.)), &img);
    img = apply_effect(threshold(0.01, 50.), &img);
    img = apply_effect(|im, i, j| im[i][j] * 2.0 - 1., &img);

    img = apply_effect(|im, i, j| im[i][j] * orig[i][j], &img);

    // img = apply_effect(dither(4, 1.), &img);

    save_image(img, "output.png");
}

fn quantize_color(num_colors: u32, color: Vec3) -> Vec3 {
    let fnum = num_colors as f32 - 1.;
    (color * fnum).map(|c| c.round()) / fnum
}

fn quantize(num_colors: u32) -> impl Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3 {
    move |img: &Vec<Vec<Vec3>>, i: usize, j: usize| quantize_color(num_colors, img[i][j])
}

fn dither(num_colors: u32, spread: f32) -> impl Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3 {
    let mat: Vec<Vec<f32>> = {
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
    };

    // println!("{:?}", mat);

    move |img: &Vec<Vec<Vec3>>, i: usize, j: usize| {
        let color = img[i][j] + spread * mat[i % 8][j % 8];
        quantize_color(num_colors, color)
    }
}

// float gaussian(float sigma, float pos) {
//             return (1.0f / sqrt(2.0f * PI * sigma * sigma)) * exp(-(pos * pos) / (2.0f * sigma * sigma));
//         }

fn sample(img: &Vec<Vec<Vec3>>, i: isize, j: isize) -> Vec3 {
    let i = isize::clamp(i, 0, img.len() as isize - 1);
    let j = isize::clamp(j, 0, img[0].len() as isize - 1);
    img[i as usize][j as usize]
}

fn gaussian(sigma: f32, pos: f32) -> f32 {
    1. / f32::sqrt(std::f32::consts::TAU * sigma * sigma)
        * f32::exp(-(pos * pos) / (2. * sigma * sigma))
}

fn luminance(color: Vec3) -> f32 {
    color.dot(Vec3::new(0.299, 0.587, 0.114))
}

fn gaussian_blur(
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
            let c = sample(
                img,
                i + x * (vertical as isize),
                j + x * (!vertical as isize),
            );
            let c = Vec3::splat(luminance(c));
            let gauss = gaussian(sigma, x as f32);

            color += c * gauss;
            kernel_sum += gauss;
        }

        color / kernel_sum
    }
}

fn threshold(t: f32, phi: f32) -> impl Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3 {
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

// returns average color and std deviation
fn sample_square(img: &Vec<Vec<Vec3>>, x1: isize, y1: isize, x2: isize, y2: isize) -> (Vec3, f32) {
    let num_samples = ((x2 - x1) * (y2 - y1)) as f32;

    let mut luminance_sum = 0.;
    let mut luminance_sum2 = 0.;
    let mut color_sum = Vec3::splat(0.);

    for i in y1..y2 {
        for j in x1..x2 {
            let color = sample(img, i, j);
            color_sum += color;

            let lum = luminance(color);
            luminance_sum += lum;
            luminance_sum2 += lum * lum;
        }
    }

    let mean = luminance_sum / num_samples;
    let std = f32::abs(luminance_sum2 / num_samples - mean * mean);

    (color_sum / num_samples, std)
}

fn square_kuwahara(kernel_size: isize) -> impl Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3 {
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
