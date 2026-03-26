# Min-Max Transformation (MMT)

This Rust project implements serial, parallel, and core-affine algorithms to perform the min-max transformation algorithm on a square matrix. It explores the performance characteristics of column-wise versus row-wise division and the impact of CPU core affinity.

## Description

The program generates an $n \times n$ square matrix of a specified size filled with random numbers. The initialization can be performed serially or in parallel. It then performs a min-max transformation on each column of the matrix, rescaling the values between 0 and 1.

The transformation can be executed in the following modes:

- **Serial**: Single-threaded implementation.
- **Parallel Column**: Distributes columns across multiple threads.
- **Parallel Row**: Distributes rows across multiple threads.
- **Core-Affine Column**: Parallel column implementation with threads pinned to specific CPU cores (excluding the last core).
- **Core-Affine Row**: Parallel row implementation with threads pinned to specific CPU cores (excluding the last core).

## Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install) (Edition 2024)

## Dependencies

-   [clap](https://crates.io/crates/clap) (4.5.60)
-   [colored](https://crates.io/crates/colored) (3.1.1)
-   [core_affinity](https://crates.io/crates/core_affinity) (0.8.3)
-   [float-cmp](https://crates.io/crates/float-cmp) (0.10.0)
-   [rand](https://crates.io/crates/rand) (0.10.0)

## Building the Project

Build the project in release mode for benchmarking:

```sh
cargo build --release
```

## Usage

Run the program using `cargo run`. The program requires one positional argument: the size of the matrix (`<MATRIX_SIZE>`).

```sh
cargo run --release -- [OPTIONS] <MATRIX_SIZE> [COMMAND]
```

### Options

- `-m`, `--matrix-preview`: Show the top-left 5x5 corner of the original and transformed matrix.
- `-s`, `--serial-init`: Initialize the matrix serially. Conflicts with `--init-threads`.
- `--init-threads=<INIT_THREADS>`: Number of threads to use for parallel initialization. Defaults to available parallelism.
- `-h`, `--help`: Print help.
- `-V`, `--version`: Print version.

### Commands

The program supports different modes of min-max transformation. If no command is provided, it defaults to the `serial` implementation.

1.  **`serial`**:
    Runs the serial implementation.
    ```sh
    cargo run --release -- 100 serial
    ```

2.  **`parallel-column`**:
    Parallel implementation distributed by column.
    ```sh
    cargo run --release -- 100 parallel-column --threads=4
    ```

3.  **`parallel-row`**:
    Parallel implementation distributed by row.
    ```sh
    cargo run --release -- 100 parallel-row --threads=4
    ```

4.  **`core-affine-column`**:
    Core-pinned parallel implementation distributed by column.
    ```sh
    cargo run --release -- 100 core-affine-column --threads=4
    ```

5.  **`core-affine-row`**:
    Core-pinned parallel implementation distributed by row.
    ```sh
    cargo run --release -- 100 core-affine-row --threads=4
    ```

### Examples

Run with a 1000x1000 matrix using 4 threads for parallel initialization and 4 threads for parallel column transformation:
```sh
cargo run --release -- --init-threads=4 1000 parallel-column --threads=4
```

Run with a 50x50 matrix, show the matrices, use serial initialization, and core-affine row transformation using 2 threads:
```sh
cargo run --release -- -m -s 50 core-affine-row -t=2
```
