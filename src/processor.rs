use image::{DynamicImage, Rgba};
use rayon::prelude::*;

pub fn parallel_autocrop(img: &DynamicImage, bg: Rgba<u8>, tolerance: u8) -> Option<DynamicImage> {
    let rgb = img.to_rgb8();
    let (w, h) = rgb.dimensions();
    let data = rgb.as_raw();
    let stride = w as usize * 3;
    let bg_c = bg.0[0..3].to_vec();

    let rows_fg: Vec<bool> = (0..h as usize)
        .into_par_iter()
        .map(|y| {
            let row = &data[y * stride..y * stride + stride];
            row.chunks_exact(3).any(|p| {
                p.iter().zip(bg_c.iter()).map(|(&a, &b)| a.abs_diff(b) as u32).sum::<u32>() > tolerance as u32
            })
        })
        .collect();

    let top = rows_fg.iter().position(|&fg| fg)? as u32;
    let bottom = rows_fg.iter().rposition(|&fg| fg)? as u32;

    let (left, right) = (top..=bottom)
        .into_par_iter()
        .map(|y| {
            let row = &data[y as usize * stride..y as usize * stride + stride];
            let (mut l, mut r) = (w as usize, 0);
            for (x, p) in row.chunks_exact(3).enumerate() {
                if p.iter().zip(bg_c.iter()).map(|(&a, &b)| a.abs_diff(b) as u32).sum::<u32>() > tolerance as u32 {
                    if x < l { l = x; }
                    if x > r { r = x; }
                }
            }
            (l, r)
        })
        .reduce(|| (w as usize, 0), |(min_l, max_r), (l, r)| (min_l.min(l), max_r.max(r)));

    if left > right { return None; }
    Some(img.crop_imm(left as u32, top, (right - left + 1) as u32, (bottom - top + 1) as u32))
}
