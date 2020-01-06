extern crate num;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Point;

use num::complex::Complex;
use std::thread;
use std::time::SystemTime;

const MAX_ITER: u32 = 200;

/// Transforms to/from pixels and complex numbers
/// TODO: Use a create with affine transformations instead
struct Transform {
    x: f64,
    y: f64,
    scale: f64,
    window_size: (u32, u32),
}

/// Information for each pixel in MandelImage
#[derive(Clone)]
struct MandelPixel {
    iterations: u32,
    color: Color,
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

    fn _point_to_complex(&self, p: &Point) -> Complex<f64> {
        self.pos_to_complex(p.x, p.y)
    }

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
    fn new() -> Self {
        MandelPixel {
            iterations: 0,
            color: Color::RGB(200, 100, 100),
        }
    }
}

impl MandelImage {
    fn new(width: u32, height: u32) -> Self {
        MandelImage {
            width,
            height,
            data: vec![MandelPixel::new(); (width * height) as usize],
        }
    }

    // fn get(&self, x: u32, y: u32) -> &MandelPixel {
    //     &self.data[(x + y * self.width) as usize]
    // }

    fn set_iterations(&mut self, x: u32, y: u32, iterations: u32) {
        self.data[(x + y * self.width) as usize].iterations = iterations;
    }

    fn set_color(&mut self, x: u32, y: u32, color: Color) {
        self.data[(x + y * self.width) as usize].color = color;
    }

    fn color(&self, x: u32, y: u32) -> Color {
        return self.data[(x + y * self.width) as usize].color;
    }
}

/// Returns a vector of one color for each given iteration number
/// TODO: make slice, no vec?
fn colors() -> Vec<Color> {
    let mut c: Vec<Color> = Vec::with_capacity(MAX_ITER as usize);

    for i in 0..MAX_ITER {
        c.push(color(i as u32));
    }

    c
}

/// Returns a color for a given number of iterations
fn color(n: u32) -> Color {
    if n < (MAX_ITER - 1) {
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
    let f = |z| z * z + c;
    let mut iter = 0;

    if in_set(&c) {
        iter = MAX_ITER - 1;
    } else {
        let mut next = Complex::new(0.0, 0.0);

        while next.norm() < 2.0 && iter < (MAX_ITER - 1) {
            next = f(next);
            iter += 1;
        }
    }

    iter
}

fn generate_image(transform: &Transform, image: &mut MandelImage) -> Result<(), String> {
    let start = SystemTime::now();

    let c = colors();

    for x in 0..image.width {
        for y in 0..image.height {
            let z = transform.pos_to_complex(x as i32, y as i32);
            let n = mandel(&z);
            image.set_iterations(x, y, n);
            image.set_color(x, y, c[n as usize])
        }
    }
    println!("Generated image in: {:?}", start.elapsed().unwrap());

    Ok(())
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
    let mut update = true;

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
                    update = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    transform.zoom(0.5);
                    update = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    transform = Transform::new(window_size);
                    update = true;
                }
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    transform.center_at(&transform.pos_to_complex(x, y));
                    update = true;
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

        if update {
            generate_image(&transform, &mut image)?;

            // draw image to texture
            // TODO: How to move this to a function? (lifetimes)
            let _result = canvas.with_texture_canvas(&mut texture, |texture_canvas| {
                for x in 0..image.width {
                    for y in 0..image.height {
                        let p = Point::new(x as i32, y as i32);
                        texture_canvas.set_draw_color(image.color(x, y));
                        texture_canvas.draw_point(p).expect("Failed to draw pixel");
                    }
                }
            });

            update = false;
        }

        canvas.copy(&texture, None, None)?;
        canvas.present();

        thread::sleep(std::time::Duration::from_millis(50));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transforms() {
        let transform = Transform::new((200, 300));
        let p = Point::new(100, 100);
        let z = transform._point_to_complex(&p);
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
