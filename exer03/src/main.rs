// Tabamo, Euan Jed S.
// CMSC 180 - CD2L
// Runtime-efficient Threaded Min-Max Transformation of a Matrix

use std::thread;

use clap::{CommandFactory, Parser};
use colored::Colorize;
use minmax_transform::args::{CliError, Mode, ProgramArgs};
use minmax_transform::matrix::{Matrix, MatrixError};

fn main() {
    // Parse the program
    let args = ProgramArgs::parse();
    if let Err(error) = args.validate() {
        exit_with_usage_error(error);
    }

    // Get the size of the matrix and the number of logical processors available
    let matrix_size = args.matrix_size as usize;
    let total_cores = core_affinity::get_core_ids().map_or(0, |ids| ids.len());

    // Matrix Initialization
    let start = std::time::Instant::now();
    let (mut matrix, mode_str, init_threads) = if args.serial_init {
        (Matrix::random(matrix_size), "Serial", None)
    } else {
        let t = args.init_threads.map_or_else(
            || {
                thread::available_parallelism()
                    .map_or(1, std::num::NonZero::get)
                    .min(matrix_size)
            },
            |v| v as usize,
        );
        let m = Matrix::par_random(matrix_size, t).unwrap_or_else(|e| exit_with_runtime_error(e));
        (m, "Parallel by Column", Some(t))
    };
    let init_duration = start.elapsed();

    print_report(
        "Initialization",
        mode_str,
        init_threads,
        format!("{total_cores} available"),
        init_duration,
        colored::Color::Green,
    );

    if args.matrix_preview {
        print_matrix("Original Matrix", &matrix, true);
    }

    // Min-Max Transformation

    // Obtain the default thread count using the suggested number of threads to use
    let default_threads = thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .min(matrix_size);

    // Closure to determine the number of threads to use
    let get_threads = |t_opt: Option<u64>| t_opt.map_or(default_threads, |v| v as usize);

    let start = std::time::Instant::now();
    let mode = args.mode.as_ref().unwrap_or(&Mode::Serial);
    let (label, threads, lp_info) = match mode {
        Mode::Serial => {
            matrix.mmt();
            ("Serial", None, format!("{total_cores} available"))
        }
        Mode::ParallelColumn { threads } => {
            let t = get_threads(*threads);
            matrix
                .par_column_mmt(t)
                .unwrap_or_else(|e| exit_with_runtime_error(e));
            (
                "Parallel by Column",
                Some(t),
                format!("{total_cores} available"),
            )
        }
        Mode::ParallelRow { threads } => {
            let t = get_threads(*threads);
            matrix
                .par_row_mmt(t)
                .unwrap_or_else(|e| exit_with_runtime_error(e));
            (
                "Parallel by Row",
                Some(t),
                format!("{total_cores} available"),
            )
        }
        Mode::CoreAffineColumn { threads } => {
            let t = get_threads(*threads);
            matrix
                .par_column_mmt_core_affine(t)
                .unwrap_or_else(|e| exit_with_runtime_error(e));

            let pool_size = total_cores.saturating_sub(1);
            let used = t.min(pool_size).min(matrix_size);
            let info = format!("{used} used, {total_cores} available");

            ("Parallel by Column (Core-Affine)", Some(t), info)
        }
        Mode::CoreAffineRow { threads } => {
            let t = get_threads(*threads);
            matrix
                .par_row_mmt_core_affine(t)
                .unwrap_or_else(|e| exit_with_runtime_error(e));

            let pool_size = total_cores.saturating_sub(1);
            let used = t.min(pool_size).min(matrix_size);
            let info = format!("{used} used, {total_cores} available");

            ("Parallel by Row (Core-Affine)", Some(t), info)
        }
    };
    let mmt_duration = start.elapsed();

    print_report(
        "Implementation",
        label,
        threads,
        lp_info,
        mmt_duration,
        colored::Color::Yellow,
    );

    if args.matrix_preview {
        print_matrix("Transformed Matrix", &matrix, false);
    }
}

/// Exits the program with a usage error message
fn exit_with_usage_error(error: CliError) -> ! {
    ProgramArgs::command()
        .error(error.error_kind(), error)
        .exit();
}

/// Exits the program with a runtime error message
fn exit_with_runtime_error(error: MatrixError) -> ! {
    eprintln!("{}: {}", "matrix error".red().bold(), error);
    std::process::exit(1);
}

/// Prints a report section
fn print_report(
    title: &str,
    mode: &str,
    threads: Option<usize>,
    lp_info: String,
    duration: std::time::Duration,
    color: colored::Color,
) {
    println!("\n{}", title.bold().underline());
    println!("{:<20} {}", "Mode:", mode.color(color));
    if let Some(t) = threads {
        println!("{:<20} {}", "Threads:", t.to_string().color(color));
    }
    println!("{:<20} {}", "Logical Processors:", lp_info.color(color));
    println!(
        "{:<20} {}",
        "Runtime:",
        format!("{:.8}", duration.as_secs_f64()).color(color)
    );
}

/// Prints the matrix with a header
fn print_matrix(title: &str, matrix: &Matrix, truncate: bool) {
    println!("\n{}", title.bold().underline());
    matrix.print(truncate);
}
