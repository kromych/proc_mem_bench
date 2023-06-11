## `pread` vs `process_vm_read` when reading remote process memory

This code measures performance of the two APIs when reading the memory of the remote
process. That might be useful for the debuggers and other tools instrospecting the
state of another processes.

The approach relies on using two processes. The process that performs benchmarking
starts the child process. The child process allocates some memory as instructed by
the command line parameters, and then it notifies the parent process that the allocation
is ready. The parent process reads the child process'es memory varying the size and
measures the `pread`/`process_vm_read` ratio.

The way of notification might come across as too involved or low-level as it uses
the processor registers (so is architecture-dependent) and the
[Linux `ptrace` API](https://man7.org/linux/man-pages/man2/ptrace.2.html). This is
intentional to show off how `ptrace` might be used.

To run the benchmark:

1. `cargo build --release`
2. `cargo run --release --bin read`

`pread`/`process_vm_read`

1. Fedora 38/aarch64, Linux 6.3.6, KVM/M1 Ultra

| Size, bytes | 1    |
|-------------|------|
| 16          | 0.98 |
| 32          | 0.97 |
| 64          | 0.96 |
| 128         | 0.98 |
| 256         | 0.98 |
| 512         | 1.00 |
| 1024        | 1.01 |
| 2048        | 1.05 |
| 4096        | 1.10 |
| 8192        | 1.27 |
| 16384       | 1.46 |
| 32768       | 1.64 |
| 65536       | 1.65 |
| 131072      | 1.61 |
| 262144      | 1.65 |
| 524288      | 1.66 |
| 1048576     | 1.69 |
| 2097152     | 1.69 |
| 4194304     | 1.43 |
| 8388608     | 1.34 |
| 16777216    | 1.31 |
| 33554432    | 1.24 |
| 67108864    | 1.21 |
| 134217728   | 1.20 |
