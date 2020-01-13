use num::complex::Complex;

use sdl2::rect::Point;

use std::ops::{Deref, DerefMut};
use std::time::SystemTime;

/// Transforms to/from pixels and complex numbers
pub struct Transform {
    x: f64,
    y: f64,
    scale: f64,
    window_size: (u32, u32),
}

impl Transform {
    pub fn new(window_size: (u32, u32)) -> Self {
        let mut t = Transform {
            scale: 1.0,
            x: 0.0,
            y: 0.0,
            window_size,
        };
        t.reset();
        t
    }

    pub fn reset(&mut self) {
        self.scale = self.window_size.0 as f64 * 0.28;
        self.x = self.window_size.0 as f64 * 0.7;
        self.y = self.window_size.1 as f64 * 0.5;
    }

    pub fn point_to_complex(&self, p: &Point) -> Complex<f64> {
        self.pos_to_complex(p.x, p.y)
    }

    // TODO: Remove
    pub fn pos_to_complex(&self, x: i32, y: i32) -> Complex<f64> {
        Complex::new(
            (x as f64 - self.x) / self.scale,
            (y as f64 - self.y) / self.scale,
        )
    }

    pub fn _complex_to_point(&self, z: Complex<f64>) -> Point {
        Point::new(
            (z.re * self.scale + self.x) as i32,
            (z.im * self.scale + self.y) as i32,
        )
    }

    pub fn zoom(&mut self, factor: f64) {
        let z_center = self.pos_to_complex(
            (self.window_size.0 / 2) as i32,
            (self.window_size.1 / 2) as i32,
        );
        self.scale *= factor;
        self.x = self.window_size.0 as f64 / 2.0 - z_center.re * self.scale;
        self.y = self.window_size.1 as f64 / 2.0 - z_center.im * self.scale;

        println!("Zoom: {}", self.zoom_factor());
    }

    pub fn zoom_factor(&self) -> f64 {
        self.scale / (self.window_size.0 as f64 * 0.28)
    }

    pub fn center_at(&mut self, z: &Complex<f64>) {
        self.x = self.window_size.0 as f64 / 2.0 - z.re * self.scale;
        self.y = self.window_size.1 as f64 / 2.0 - z.im * self.scale;
    }
}

/// Information for each pixel in MandelImage
#[derive(Clone)]
pub struct MandelPixel {
    pub point: Point,
    pub iterations: u32,
    pub iterations_equalized: u32,
}

impl MandelPixel {
    pub fn new(x: i32, y: i32) -> Self {
        MandelPixel {
            point: Point::new(x, y),
            iterations: 0,
            iterations_equalized: 0,
        }
    }
}

/// Generated image data for the Mandelbrot set
pub struct MandelImage {
    pub width: u32,
    pub height: u32,
    pub max_iterations: u32,
    data: Vec<MandelPixel>,
}

impl MandelImage {
    pub fn new(width: u32, height: u32, max_iterations: u32) -> Self {
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
            max_iterations,
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

/// Keeps the draw settings
pub struct DrawSettings {
    pub run: bool,
    pub update_image: bool,
    pub update_texture: bool,
    pub use_histogram: bool,
}

impl DrawSettings {
    pub fn new() -> Self {
        DrawSettings {
            run: true,
            update_image: true,
            update_texture: true,
            use_histogram: false,
        }
    }
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
