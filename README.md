# Dragknife repath
CNC dragknives trail behind the router like the back wheel of a bicycle. This tool can be used to transform gcode toolpaths which show the intended back-wheel movement to the corresponding front-wheel movement. This involves adding swivel movements at sharp corners and slightly adjusting the lines and arcs which make up the toolpath.

## Features & Limitations
* Works with `G0-3,28` movement commands.
* Works on different planes (`G17-19`), but assumes the plane doesn't change halfway (i.e. there is only one `G17-19` near the start).
* Works with switching between mm and inches (`G20,21`).
* Ignores coordinate system commands `G54-G59`. If they are only near the front, the output _should_ still make sense.
* Should work with relative positioning (`G90`) but only tested with absolute positioning (`G91`).
* Unknown commands are just copied to the output.

In general, I suggest viewing the output with a [site like this](https://ncviewer.com).

## Building
To build for your system, install Rust and Git and run
```
git clone https://github.com/evnor/dragknife-repath.git
cd dragknife-repath.git
cargo build --release
```
The output should be in `/target/release/`

### TODO
* Separate the GCode fixing and the egui stuff
* Add tests
* Look at which parts of `types.rs`, `lib.rs` and `vec3.rs` should be `pub`