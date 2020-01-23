# MandelbRust

An appliction showing the classic Mandelbrot set implemented in Rust with SDL2.

## Requirements

First install these requirements.

* `SDL2-devel`

## Usage

The release version performs much better, so run

```
cargo run --release
```

Keyboard shortcuts and mouse functions.

* `+` and `-` keys zooms in and out.
* Left mouse button sets image center.
* `PageUp` and `PageDown` changes maximum interation count.
* `H` toggles histogram equalization.
* `C` shows the available color schemes.
* `Num keys` selects color schemes.
* `Space` resets the zoom level.
* Right mouse button prints pixel information to console
* `Esc` stops the program.

