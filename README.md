# pos server

A server for creating Spacemesh proof of space data using one or more supported system gpus.
Providers a grpc service to clients to configure the service, start pos jobs and get jobs progress.
Reuqested client jobs are executed on the system available supported gpus.

## Building

Build the [gpu-post](http://github.com/spacemeshos/gpu-post) c-library and copy the shared lib to the `./pos-compute/resources` folder.
On macOS, the library file is `libgpu-setup.dylib` and on Linux it is `libgpu-setup.so`.
On Windows, copy both `gpu-setup.dll` and `gpu-setup.lib` to this project root.


```bash
make
```

## Running the tests

```bash
make test
```
