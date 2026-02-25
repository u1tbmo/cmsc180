# Parallel Min-Max Transformation (MMT)

This Rust project implements serial and parallel algorithms to perform the min-max transformation algorithm on a square matrix.

## Description

The program generates an n x n square matrix of a specified size filled with random numbers. The initialization can be performed serially or in parallel.
It then performs a min-max transformation on each column of the matrix. This transformation can be executed serially, in parallel by column, or in parallel by row.
The transformation rescales the values in each column such that they fall between 0 and 1.

## Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install)

## Dependencies

-   [clap](https://crates.io/crates/clap) (4.5.57)
-   [colored](https://crates.io/crates/colored) (3.1.1)
-   [float-cmp](https://crates.io/crates/float-cmp) (0.10.0)
-   [rand](https://crates.io/crates/rand) (0.10.0)

## Building the Project

Build the project in release mode:

```sh
cargo build --release
```

## Usage

Run the program using `cargo run`. The program requires one positional argument: the size of the matrix (`<MATRIX_SIZE>`).

```sh
cargo run --release -- [OPTIONS] <MATRIX_SIZE> [COMMAND]
```

### Options

- `-m`, `--matrix-preview`: Show the original and transformed matrix.
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
    Runs the parallel implementation where work is distributed by column.
    ```sh
    cargo run --release -- 100 parallel-column [OPTIONS]
    ```
    - `-t`, `--threads=<THREADS>`: The number of threads to use. Defaults to available parallelism.

3.  **`parallel-row`**:
    Runs the parallel implementation where work is distributed by row.
    ```sh
    cargo run --release -- 100 parallel-row [OPTIONS]
    ```
    - `-t`, `--threads=<THREADS>`: The number of threads to use. Defaults to available parallelism.

### Examples

Run with a 1000x1000 matrix using 4 threads for parallel initialization and 4 threads for parallel column transformation:
```sh
cargo run --release -- --init-threads=4 1000 parallel-column --threads=4
```

Run with a 50x50 matrix, show the matrices, use serial initialization, and parallel row transformation using 2 threads:
```sh
cargo run --release -- -m -s 50 parallel-row -t=2
```

## Output

The program outputs:
1.  Time taken to create and fill the matrix (Initialization), including the mode (Serial or Parallel by Column) and number of threads used.
2.  Time taken to perform the min-max transformation (Implementation), including the mode (Serial, Parallel by Column, or Parallel by Row) and number of threads used.

If the `--matrix-preview` (`-m`) flag is provided, the original matrix and the transformed matrix are printed.
