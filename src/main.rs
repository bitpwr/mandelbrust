extern crate num;
extern crate sdl2;
mod mandelbrot;
mod types;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::video::Window;

use std::thread;
use std::time::SystemTime;

use types::DrawSettings;
use types::MandelImage;
use types::MandelPixel;
use types::Transform;

/// Returns a color for a given number of iterations
fn color(n: u32, max: u32) -> Color {
    if n < max {
        let ratio = n as f64 / (max - 1) as f64;
        let level = (ratio * 255.0) as u8;

        Color::RGB(0, level, 0)
    } else {
        Color::RGB(0, 0, 0)
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

struct Sdl {
    canvas: sdl2::render::Canvas<Window>,
    event_pump: sdl2::EventPump,
    texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext>,
}

fn setup_sdl(width: u32, height: u32) -> Result<Sdl, String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("MandelbRust", width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let event_pump = sdl_context.event_pump()?;
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    println!("Using SDL_Renderer \"{}\"", canvas.info().name);
    println!("Windows size {:?}", canvas.output_size()?);

    Ok(Sdl {
        canvas,
        event_pump,
        texture_creator,
    })
}

fn poll_events(
    event_pump: &mut sdl2::EventPump,
    settings: &mut DrawSettings,
    transform: &mut Transform,
    max_iter: u32,
) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => {
                settings.run = false;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Plus),
                ..
            } => {
                transform.zoom(2.0);
                settings.update_image = true;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Minus),
                ..
            } => {
                transform.zoom(0.5);
                settings.update_image = true;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                transform.reset();
                settings.update_image = true;
            }
            Event::KeyDown {
                keycode: Some(Keycode::H),
                ..
            } => {
                settings.use_histogram = !settings.use_histogram;
                settings.update_texture = true;
            }
            Event::MouseButtonDown {
                x,
                y,
                mouse_btn: MouseButton::Left,
                ..
            } => {
                transform.center_at(&transform.pos_to_complex(x, y));
                settings.update_image = true;
            }
            Event::MouseButtonDown {
                x,
                y,
                mouse_btn: MouseButton::Right,
                ..
            } => {
                let z = transform.pos_to_complex(x, y);
                println!(
                    "Complex: [{}, {}i], iterations: {}",
                    z.re,
                    z.im,
                    mandelbrot::mandel(&z, max_iter)
                );
            }
            _ => {}
        }
    }
}

pub fn main() -> Result<(), String> {
    let mut image = MandelImage::new(800, 600, 150);
    let mut transform = Transform::new((image.width, image.height));
    let mut settings = DrawSettings::new();
    let mut sdl = setup_sdl(image.width, image.height)?;
    let mut texture = sdl
        .texture_creator
        .create_texture_target(
            sdl.texture_creator.default_pixel_format(),
            image.width,
            image.height,
        )
        .expect("Failed to create texture");

    while settings.run {
        poll_events(
            &mut sdl.event_pump,
            &mut settings,
            &mut transform,
            image.max_iterations,
        );

        if settings.update_image {
            mandelbrot::generate_image(&transform, &mut image);
            mandelbrot::equalize_image(&mut image);

            settings.update_image = false;
            settings.update_texture = true;
        }

        if settings.update_texture {
            // select color function
            // TODO: use array instead of function
            let clr: Box<dyn Fn(&MandelPixel) -> Color> = match settings.use_histogram {
                true => Box::new(|pix| color(pix.iterations_equalized, image.max_iterations)),
                false => Box::new(|pix| color(pix.iterations, image.max_iterations)),
            };
            draw_texture(&mut sdl.canvas, &mut texture, &image, clr);
            settings.update_texture = false;
        }

        sdl.canvas.copy(&texture, None, None)?;
        sdl.canvas.present();

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
