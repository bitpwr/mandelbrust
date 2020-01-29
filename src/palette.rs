use sdl2::pixels::Color;

/// defined color scheme
#[derive(Copy, Clone)]
pub enum ColorScheme {
    Green,
    Rainbow,
    Redish,
    Blue,
}

/// Converts a HSV color to a SDL Color
/// hue [0..360], saturation [0..1], value [0..1]
pub fn hsv(h: f64, s: f64, v: f64) -> Color {
    let color = |r: f64, g: f64, b: f64| {
        Color::RGB((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    };

    if s <= 0.0 {
        return color(v, v, v);
    }

    let h = h % 360.0;
    let hh = h / 60.0;
    let region = hh as u32;
    let ff = hh - region as f64;

    let p = v * (1.0 - s);
    let q = v * (1.0 - (s * ff));
    let t = v * (1.0 - (s * (1.0 - ff)));

    match region {
        0 => color(v, t, p),
        1 => color(q, v, p),
        2 => color(p, v, t),
        3 => color(p, q, v),
        4 => color(t, p, v),
        _ => color(v, p, q),
    }
}

/// colors: rainbow
fn color_rainbow(n: u32, max: u32) -> Color {
    let v = match n {
        _ if n == max => 0.0,
        _ => 1.0,
    };

    hsv(300.0 * (n as f64 / max as f64), 1.0, v)
}

/// colors: blue - purple
fn color_blue(n: u32, max: u32) -> Color {
    let blue_limit = max / 3;
    if n < blue_limit {
        let ratio = n as f64 / blue_limit as f64;
        let level = (ratio.sqrt() * 255.0) as u8;

        Color::RGB(0, 0, level)
    } else {
        let v = match n {
            _ if n == max => 0.0,
            _ => 1.0,
        };

        let ratio = (n - blue_limit) as f64 / (max - blue_limit) as f64;
        hsv(240.0 + 60.0 * ratio, 1.0, v)
    }
}

/// colors: red - yellow
fn color_red(n: u32, max: u32) -> Color {
    let red_limit = max / 2;
    if n < red_limit {
        let ratio = n as f64 / red_limit as f64;
        let level = (ratio.sqrt() * 255.0) as u8;

        Color::RGB(level, 0, 0)
    } else {
        let v = match n {
            _ if n == max => 0.0,
            _ => 1.0,
        };

        let ratio = (n - red_limit) as f64 / (max - red_limit) as f64;
        hsv(60.0 * ratio, 1.0, v)
    }
}

/// colors: black - green - white
fn color_green(n: u32, max: u32) -> Color {
    if n < max {
        let ratio = n as f64 / (max - 1) as f64;
        let level = (ratio.sqrt() * 255.0) as u8;

        let mut rb = 0;
        if ratio > 0.5 {
            rb = ((ratio - 0.5) / 0.5 * 180.0) as u8;
        }

        Color::RGB(rb, level, rb)
    } else {
        Color::RGB(0, 0, 0)
    }
}

pub fn color(color_type: ColorScheme, n: u32, max: u32) -> Color {
    match color_type {
        ColorScheme::Green => color_green(n, max),
        ColorScheme::Rainbow => color_rainbow(n, max),
        ColorScheme::Redish => color_red(n, max),
        ColorScheme::Blue => color_blue(n, max),
    }
}

// Returns a vector of one color for each given iteration number
// fn colors(count: u32) -> Vec<Color> {
//     let mut c: Vec<Color> = Vec::with_capacity((count + 1) as usize);

//     for i in 0..=count {
//         c.push(color_green(i as u32, count));
//     }

//     c
// }
