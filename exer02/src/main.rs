// Tabamo, Euan Jed S.
// CMSC 180 - CD2L
// Runtime-efficient Threaded Min-Max Transformation of a Matrix

use std::thread;

use clap::{CommandFactory, Parser};
use colored::Colorize;
use minmax_transform::args::{Mode, ProgramArgs};
use minmax_transform::matrix::Matrix;

fn main() {
    let args = ProgramArgs::parse();

    // Validate arguments
    if let Err(error) = args.validate() {
        ProgramArgs::command()
            .error(error.error_kind(), error)
            .exit();
    }

    let matrix_size = args.matrix_size as usize;

    // Initialization
    let init_threads = if args.serial_init {
        None
    } else {
        let t = args.init_threads.map_or_else(
            || {
                thread::available_parallelism()
                    .map_or(1, std::num::NonZero::get)
                    .min(matrix_size)
            },
            |v| v as usize,
        );
        assert!(t > 0, "expected at least one thread");
        Some(t)
    };

    let start = std::time::Instant::now();
    let mut matrix = match init_threads {
        Some(t) => Matrix::par_random(matrix_size, t),
        None => Matrix::random(matrix_size),
    };
    let init_duration = start.elapsed();

    print_report(
        "Initialization",
        if init_threads.is_some() {
            "Parallel by Column"
        } else {
            "Serial"
        },
        init_threads,
        init_duration,
        colored::Color::Green,
    );

    if args.matrix_preview {
        print_matrix("Original Matrix", &matrix, true);
    }

    // Transformation
    let start = std::time::Instant::now();
    let mode = args.mode.as_ref().unwrap_or(&Mode::Serial);

    let default_threads = thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .min(matrix_size);

    let get_threads = |threads: Option<u64>| -> usize {
        let t = threads.map_or(default_threads, |t| t as usize);
        assert!(t > 0, "expected at least one thread");
        t
    };

    match mode {
        Mode::Serial => matrix.mmt(),
        Mode::ParallelColumn { threads } => {
            matrix.par_column_mmt(get_threads(*threads));
        }
        Mode::ParallelRow { threads } => {
            matrix.par_row_mmt(get_threads(*threads));
        }
    }
    let mmt_duration = start.elapsed();

    let (mode_str, threads) = match mode {
        Mode::Serial => ("Serial".to_string(), None),
        Mode::ParallelColumn { threads } => (
            "Parallel by Column".to_string(),
            Some(get_threads(*threads)),
        ),
        Mode::ParallelRow { threads } => {
            ("Parallel by Row".to_string(), Some(get_threads(*threads)))
        }
    };
    print_report(
        "Implementation",
        &mode_str,
        threads,
        mmt_duration,
        colored::Color::Yellow,
    );

    if args.matrix_preview {
        print_matrix("Transformed Matrix", &matrix, false);
    }
}

/// Prints a report section
fn print_report(
    title: &str,
    mode: &str,
    threads: Option<usize>,
    duration: std::time::Duration,
    color: colored::Color,
) {
    println!("\n{}", title.bold().underline());
    println!("{:<8} {}", "Mode:", mode.color(color));
    if let Some(t) = threads {
        println!("{:<8} {}", "Threads:", t.to_string().color(color));
    }
    println!(
        "{:<8} {}",
        "Runtime:",
        format!("{:.8}", duration.as_secs_f64()).color(color)
    );
}

/// Prints the matrix with a header
fn print_matrix(title: &str, matrix: &Matrix, truncate: bool) {
    println!("\n{}", title.bold().underline());
    matrix.print(truncate);
}
