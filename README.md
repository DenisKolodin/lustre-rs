# lustre-rs: Toy RT Renderer

[![Rust CI](https://github.com/nbarrios1337/lustre-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/nbarrios1337/lustre-rs/actions/workflows/rust.yml)

Learning Rust via [Peter Shirley's Ray Tracing in One Weekend](https://raytracing.github.io/) Book series and other sources.

![a modified cover photo scene of Peter Shirley's Ray Tracing in One Weekend book, rendered at 10,000 samples per pixel at about 4k resolution](images/big.png)

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
