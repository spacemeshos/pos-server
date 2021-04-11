# gpu-post-bindings

## Building

Before building the project, you need to build the `post-gpu` library and copy the shared object to the root of the project. 
For macOS, the library file is `libgpu-setup.dylib`, for Linux it is `libgpu-setup.so`.
In windows, copy `gpu-setup.dll` and `gpu-setup.lib`.

```bash
RUSTFLAGS="-L /path/to/project/root" cargo build
```

## Running the Demo App

```bash
RUSTFLAGS="-L /path/to/project/root" cargo run
```
