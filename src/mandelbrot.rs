use crate::types::Transform;
use crate::types::MandelImage;

use num::complex::Complex;

use std::time::SystemTime;

/// Calculates the number of iterations for a given complex number
/// to "escape" the Mandelbrot set
pub fn mandel(c: &Complex<f64>, max_iter: u32) -> u32 {
    if in_set(&c) {
        max_iter
    } else {
        let f = |z| z * z + c;
        let mut iter = 0;
        let mut next = Complex::new(0.0, 0.0);

        while next.norm() < 2.0 && iter < max_iter {
            next = f(next);
            iter += 1;
        }

        iter
    }
}

/// Checks if z is definitely within the Mandelbort set
/// according to wikipedia
fn in_set(z: &Complex<f64>) -> bool {
    let p = ((z.re - 0.25).powi(2) + z.im.powi(2)).sqrt();

    if z.re <= (p - 2.0 * p.powi(2) + 0.25) {
        return true;
    } else if ((z.re + 1.0).powi(2) + z.im.powi(2)) <= 0.0625 {
        return true;
    }

    false
}

pub fn generate_image(transform: &Transform, image: &mut MandelImage) {
    let start = SystemTime::now();
    let max_iter = image.max_iterations;

    image
        .iter_mut()
        .for_each(|p| p.iterations = mandel(&transform.point_to_complex(&p.point), max_iter));

    println!("Generated image in: {:?}", start.elapsed().unwrap());
}

/// histogram equalization
pub fn equalize_image(image: &mut MandelImage) {
    let start = SystemTime::now();

    // count each iteration count
    let size: usize = (image.max_iterations + 1) as usize;
    let mut iteration_counts = vec![0; size];
    image
        .iter()
        .for_each(|p| iteration_counts[p.iterations as usize] += 1);

    let mut cumulative_distribution = vec![0; size];

    // TODO: use iter
    // skip 'image.max_iterations' value (in set) in equalization
    let mut last = 0;
    for i in 0..image.max_iterations {
        cumulative_distribution[i as usize] = last + iteration_counts[i as usize];
        last = cumulative_distribution[i as usize];
    }

    // calc equalized array of iterations
    let sum: i32 = iteration_counts.iter().take(image.max_iterations as usize).sum();
    let mut adjusted = vec![0; size];
    let nominator = sum - cumulative_distribution[0];
    let hist = |n: u32| {
        if n == image.max_iterations {
            n
        }
        else  {
        ((cumulative_distribution[n as usize] - cumulative_distribution[0]) as f64
            / nominator as f64
            * (image.max_iterations - 1) as f64)
            .round() as u32
        }
    };

    for i in 0..size {
        adjusted[i] = hist(i as u32);
    }

    // let mut i = 0;
    // for c in iteration_counts.iter() {
    //     println!(
    //         "{}, {}, {}, {}",
    //         i, c, cumulative_distribution[i], adjusted[i]
    //     );
    //     i += 1;
    // }

    // set adjusted iterations
    image
        .iter_mut()
        .for_each(|p| p.iterations_equalized = adjusted[p.iterations as usize]);

    println!("Equalized image in: {:?}", start.elapsed().unwrap());
}
