extern crate num;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Point;

use num::complex::Complex;

use std::ops::{Deref, DerefMut};
use std::thread;
use std::time::SystemTime;

const MAX_ITER: u32 = 150;

/// Transforms to/from pixels and complex numbers
struct Transform {
    x: f64,
    y: f64,
    scale: f64,
    window_size: (u32, u32),
}

/// Information for each pixel in MandelImage
#[derive(Clone)]
struct MandelPixel {
    point: Point,
    iterations: u32,
    iterations_equalized: u32,
}

/// Generated image data for the Mandelbrot set
struct MandelImage {
    width: u32,
    height: u32,
    data: Vec<MandelPixel>,
}

impl Transform {
    fn new(window_size: (u32, u32)) -> Self {
        Transform {
            scale: window_size.0 as f64 * 0.28,
            x: window_size.0 as f64 * 0.7,
            y: window_size.1 as f64 * 0.5,
            window_size,
        }
    }

    fn point_to_complex(&self, p: &Point) -> Complex<f64> {
        self.pos_to_complex(p.x, p.y)
    }

    // TODO: Remove
    fn pos_to_complex(&self, x: i32, y: i32) -> Complex<f64> {
        Complex::new(
            (x as f64 - self.x) / self.scale,
            (y as f64 - self.y) / self.scale,
        )
    }

    fn _complex_to_point(&self, z: Complex<f64>) -> Point {
        Point::new(
            (z.re * self.scale + self.x) as i32,
            (z.im * self.scale + self.y) as i32,
        )
    }

    fn zoom(&mut self, factor: f64) {
        let z_center = self.pos_to_complex(
            (self.window_size.0 / 2) as i32,
            (self.window_size.1 / 2) as i32,
        );
        self.scale *= factor;
        self.x = self.window_size.0 as f64 / 2.0 - z_center.re * self.scale;
        self.y = self.window_size.1 as f64 / 2.0 - z_center.im * self.scale;

        println!("Zoom: {}", self.zoom_factor());
    }

    fn zoom_factor(&self) -> f64 {
        self.scale / (self.window_size.0 as f64 * 0.28)
    }

    fn center_at(&mut self, z: &Complex<f64>) {
        self.x = self.window_size.0 as f64 / 2.0 - z.re * self.scale;
        self.y = self.window_size.1 as f64 / 2.0 - z.im * self.scale;
    }
}

impl MandelPixel {
    fn new(x: i32, y: i32) -> Self {
        MandelPixel {
            point: Point::new(x, y),
            iterations: 0,
            iterations_equalized: 0,
        }
    }
}

impl MandelImage {
    fn new(width: u32, height: u32) -> Self {
        let start = SystemTime::now();
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                pixels.push(MandelPixel::new(x as i32, y as i32));
            }
        }

        println!("Created image in: {:?}", start.elapsed().unwrap());
        MandelImage {
            width,
            height,
            data: pixels,
        }
    }
}

impl Deref for MandelImage {
    type Target = Vec<MandelPixel>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for MandelImage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// Returns a vector of one color for each given iteration number
/// TODO: make slice, no vec?
// fn colors() -> Vec<Color> {
//     let count = MAX_ITER + 1;
//     let mut c: Vec<Color> = Vec::with_capacity(count as usize);

//     for i in 0..count {
//         c.push(color(i as u32));
//     }

//     c
// }

/// Returns a color for a given number of iterations
fn color(n: u32) -> Color {
    if n < MAX_ITER {
        let ratio = n as f64 / (MAX_ITER - 1) as f64;
        let level = (ratio * 255.0) as u8;

        Color::RGB(0, level, 0)
    } else {
        Color::RGB(0, 0, 0)
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

/// Calculates the number of iterations for a given complex number
/// to "escape" the Mandelbrot set
fn mandel(c: &Complex<f64>) -> u32 {
    if in_set(&c) {
        MAX_ITER as u32
    } else {
        let f = |z| z * z + c;
        let mut iter = 0;
        let mut next = Complex::new(0.0, 0.0);

        while next.norm() < 2.0 && iter < MAX_ITER {
            next = f(next);
            iter += 1;
        }

        iter
    }
}

fn generate_image(transform: &Transform, image: &mut MandelImage) {
    let start = SystemTime::now();

    image
        .iter_mut()
        .for_each(|p| p.iterations = mandel(&transform.point_to_complex(&p.point)));

    println!("Generated image in: {:?}", start.elapsed().unwrap());
}

/// histogram equalization
fn equalize_image(image: &mut MandelImage) {
    let start = SystemTime::now();

    // count each iteration count
    const SIZE: usize = (MAX_ITER + 1) as usize;
    let mut iteration_counts = [0; SIZE];
    image
        .iter()
        .for_each(|p| iteration_counts[p.iterations as usize] += 1);

    let mut cumulative_distribution = [0; SIZE];

    // TODO: use iter
    // skip MAX_ITER (in set) in equalization
    let mut last = 0;
    for i in 0..MAX_ITER {
        cumulative_distribution[i as usize] = last + iteration_counts[i as usize];
        last = cumulative_distribution[i as usize];
    }

    // calc equalized array of iterations
    let sum: i32 = iteration_counts.iter().take(MAX_ITER as usize).sum();
    let mut adjusted = [0; SIZE as usize];
    let nominator = sum - cumulative_distribution[0];
    let hist = |n: u32| {
        ((cumulative_distribution[n as usize] - cumulative_distribution[0]) as f64
            / nominator as f64
            * (MAX_ITER - 1) as f64)
            .round() as u32
    };

    let equalized = |n| match n {
        MAX_ITER => MAX_ITER,
        _ => hist(n),
    };

    for i in 0..SIZE {
        adjusted[i] = equalized(i as u32);
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

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("MandelbRust", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let window_size = canvas.output_size()?;
    println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    println!("Windows size {:?}", window_size);

    let mut transform = Transform::new(window_size);
    let mut image = MandelImage::new(window_size.0, window_size.1);

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_target(
            texture_creator.default_pixel_format(),
            window_size.0,
            window_size.1,
        )
        .unwrap();

    let mut event_pump = sdl_context.event_pump()?;
    let mut update_image = true;
    let mut update_texture = true;
    let mut use_histogram = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Plus),
                    ..
                } => {
                    transform.zoom(2.0);
                    update_image = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    transform.zoom(0.5);
                    update_image = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    transform = Transform::new(window_size);
                    update_image = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::H),
                    ..
                } => {
                    use_histogram = !use_histogram;
                    update_texture = true;
                }
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    transform.center_at(&transform.pos_to_complex(x, y));
                    update_image = true;
                }
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn: MouseButton::Right,
                    ..
                } => {
                    let z = transform.pos_to_complex(x, y);
                    println!("{:?} iter: {}", z, mandel(&z));
                }
                _ => {}
            }
        }

        if update_image {
            generate_image(&transform, &mut image);
            equalize_image(&mut image);

            update_image = false;
            update_texture = true;
        }

        if update_texture {
            // select color function
            // TODO: use array instead of function
            let clr: Box<dyn Fn(&MandelPixel) -> Color> = match use_histogram {
                true => Box::new(|pix| color(pix.iterations_equalized)),
                false => Box::new(|pix| color(pix.iterations)),
            };
            draw_texture(&mut canvas, &mut texture, &image, clr);
            update_texture = false;
        }

        canvas.copy(&texture, None, None)?;
        canvas.present();

        thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(())
}

fn draw_texture<F>(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture: &mut sdl2::render::Texture<'_>,
    image: &MandelImage,
    color: F,
) where
    F: Fn(&MandelPixel) -> Color,
{
    let start = SystemTime::now();

    let _result = canvas.with_texture_canvas(texture, |texture_canvas| {
        image.iter().for_each(|pix| {
            texture_canvas.set_draw_color(color(&pix));
            texture_canvas
                .draw_point(pix.point)
                .expect("Failed to draw pixel");
        });
    });
    println!("Texture drawn in: {:?}", start.elapsed().unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transforms() {
        let transform = Transform::new((200, 300));
        let p = Point::new(100, 100);
        let z = transform.point_to_complex(&p);
        assert_eq!(p, transform._complex_to_point(z));
    }

    #[test]
    fn test_zoom() {
        let mut transform = Transform::new((200, 300));
        assert_eq!(transform.zoom_factor(), 1.0);

        transform.zoom(2.0);
        assert_eq!(transform.zoom_factor(), 2.0);

        transform.zoom(5.0);
        assert_eq!(transform.zoom_factor(), 10.0);

        transform.zoom(0.5);
        assert_eq!(transform.zoom_factor(), 5.0);
    }
}
