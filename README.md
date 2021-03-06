# tinygl

[![Build](https://github.com/vtavernier/tinygl/workflows/Build/badge.svg?branch=master)](https://github.com/vtavernier/tinygl/actions)

tinygl is an environment to create OpenGL programs in Rust with:

* Pre-processing of GLSL shader code (#include support, syntax checking, etc.), using [shaderc](https://github.com/google/shaderc-rs/)
* Conversion of GLSL shaders to SPIR-V or transpilation to GLSL ES for WebGL, also using shaderc
* Rust code generation for loading shaders, programs and type-checked uniform setter methods

This is a project currently under heavy development, do not expect any kind of stability for a while.

## Building

* (Optional) Re-generate OpenGL desktop bindings using `cargo xtask gen-bindings`
* `cargo build`

## Authors

Vincent Tavernier <vince.tavernier@gmail.com>

## License

Licensed under MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT).
