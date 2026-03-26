use clap::{error::ErrorKind, Parser, Subcommand};
use colored::Colorize;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    ExcessiveThreadCount {
        matrix_size: usize,
        threads: usize,
    },
    CoreAffinityNotSupported,
    InsufficientCores {
        count: usize,
    },
    InvalidMatrixSize,
    InvalidThreadCount {
        context: &'static str,
    },
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
        value_parser = clap::value_parser!(u64)
    )]
    pub init_threads: Option<u64>,

    /// The size of the square matrix.
    #[arg(value_parser = clap::value_parser!(u64))]
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
            require_equals = true,
            value_parser = clap::value_parser!(u64)
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
            require_equals = true,
            value_parser = clap::value_parser!(u64)
        )]
        threads: Option<u64>,
    },

    /// Core-affine parallel implementation by column.
    #[command(name = "core-affine-column")]
    CoreAffineColumn {
        /// The number of threads to use.
        #[arg(
            short,
            long,
            require_equals = true,
            value_parser = clap::value_parser!(u64)
        )]
        threads: Option<u64>,
    },

    /// Core-affine parallel implementation by row.
    #[command(name = "core-affine-row")]
    CoreAffineRow {
        /// The number of threads to use.
        #[arg(
            short,
            long,
            require_equals = true,
            value_parser = clap::value_parser!(u64)
        )]
        threads: Option<u64>,
    },
}

impl ProgramArgs {
    /// Validates all program arguments
    pub fn validate(&self) -> Result<(), CliError> {
        // Validate matrix size
        if self.matrix_size < 1 {
            return Err(CliError::InvalidMatrixSize);
        }

        // Validate parallel init
        if let Some(init_threads) = self.init_threads {
            if init_threads < 1 {
                return Err(CliError::InvalidThreadCount {
                    context: "initialization",
                });
            }
            if init_threads > self.matrix_size {
                return Err(CliError::ExcessiveThreadCount {
                    matrix_size: self.matrix_size as usize,
                    threads: init_threads as usize,
                });
            }
        }

        // Validate execution mode threads
        if let Some(mode) = &self.mode {
            let threads = match mode {
                Mode::ParallelColumn { threads }
                | Mode::ParallelRow { threads }
                | Mode::CoreAffineColumn { threads }
                | Mode::CoreAffineRow { threads } => *threads,
                Mode::Serial => None,
            };

            if let Some(t) = threads {
                if t < 1 {
                    return Err(CliError::InvalidThreadCount {
                        context: "transformation",
                    });
                }
                if t > self.matrix_size {
                    return Err(CliError::ExcessiveThreadCount {
                        matrix_size: self.matrix_size as usize,
                        threads: t as usize,
                    });
                }
            }

            if matches!(mode, Mode::CoreAffineColumn { .. } | Mode::CoreAffineRow { .. }) {
                let core_ids =
                    core_affinity::get_core_ids().ok_or(CliError::CoreAffinityNotSupported)?;
                if core_ids.len() < 2 {
                    return Err(CliError::InsufficientCores {
                        count: core_ids.len(),
                    });
                }
            }
        }
        Ok(())
    }
}

impl CliError {
    pub fn error_kind(&self) -> ErrorKind {
        match self {
            Self::ExcessiveThreadCount { .. }
            | Self::InsufficientCores { .. }
            | Self::InvalidMatrixSize
            | Self::InvalidThreadCount { .. } => ErrorKind::ValueValidation,
            Self::CoreAffinityNotSupported => ErrorKind::Io,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExcessiveThreadCount {
                matrix_size,
                threads,
            } => write!(
                f,
                "unexpectedly got {} threads, which exceeds matrix size {}",
                threads.to_string().yellow(),
                matrix_size.to_string().yellow(),
            ),
            Self::CoreAffinityNotSupported => write!(
                f,
                "core affinity is not supported on this platform or environment",
            ),
            Self::InsufficientCores { count } => write!(
                f,
                "core-affine mode requires at least 2 logical processors, but only {} {} detected",
                count.to_string().yellow(),
                if *count == 1 { "was" } else { "were" }
            ),
            Self::InvalidMatrixSize => write!(f, "matrix size must be at least 1"),
            Self::InvalidThreadCount { context } => {
                write!(f, "{context} thread count must be at least 1")
            }
        }
    }
}
