# lustre-rs: Toy RT Renderer

[![Rust CI](https://github.com/nbarrios1337/lustre-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/nbarrios1337/lustre-rs/actions/workflows/rust.yml)

Learning Rust via [Peter Shirley's Ray Tracing in One Weekend](https://raytracing.github.io/) Book series and other sources.

## Usage

1. If you don't have Rust installed, take a look at [Rust's Getting Started page](https://www.rust-lang.org/learn/get-started).
2. Clone this repository:

    ```shell
    git clone git@github.com:nbarrios1337/lustre-rs.git
    ```

3. Build `lustre` by running:

    ```shell
    cargo build --release
    ```

4. Run `lustre` by specifying a scene to render:

   ```shell
   ./target/release/lustre -s cover-photo
   ```

   See `lustre --help` for more options.

## Progress

- [x] Implementing Book 1: [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html) - 100%
- [x] Documenting Book 1 implementation - 100%
- [x] Implementing Book 2: [Ray Tracing: The Next Week](https://raytracing.github.io/books/RayTracingTheNextWeek.html) - 100%
- [x] Documenting Book 2 implementation - 100%
- [ ] Implementing Book 3: [Ray Tracing: The Rest of Your Life](https://raytracing.github.io/books/RayTracingTheRestOfYourLife.html)
- [ ] Documenting Book 3 implementation
- [ ] Look into other ways to expand this renderer. Possibilties:
  - Integration with shaders
  - Integration with graphics APIs
  - Rendering in realtime
  - ...

## Additional Sources

- [Peter Shirley's "In One Weekend" Blog](https://in1weekend.blogspot.com/), serving as addendums to his aforementioned book series.
- [Pharr's, Jakob's, and Humphrey's "Physically Based Rendering: From Theory to Implementation"](https://pbr-book.org/): incorporated some of the acceleration structure ideas for the Tree module.

## Examples

Below are some resulting images from lustre:

The Cornell Box, rendered at 2k x 2k with 10k samples per pixel:
![A render of the famous Cornell Box scene, rendered at 10,000 samples at a 2160 by 2160 resolution](images/cornell.png)

A modified (in the dark with lights!) version of the RT in One Weekend cover photo, rendered at 4k with 5k samples per pixel:
![A render of a modified RT in One Weekend cover photo, rendered at 5000 samples at a 4k resolution](images/lights.png)

A slightly altered version of the final scene from [Book 2 of RT in One Weekend](https://raytracing.github.io/books/RayTracingTheNextWeek.html), rendered at 4k with 10k samples per pixel (this took ~27 hours to render!):
![A render of the final scene in Ray Tracing: The Next Week, rendered at 10000 samples at a 4k resolution](images/final-scene.png)
