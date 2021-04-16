# pos-compute

A Rust create for using Spacemesh [gpu-post](http://github.com/spacemeshos/gpu-post) c-lib form Rust.

## Building

Build the [gpu-post](http://github.com/spacemeshos/gpu-post) c-library and copy the shared lib to the root of this project. 
On macOS, the library file is `libgpu-setup.dylib` and on Linux it is `libgpu-setup.so`.
On Windows, copy both `gpu-setup.dll` and `gpu-setup.lib` to this project root.

You need to add your project root directory to Cargo's linkable libs search path.

```bash
RUSTFLAGS="-L ./crates/pos-compute/resources" cargo build
```

## Running the Demo App

1. Copy the gpu-post dlls to the resources folder.

2.
```bash
RUSTFLAGS="-L ./creates/pos-compute/resources" cargo run
```
