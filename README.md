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

```bash
make test
```

## Using
1. Configure and deploy the server on a system with one or more supported compute providers.
1. Use any GRPC client to connect to the server's GRPC service.
1. Call the [service's methods]((https://github.com/spacemeshos/pos-server/blob/main/crates/pos-api/proto/pos_api_service/api.proto) from your client.

