extern crate num;
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point};

use num::complex::Complex;
use std::time::SystemTime;

fn to_pix(z: Complex<f64>) -> Point {
    Point::new((z.re * 200.0 + 500.0) as i32, (z.im * 200.0 + 300.0) as i32)
}

fn to_complex(p: Point) -> Complex<f64> {
    Complex::new((p.x as f64 - 500.0) / 200.0, (p.y as f64 - 300.0) / 200.0)
}

fn _print_complex(z: Complex<f64>) {
    println!("z: {:?} -> {:?}", z, to_pix(z));
}

fn _print_point(p: Point) {
    println!("p: {:?} -> {:?}", p, to_complex(p));
}

fn colors() -> Vec<Color> {
    let mut c: Vec<Color> = Vec::with_capacity(256);

    for i in 0..256 {
        let n = (255 - i) as u8;
        c.push(Color::RGB(n, n, n));
    }

    c
}

fn mandel(p: Point) -> u32 {
    let c = to_complex(p);
    let f = |z| z * z + c;
    let mut iter = 0;
    let mut next = Complex::new(0.0, 0.0);

    for n in 0..255 {
        iter = n;
        next = f(next);
        if next.norm() > 2.0 {
            break;
        }
    }

    iter
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

    canvas.set_draw_color(Color::RGB(170, 190, 230));
    canvas.clear();
    canvas.present();

    ::std::thread::sleep(std::time::Duration::from_millis(1000));

    let start = SystemTime::now();

    let c = colors();
    let mut p = Point::new(0, 0);

    for x in 0..window_size.0 {
        for y in 0..window_size.1 {
            let n = mandel(p);
            canvas.set_draw_color(c[(n % 255) as usize]);
            p.x = x as i32;
            p.y = y as i32;
            let _e = canvas.draw_point(p);
        }
    }

    println!("Time: {:?}", start.elapsed().unwrap());
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // canvas.clear();
        // canvas.present();

        ::std::thread::sleep(std::time::Duration::from_millis(20));
    }

    Ok(())
}
