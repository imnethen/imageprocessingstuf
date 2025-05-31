use glam::Vec3;

pub fn load_image(path: &str) -> Vec<Vec<Vec3>> {
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

pub fn save_image(img: Vec<Vec<Vec3>>, path: &str) {
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

pub fn apply_effect<T: Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3>(
    effect: T,
    img: &Vec<Vec<Vec3>>,
) -> Vec<Vec<Vec3>> {
    let mut new_img = img.clone();
    for i in 0..img.len() {
        for j in 0..img[0].len() {
            new_img[i][j] = effect(img, i, j);
        }
    }

    new_img
}

// pub fn apply_multipass_effect<T: Fn(&Vec<Vec<Vec3>>, usize, usize) -> Vec3>(
//     effects: Vec<T>,
//     img: &Vec<Vec<Vec3>>,
// ) -> Vec<Vec<Vec3>> {
//     let mut new_img = img.clone();

//     for effect in effects {
//         new_img = apply_effect(effect, &new_img);
//     }

//     new_img
// }

pub fn quantize(num_colors: u32, color: Vec3) -> Vec3 {
    let fnum = num_colors as f32 - 1.;
    (color * fnum).map(|c| c.round()) / fnum
}

// clamp to edge
pub fn sample(img: &Vec<Vec<Vec3>>, i: isize, j: isize) -> Vec3 {
    let i = isize::clamp(i, 0, img.len() as isize - 1);
    let j = isize::clamp(j, 0, img[0].len() as isize - 1);
    img[i as usize][j as usize]
}

pub fn luminance(color: Vec3) -> f32 {
    color.dot(Vec3::new(0.299, 0.587, 0.114))
}
