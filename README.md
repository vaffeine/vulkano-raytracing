# [WIP] vulkano-raytracing

Real-time interactive GPU ray-tracer, created using awesome [vulkano](https://vulkano.rs/).
It is currently in very early stage of development.

## Build

To build this project you need:
- Rust nightly toolchain (just use [rustup](https://www.rustup.rs/) if you
  don't have Rust yet)
- Vulkan drivers for your GPU
- (Optional) [LunarG SDK](https://www.lunarg.com/vulkan-sdk/) for
  validation and debug

Then comes the moment you understand you love Rust so much.
```bash
cargo build
```
And you are done!


## Usage

[clap](https://clap.rs/) provides user with excelent help message:
```bash
USAGE:
    tracer [FLAGS] [OPTIONS] <model>

FLAGS:
        --benchmark  Turn on benchmarking
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -r, --resolution <width> <height>
            Sets the resolution of the image [default: 640 480]
    -p, --position <x> <y> <z>
            Sets the position of camera [default: 0.0 0.0 5.0]
        --fov <horizontal> <vertical>
            Sets the field of view [default: 40.0 40.0]
        --sensitivity <sensitivity>
            Sets the sensitivity of the controls (camera movement) [default: 1.0]
        --fps-update-interval <fps-update-interval>
            Sets the interval (in milliseconds) of FPS update.
            Displayed FPS is the average in the last interval [default: 100]
        --log-level <log-level>
            Sets the log messages amount [default: perf]
            [values: none, error, warning, perf, info, debug]

ARGS:
    <model>    Sets the path to file with model to render
```
Basicly, you just want to run
```bash
cargo run -- /path/to/model.obj
```
There are some models in assets folder that you can use to get the idea.
But it should work with any Wavefront model you want. (You want to
run release build with good GPU to render big models, though).

Then the window will open and display the passed model. You can control the camera using
keyboard and mouse. Use WASD or arrow keys to move around, Shift and Ctrl to
move up and down, and mouse to rotate the camera. You can see current FPS and
camera position+rotation in the left-top corner.

## Development

It is my own pet-project, that I develop just for fun. But help is highly
appriciated, especially if you are a Vulkan/Vulkano/raytracing expert, 'cause I'm not.
Feel free to contact me, create an issue or pull request.
But make sure to run `cargo fmt` before creating pull request.

## Roadmap

This is probably a subject to change.

**v0.1** (aka baby-tracer)
- [x] Single triangle rendering
- [x] User-controllable pinhole camera model
- [x] Wavefront model support

**v0.2** (acceleration time!)
- [  ] Single-level uniform grid
- [  ] Two-level grid

**v0.3** (stop hurting eyes)
- [  ] Basic shading (shadows, refraction, reflection)
- [  ] Depth of field and motion blur

**v0.4** (OMG, it's moving)
- [  ] Animated glTF models support

**v0.5** (adult swim)
- [  ] Unidirectional path-tracing
- [  ] Anti-aliasing

**v0.x** (*scary future that needs more research*)
- [  ] Better path-tracing algorithm
- [  ] Better acceleration structure
- [  ] Denoising
- [  ] Some mindblowing hacks
- [  ] Load-balancing
- [  ] Ambient occlusion
...and more

## Links

- According to this awesome [video](https://www.youtube.com/watch?v=JSr6wvkvgM0)
by Matt Swoboda, there are such ray-per-pixel (rpp) requirements per feature.
(30FPS 720p picture, scene with ~10'000 triangles)
    - Camera rays: 1rpp
    - Reflection / refraction: 1rpp per bounce
    - Accurate hard shadows: 1rpp per light
    - Soft shadows: 10-200 rpp per light
    - Path-tracing: 500-1000 rpp
    - Ambient occlusion: looks good at 100-500 rpp
- [Great paper](https://www.kalojanov.com/data/gpu_grid_construction.pdf) with
  uniform grid construction algorithm by Javor Kalojanov and Philipp Slusallek.
- [Another great paper](https://www.kalojanov.com/data/two_level_grids.pdf) by
  Javor Kalojanov togeather with Markus Billeter and Philipp Slusallek, that
  describes two-level grid structure and uses unifor grid algorithm for
  top-level cells. Will probably be used for v0.2

