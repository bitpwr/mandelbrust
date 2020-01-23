extern crate num;
extern crate sdl2;

mod mandelbrot;
mod palette;
mod types;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::Window;

use std::thread;
use std::time::SystemTime;

use palette::ColorScheme;
use types::MandelImage;
use types::MandelPixel;
use types::Transform;

/// Keeps the draw settings
struct DrawSettings {
    run: bool,
    update_image: bool,
    update_texture: bool,
    use_histogram: bool,
    show_colors: bool,
    color_scheme: ColorScheme,
}

impl DrawSettings {
    fn new() -> Self {
        DrawSettings {
            run: true,
            update_image: true,
            update_texture: true,
            use_histogram: false,
            show_colors: false,
            color_scheme: ColorScheme::Green,
        }
    }
}

/// Owns SDL objects
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

/// Defined the different user events
enum MandelEvent {
    Idle,
    Quit,
    Zoom(f64),
    Scheme(ColorScheme),
    Center(i32, i32),
    ChangeIterations(f64),
    ToggleHistogram,
    ToggleColorschemes,
    ShowInfo(i32, i32),
}

fn get_event(event_pump: &mut sdl2::EventPump) -> MandelEvent {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return MandelEvent::Quit,
            Event::KeyDown {
                keycode: Some(Keycode::Plus),
                ..
            } => {
                return MandelEvent::Zoom(2.0);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Minus),
                ..
            } => {
                return MandelEvent::Zoom(0.5);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                return MandelEvent::Zoom(0.0);
            }
            Event::KeyDown {
                keycode: Some(Keycode::H),
                ..
            } => {
                return MandelEvent::ToggleHistogram;
            }
            Event::KeyDown {
                keycode: Some(Keycode::C),
                ..
            } => {
                return MandelEvent::ToggleColorschemes;
            }
            Event::KeyDown {
                keycode: Some(Keycode::PageUp),
                ..
            } => {
                return MandelEvent::ChangeIterations(2.0);
            }
            Event::KeyDown {
                keycode: Some(Keycode::PageDown),
                ..
            } => {
                return MandelEvent::ChangeIterations(0.5);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num1),
                ..
            } => {
                return MandelEvent::Scheme(ColorScheme::Green);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num2),
                ..
            } => {
                return MandelEvent::Scheme(ColorScheme::Rainbow);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num3),
                ..
            } => {
                return MandelEvent::Scheme(ColorScheme::Redish);
            }
            Event::KeyDown {
                keycode: Some(Keycode::Num4),
                ..
            } => {
                return MandelEvent::Scheme(ColorScheme::Nice);
            }
            Event::MouseButtonDown {
                x,
                y,
                mouse_btn: MouseButton::Left,
                ..
            } => {
                return MandelEvent::Center(x, y);
            }
            Event::MouseButtonDown {
                x,
                y,
                mouse_btn: MouseButton::Right,
                ..
            } => {
                return MandelEvent::ShowInfo(x, y);
            }
            _ => {}
        }
    }

    MandelEvent::Idle
}

pub fn main() -> Result<(), String> {
    let mut image = MandelImage::new(800, 600, 150);
    let mut transform = Transform::new((image.width, image.height));
    let mut settings = DrawSettings::new();
    let mut sdl = setup_sdl(image.width, image.height)?;

    let mut mandel_texture = sdl
        .texture_creator
        .create_texture_target(
            sdl.texture_creator.default_pixel_format(),
            image.width,
            image.height,
        )
        .expect("Failed to create mandel texture");
    let mut color_texture = sdl
        .texture_creator
        .create_texture_target(
            sdl.texture_creator.default_pixel_format(),
            image.width,
            image.height,
        )
        .expect("Failed to create color texture");

    draw_color_texture(&mut sdl.canvas, &mut color_texture);

    while settings.run {
        let e = get_event(&mut sdl.event_pump);
        match e {
            MandelEvent::Quit => settings.run = false,
            MandelEvent::Zoom(factor) => {
                if factor > 0.0 {
                    transform.zoom(factor);
                } else {
                    transform.reset();
                    image.max_iterations = 150;
                }
                settings.update_image = true;
            }
            MandelEvent::Center(x, y) => {
                transform.center_at(&transform.pos_to_complex(x, y));
                settings.update_image = true;
            }
            MandelEvent::Scheme(scheme) => {
                settings.color_scheme = scheme;
                settings.update_texture = true;
            }
            MandelEvent::ToggleHistogram => {
                settings.use_histogram = !settings.use_histogram;
                settings.update_texture = true;
            }
            MandelEvent::ToggleColorschemes => {
                settings.show_colors = !settings.show_colors;
            }
            MandelEvent::ChangeIterations(factor) => {
                image.max_iterations = (image.max_iterations as f64 * factor).round() as u32;
                settings.update_image = true;
            }
            MandelEvent::ShowInfo(x, y) => {
                let z = transform.pos_to_complex(x, y);
                println!(
                    "Comples: [{}, {}i], iterations: {}",
                    z.re,
                    z.im,
                    image.iterations(x, y)
                );
            }
            _ => {}
        }

        if settings.update_image {
            mandelbrot::generate_image(&transform, &mut image);
            mandelbrot::equalize_image(&mut image);

            settings.update_image = false;
            settings.update_texture = true;
        }

        if settings.update_texture {
            // select color function
            // TODO: use array instead of function
            let color: Box<dyn Fn(&MandelPixel) -> Color> = match settings.use_histogram {
                true => Box::new(|pix| {
                    palette::color(
                        settings.color_scheme,
                        pix.iterations_equalized,
                        image.max_iterations,
                    )
                }),
                false => Box::new(|pix| {
                    palette::color(settings.color_scheme, pix.iterations, image.max_iterations)
                }),
            };
            draw_texture(&mut sdl.canvas, &mut mandel_texture, &image, color);
            settings.update_texture = false;
        }

        let texture = match settings.show_colors {
            true => &color_texture,
            false => &mandel_texture,
        };
        sdl.canvas.copy(texture, None, None)?;
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

    canvas
        .with_texture_canvas(texture, |texture_canvas| {
            image.iter().for_each(|pix| {
                texture_canvas.set_draw_color(color(&pix));
                texture_canvas
                    .draw_point(pix.point)
                    .expect("Failed to draw pixel");
            });
        })
        .expect("Failed to draw texture");
    println!("Texture drawn in: {:?}", start.elapsed().unwrap());
}

fn draw_color_texture(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture: &mut sdl2::render::Texture<'_>,
) {
    let start = SystemTime::now();

    let (width, _) = canvas.output_size().unwrap();
    let bar_height = 100;
    let draw_rect =
        |can: &mut sdl2::render::Canvas<sdl2::video::Window>, x: u32, y: u32, s: ColorScheme| {
            can.set_draw_color(palette::color(s, x, width));
            can.draw_rect(Rect::new(x as i32, y as i32, 1, bar_height))
                .expect("Failed to draw pixel");
        };

    canvas
        .with_texture_canvas(texture, |mut texture_canvas| {
            for x in 0..width {
                draw_rect(&mut texture_canvas, x, 0, ColorScheme::Green);
                draw_rect(&mut texture_canvas, x, bar_height, ColorScheme::Rainbow);
                draw_rect(&mut texture_canvas, x, bar_height * 2, ColorScheme::Redish);
                draw_rect(&mut texture_canvas, x, bar_height * 3, ColorScheme::Nice);
            }
        })
        .expect("Failed to draw texture");

    println!("Color texture drawn in: {:?}", start.elapsed().unwrap());
}
