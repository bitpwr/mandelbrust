extern crate num;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Point;

use num::complex::Complex;
use std::time::SystemTime;

const MAX_ITER: u32 = 200;

#[derive(Copy, Clone)]
struct Mapping {
    x: f64,
    y: f64,
    scale: f64,
}

impl Mapping {
    fn reset(&mut self, window_size: (u32, u32)) {
        self.scale = window_size.0 as f64 * 0.28;
        self.x = window_size.0 as f64 * 0.7;
        self.y = window_size.1 as f64 * 0.5;
    }

    fn zoom(&self, window_size: (u32, u32)) -> f64 {
        self.scale / (window_size.0 as f64 * 0.28)
    }
}

// fn to_pix(z: &Complex<f64>) -> Point {
//     Point::new((z.re * 200.0 + 500.0) as i32, (z.im * 200.0 + 300.0) as i32)
// }

fn to_complex(map: Mapping, p: Point) -> Complex<f64> {
    Complex::new(
        (p.x as f64 - map.x) / map.scale,
        (p.y as f64 - map.y) / map.scale,
    )
}

fn colors() -> Vec<Color> {
    let mut c: Vec<Color> = Vec::with_capacity(MAX_ITER as usize);

    for i in 0..MAX_ITER {
        c.push(color(i as u32));
    }

    c
}

fn color(n: u32) -> Color {
    if n < (MAX_ITER - 1) {
        let ratio = n as f64 / (MAX_ITER - 1) as f64;
        let level = (ratio * 255.0) as u8;

        Color::RGB(0, level, 0)
    } else {
        Color::RGB(0, 0, 0)
    }
}

// check if z is definitely within the Mandelbort set
// according to wikipedia
fn in_set(z: Complex<f64>) -> bool {
    let p = ((z.re - 0.25).powi(2) + z.im.powi(2)).sqrt();

    if z.re <= (p - 2.0 * p.powi(2) + 0.25) {
        return true;
    } else if ((z.re + 1.0).powi(2) + z.im.powi(2)) <= 0.0625 {
        return true;
    }

    false
}

fn mandel(c: Complex<f64>) -> u32 {
    let f = |z| z * z + c;
    let mut iter = 0;

    if in_set(c) {
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

fn draw(map: Mapping, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    let start = SystemTime::now();

    let window_size = canvas.output_size().unwrap();
    let c = colors();

    for x in 0..window_size.0 {
        for y in 0..window_size.1 {
            let p = Point::new(x as i32, y as i32);
            let z = to_complex(map, p);
            let n = mandel(z);
            canvas.set_draw_color(c[n as usize]);
            let _e = canvas.draw_point(p);
        }
    }

    println!("Time: {:?}", start.elapsed().unwrap());
}

fn zoom(map: &mut Mapping, window_size: (u32, u32), factor: f64) {
    let z_center = to_complex(
        *map,
        Point::new((window_size.0 / 2) as i32, (window_size.1 / 2) as i32),
    );
    map.scale *= factor;
    map.x = window_size.0 as f64 / 2.0 - z_center.re * map.scale;
    map.y = window_size.1 as f64 / 2.0 - z_center.im * map.scale;

    println!("Zoom: {}", map.zoom(window_size));
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

    println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    let window_size = canvas.output_size()?;
    println!("Windows size {:?}", window_size);

    let mut map = Mapping {
        x: 500.0,
        y: 300.0,
        scale: 200.0,
    };
    map.reset(window_size);

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

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
                    zoom(&mut map, window_size, 1.5);
                    update = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Minus),
                    ..
                } => {
                    zoom(&mut map, window_size, 0.66);
                    update = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    map.reset(window_size);
                    update = true;
                }
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn: MouseButton::Left,
                    ..
                } => {
                    let z = to_complex(map, Point::new(x, y));
                    println!("New center {:?}", z);
                    map.x = window_size.0 as f64 / 2.0 - z.re * map.scale;
                    map.y = window_size.1 as f64 / 2.0 - z.im * map.scale;
                    update = true;
                }
                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn: MouseButton::Right,
                    ..
                } => {
                    let z = to_complex(map, Point::new(x, y));
                    println!("{:?} iter: {}", z, mandel(z));
                }
                _ => {}
            }
        }

        if update {
            draw(map, &mut canvas);
            update = false;
            canvas.present();
        }

        ::std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transforms() {
        // let p = Point::new(100,100);
        // let z = to_complex(p);
        // assert_eq!(p, to_pix(&z));

        // let mut z2 = Complex::new(0.0, 0.0);
        // to_complex2(&p, &mut z2);
        // assert_eq!(z, z2);
    }
}
