# Pos Server

A server for creating Spacemesh proof of space data files using one or more supported system gpus.
The service provides a grpc service for clients to configure it, submit pos jobs and get jobs execution progress.
Reuqested client jobs are executed using the system's available supported gpus. 
To run the server, you need to have at least one supported gpu or set the `use_cpu_provider` config param to true to use the system's cpu. This is not recommended to production, only for testing.

## Prerequisites

You must include a gpu-post c-library for the platform you are building this project on.
1. Build the [gpu-post](http://github.com/spacemeshos/gpu-post) c-library
2. Copy the shared library file(s) to the `./pos-compute/resources` directory. On macOS, the library file name is `libgpu-setup.dylib` and on Linux it is `libgpu-setup.so`.
On Windows, copy both `gpu-setup.dll` and `gpu-setup.lib`.

## Building

```bash
make
```

## Testing
Copy all file from `crates/pos-compute/resources/` to `target/debug/`.

```bash
make test
```

## Running
1. Copy all gpu-lib artifacts to `pos-service` executable directory.
1. Add the gpu-post dynamic lib path to your system's dynamic lib path. macOS: `export DYLD_LIBRARY_PATH=.:$DYLD_LIBRARY_PATH`. linux: use LD_LIBRARY_PATH.
1. Execute the `pos-service` process.
1. Use any GRPC client to connect to the server's GRPC service.
1. Call the [service's methods](https://github.com/spacemeshos/pos-server/blob/main/crates/pos-api/proto/pos_api_service/api.proto) from your client.

---

## Design
- The core pos data computation is done by the [gpu-post](https://github.com/spacemeshos/gpu-post) c library. The library is wrapped as a Rust library module for access from other Rust modules in the `pos-compute` crate.
- The grpc server is implemented using [Tonic](https://github.com/hyperium/tonic) in the `pos-api` crate.
- The server is implemented as an [xactor](https://github.com/sunli829/xactor) system service actor in the `pos-service` create to provide safe read/write to server state from tasks.
- The server uses the [tokio runtime](https://github.com/tokio-rs/tokio) for tasks execution. Each task is spawned as a blocking tokio task as the gpu-post c lib is a blocking i/o library.
