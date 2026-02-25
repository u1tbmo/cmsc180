use clap::{Parser, Subcommand, error::ErrorKind};
use colored::Colorize;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    ThreadCountExceedsMatrixSize { matrix_size: usize, threads: usize },
}

/// A program that runs various implementations of min-max transformation on columns of a matrix.
#[derive(Debug, Parser)]
#[clap(author, version)]
pub struct ProgramArgs {
    /// Show the original and transformed matrix
    #[arg(short = 'm', long)]
    pub matrix_preview: bool,

    /// Initialize the matrix serially.
    #[arg(short = 's', long, conflicts_with = "init_threads")]
    pub serial_init: bool,

    /// Number of threads to use for parallel initialization
    ///
    /// Defaults to available parallelism
    #[arg(
        long,
        require_equals = true,
        value_name = "INIT_THREADS",
        value_parser = clap::value_parser!(u64).range(1..)
    )]
    pub init_threads: Option<u64>,

    /// The size of the square matrix.
    #[arg(value_parser = clap::value_parser!(u64).range(1..))]
    pub matrix_size: u64,

    /// The specific implementation of min-max transformation to use
    #[command(subcommand)]
    pub mode: Option<Mode>,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Serial implementation.
    Serial,
    /// Parallel implementation by column.
    #[command(name = "parallel-column")]
    ParallelColumn {
        /// The number of threads to use.
        #[arg(
            short,
            long, 
            require_equals=true, 
            value_parser = clap::value_parser!(u64).range(1..)
        )]
        threads: Option<u64>,
    },
    /// Parallel implementation by row.
    #[command(name = "parallel-row")]
    ParallelRow {
        /// The number of threads to use.
        #[arg(
            short, 
            long, 
            require_equals=true, 
            value_parser = clap::value_parser!(u64).range(1..)
        )]
        threads: Option<u64>,
    },
}

impl ProgramArgs {
    /// Validates all program arguments
    pub fn validate(&self) -> Result<(), CliError> {
        // Validate parallel init
        if let Some(init_threads) = self.init_threads
            && init_threads > self.matrix_size {
                return Err(CliError::ThreadCountExceedsMatrixSize {
                    matrix_size: self.matrix_size as usize,
                    threads: init_threads as usize,
                });
            }

        // Validate execution mode threads
        if let Some(mode) = &self.mode
            && let Mode::ParallelColumn { threads: Some(t) }
                            | Mode::ParallelRow { threads: Some(t) } = mode
                && *t > self.matrix_size {
                    return Err(CliError::ThreadCountExceedsMatrixSize {
                        matrix_size: self.matrix_size as usize,
                        threads: *t as usize,
                    });
                }
        Ok(())
    }
}

impl CliError {
    pub fn error_kind(&self) -> ErrorKind {
        match self {
            Self::ThreadCountExceedsMatrixSize { .. } => ErrorKind::ValueValidation,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ThreadCountExceedsMatrixSize {
                matrix_size,
                threads,
            } => write!(
                f,
                "unexpectedly got {} threads, which exceeds matrix size {}",
                threads.to_string().yellow(),
                matrix_size.to_string().yellow(),
            ),
        }
    }
}
