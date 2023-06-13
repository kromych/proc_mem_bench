## `pread` vs `process_vm_read` when reading remote process memory

This code measures performance of the two APIs when reading the memory of the remote
process. That might be useful for the debuggers and other tools alike that instrospect
the state of another processes. From the below data (rather limited yet coming from
two very diffrent systems) it follows that when reading at least `8KiB` of data from
the remote process, `process_vm_read` is significantly more performant.

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

1. Fedora 38/aarch64, Linux 6.3.6 KVM, mac Studio M1 Ultra (2022)
2. Fedora 38/x86_64, Linux 6.3.6 baremetal, Dell 9550 laptop (~2016)
3. Arch/aarch64, Linux 6.3.0 baremetal,  mac Mini M1 (2021)

| Size, bytes | 1    | 2    | 3    |
|-------------|------|------|------|
| 16          | 0.98 | 0.83 | 0.75 |
| 32          | 0.97 | 0.83 | 0.75 |
| 64          | 0.96 | 0.83 | 0.76 |
| 128         | 0.98 | 0.83 | 0.75 |
| 256         | 0.98 | 0.84 | 0.76 |
| 512         | 1.00 | 0.86 | 0.77 |
| 1024        | 1.01 | 0.88 | 0.79 |
| 2048        | 1.05 | 0.90 | 0.83 |
| 4096        | 1.10 | 0.97 | 0.91 |
| 8192        | 1.27 | 1.37 | 1.02 |
| 16384       | 1.46 | 1.62 | 1.13 |
| 32768       | 1.64 | 1.87 | 1.35 |
| 65536       | 1.65 | 2.12 | 1.47 |
| 131072      | 1.61 | 2.17 | 1.47 |
| 262144      | 1.65 | 2.04 | 1.53 |
| 524288      | 1.66 | 2.04 | 1.56 |
| 1048576     | 1.69 | 2.06 | 1.57 |
| 2097152     | 1.69 | 2.20 | 1.57 |
| 4194304     | 1.43 | 1.99 | 1.31 |
| 8388608     | 1.34 | 1.81 | 1.25 |
| 16777216    | 1.31 | 1.65 | 1.20 |
| 33554432    | 1.24 | 1.66 | 1.18 |
| 67108864    | 1.21 | 1.72 | 1.15 |
| 134217728   | 1.20 | 1.67 | 1.12 |
