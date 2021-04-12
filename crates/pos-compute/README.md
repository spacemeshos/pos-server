# gpu-post-bindings

A Rust create for using Spacemesh [gpu-post](http://github.com/spacemeshos/gpu-post) c-lib form Rust.

## Building

Build the [gpu-post](http://github.com/spacemeshos/gpu-post) c-library and copy the shared lib to the root of this project. 
On macOS, the library file is `libgpu-setup.dylib` and on Linux it is `libgpu-setup.so`.
On Windows, copy both `gpu-setup.dll` and `gpu-setup.lib` to this project root.

You need to add your project root directory to Cargo's linkable libs search path.

```bash
RUSTFLAGS="-L ./crates/gpu-post/resources" cargo build
```

## Running the Demo App


todo: automate step 1 using build.rs.

1. copy the gpu-post dlls to the target folder.

2.
```bash
RUSTFLAGS="-L ./creates/gpu-post/resources" cargo run
```
