use std::fmt;
use std::thread;

use colored::Colorize;
use rand::{RngExt, rng};

/// Errors that can occur during matrix operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixError {
    /// Data length does not match the expected size for a square matrix.
    InvalidDataLength { expected: usize, actual: usize },
    /// Number of threads must be at least 1.
    ZeroThreads,
    /// The current platform or environment does not support core affinity.
    CoreAffinityNotSupported,
    /// Not enough cores are available for the requested operation.
    InsufficientCores { detected: usize, required: usize },
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDataLength { expected, actual } => write!(
                f,
                "data length {} does not match expected size {} for a square matrix",
                actual.to_string().yellow(),
                expected.to_string().yellow()
            ),
            Self::ZeroThreads => write!(f, "number of threads must be greater than 0"),
            Self::CoreAffinityNotSupported => {
                write!(f, "core affinity is not supported on this platform")
            }
            Self::InsufficientCores { detected, required } => write!(
                f,
                "insufficient cores: detected {}, but {} are required",
                detected.to_string().yellow(),
                required.to_string().yellow()
            ),
        }
    }
}

impl std::error::Error for MatrixError {}

/// A square matrix represented as a 1D vector.
#[derive(Debug, Clone)]
pub struct Matrix {
    pub data: Vec<f64>,
    pub size: usize,
}

const RANDOM_RANGE: std::ops::RangeInclusive<i32> = 1..=100;

impl Matrix {
    /// Creates a new square matrix.
    pub fn new(data: Vec<f64>, size: usize) -> Result<Self, MatrixError> {
        let len = data.len();
        if size * size != len {
            return Err(MatrixError::InvalidDataLength {
                expected: size * size,
                actual: len,
            });
        }
        Ok(Matrix { data, size })
    }

    /// Creates a randomly filled square matrix.
    ///
    /// The matrix is filled with random i32s cast to f64s.
    pub fn random(size: usize) -> Self {
        let mut rng = rng();
        let data: Vec<f64> = (0..size * size)
            .map(|_| f64::from(rng.random_range(RANDOM_RANGE)))
            .collect();

        // Return the Matrix
        Matrix { data, size }
    }

    /// Creates a randomly filled square matrix in parallel.
    ///
    /// The matrix is filled with random i32s cast to f64s.
    pub fn par_random(size: usize, threads: usize) -> Result<Self, MatrixError> {
        if threads == 0 {
            return Err(MatrixError::ZeroThreads);
        }
        let mut data = vec![0.0; size * size];
        let chunk_size = (size * size).div_ceil(threads);

        thread::scope(|s| {
            // Split the data into chunks and generate random numbers for each slice
            for chunk in data.chunks_mut(chunk_size) {
                s.spawn(move || {
                    let mut rng = rand::rng();
                    for val in chunk {
                        *val = f64::from(rng.random_range(RANDOM_RANGE));
                    }
                });
            }
        });

        Ok(Matrix { data, size })
    }

    /// Applies the min-max transformation on an f64 slice
    fn slice_mmt(slice: &mut [f64]) {
        // Iterate over the slice once to find its minimum and maximum
        let (min, max) = &slice.iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(current_minimum, current_maximum), &current| {
                (current_minimum.min(current), current_maximum.max(current))
            },
        );

        if max - min > 0.0 {
            // Calculate minmax transformation of the element
            for e in slice {
                *e = (*e - min) / (max - min);
            }
        } else {
            // If all numbers are the same, then the result is 0.0 Prevents NaN for case N=1 due to division by zero.
            for e in slice {
                *e = 0.0;
            }
        }
    }

    /// Applies min-max scaling on the columns of a matrix serially
    pub fn mmt(&mut self) {
        // Split the data into column slices
        for col in self.data.chunks_mut(self.size) {
            // Apply mmt to each slice
            Self::slice_mmt(col);
        }
    }

    /// Applies min-max scaling on the columns of a matrix in parallel by column
    pub fn par_column_mmt(&mut self, threads: usize) -> Result<(), MatrixError> {
        if threads == 0 {
            return Err(MatrixError::ZeroThreads);
        }
        let size = self.size;
        let columns_per_thread = size.div_ceil(threads);
        let chunk_size = columns_per_thread * size;

        thread::scope(|s| {
            // Split the data into chunks of column slices
            for chunk in self.data.chunks_mut(chunk_size) {
                // Spawn a thread for each chunk
                s.spawn(move || {
                    // Apply mmt for each column in the chunk
                    for column in chunk.chunks_mut(size) {
                        Self::slice_mmt(column);
                    }
                });
            }
        });

        Ok(())
    }

    /// Applies the min-max scaling on the columns of a matrix in parallel by row
    pub fn par_row_mmt(&mut self, threads: usize) -> Result<(), MatrixError> {
        if threads == 0 {
            return Err(MatrixError::ZeroThreads);
        }
        let size = self.size;
        let rows_per_thread = size.div_ceil(threads);
        let data = &self.data;

        // Threads process horizontal bands of the matrix to find the minimum and maximum for their assigned rows.
        let local_extremes = thread::scope(|s| {
            let mut handles = Vec::with_capacity(threads);

            for row in 0..threads {
                // Determine the starting and ending row indices for this specific thread's band
                let start = row * rows_per_thread;
                if start >= size {
                    break;
                }
                let end = (start + rows_per_thread).min(size);

                handles.push(s.spawn(move || {
                    let mut local_mins = vec![f64::INFINITY; size];
                    let mut local_maxes = vec![f64::NEG_INFINITY; size];

                    for column in 0..size {
                        // Calculate offset to reach the correct column in column-major storage
                        let offset = column * size;
                        // Slice the specific row range (start..end) within this column
                        let column_chunk = &data[offset + start..offset + end];

                        // Iterate over each column to find its minimum and maximum
                        (local_mins[column], local_maxes[column]) = column_chunk.iter().fold(
                            (f64::INFINITY, f64::NEG_INFINITY),
                            |(current_minimum, current_maximum), &current| {
                                (current_minimum.min(current), current_maximum.max(current))
                            },
                        );
                    }
                    (local_mins, local_maxes)
                }));
            }

            // Wait for all threads to finish and collect their local results
            // This is a vector of minimums and maximums
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<(Vec<f64>, Vec<f64>)>>()
        });

        // Combine the local results from all threads to find the true min/max for every column.
        let mut global_mins = vec![f64::INFINITY; size];
        let mut global_maxes = vec![f64::NEG_INFINITY; size];
        for (local_mins, local_maxes) in local_extremes {
            for column in 0..size {
                global_mins[column] = global_mins[column].min(local_mins[column]);
                global_maxes[column] = global_maxes[column].max(local_maxes[column]);
            }
        }

        // Calculate the number of rows each thread is responsible for.
        let rows_per_thread: Vec<usize> = (0..threads)
            .map(|i| {
                let start = i * rows_per_thread;
                if start >= size {
                    0
                } else {
                    (start + rows_per_thread).min(size) - start
                }
            })
            .collect();

        // Buckets to hold the mutable slices for each thread.
        // thread_buckets[i] will contain 'size' column slices.
        let mut thread_buckets: Vec<Vec<&mut [f64]>> =
            (0..threads).map(|_| Vec::with_capacity(size)).collect();

        // Distribute the data into the buckets.
        // Split each column into chunks corresponding to the row bands assigned to each thread.
        for col in self.data.chunks_mut(size) {
            let mut remaining_col = col;
            for (bucket, &count) in thread_buckets.iter_mut().zip(&rows_per_thread) {
                let (chunk, rest) = remaining_col.split_at_mut(count);
                bucket.push(chunk);
                remaining_col = rest;
            }
        }

        // Apply the min-max transformation
        thread::scope(|s| {
            let mins = &global_mins;
            let maxes = &global_maxes;

            for bucket in thread_buckets {
                s.spawn(move || {
                    // Iterate over the columns assigned to this thread.
                    for (col_idx, chunk) in bucket.into_iter().enumerate() {
                        let min = mins[col_idx];
                        let max = maxes[col_idx];

                        for val in chunk {
                            if max - min > 0.0 {
                                *val = (*val - min) / (max - min);
                            } else {
                                *val = 0.0;
                            }
                        }
                    }
                });
            }
        });

        Ok(())
    }

    /// Applies the min-max scaling on the columns of a matrix in parallel by row with core affinity
    pub fn par_row_mmt_core_affine(&mut self, threads: usize) -> Result<(), MatrixError> {
        if threads == 0 {
            return Err(MatrixError::ZeroThreads);
        }

        let size = self.size;
        let rows_per_thread = size.div_ceil(threads);
        let data = &self.data;

        // Obtain available cores and remove the last core
        let available_cores =
            core_affinity::get_core_ids().ok_or(MatrixError::CoreAffinityNotSupported)?;
        let len = available_cores.len();
        if len < 2 {
            return Err(MatrixError::InsufficientCores {
                detected: len,
                required: 2,
            });
        }
        let available_cores = available_cores[..len - 1].to_vec();
        let core_count = available_cores.len();

        // Threads process horizontal bands of the matrix to find the minimum and maximum for their assigned rows.
        let local_extremes = thread::scope(|s| {
            let mut handles = Vec::with_capacity(threads);

            for row_band_idx in 0..threads {
                // Determine the starting and ending row indices for this specific thread's band
                let start = row_band_idx * rows_per_thread;
                if start >= size {
                    break;
                }
                let end = (start + rows_per_thread).min(size);

                // Determine the core to use for this thread
                let core_id = available_cores[row_band_idx % core_count];

                handles.push(s.spawn(move || {
                    core_affinity::set_for_current(core_id);

                    let mut local_mins = vec![f64::INFINITY; size];
                    let mut local_maxes = vec![f64::NEG_INFINITY; size];

                    for column in 0..size {
                        // Calculate offset to reach the correct column in column-major storage
                        let offset = column * size;
                        // Slice the specific row range (start..end) within this column
                        let column_chunk = &data[offset + start..offset + end];

                        // Iterate over each column to find its minimum and maximum
                        (local_mins[column], local_maxes[column]) = column_chunk.iter().fold(
                            (f64::INFINITY, f64::NEG_INFINITY),
                            |(current_minimum, current_maximum), &current| {
                                (current_minimum.min(current), current_maximum.max(current))
                            },
                        );
                    }
                    (local_mins, local_maxes)
                }));
            }

            // Wait for all threads to finish and collect their local results
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .collect::<Vec<(Vec<f64>, Vec<f64>)>>()
        });

        // Combine the local results from all threads to find the true min/max for every column.
        let mut global_mins = vec![f64::INFINITY; size];
        let mut global_maxes = vec![f64::NEG_INFINITY; size];
        for (local_mins, local_maxes) in local_extremes {
            for column in 0..size {
                global_mins[column] = global_mins[column].min(local_mins[column]);
                global_maxes[column] = global_maxes[column].max(local_maxes[column]);
            }
        }

        // Calculate the number of rows each thread is responsible for.
        let rows_per_thread_counts: Vec<usize> = (0..threads)
            .map(|i| {
                let start = i * rows_per_thread;
                if start >= size {
                    0
                } else {
                    (start + rows_per_thread).min(size) - start
                }
            })
            .collect();

        // Buckets to hold the mutable slices for each thread.
        let mut thread_buckets: Vec<Vec<&mut [f64]>> =
            (0..threads).map(|_| Vec::with_capacity(size)).collect();

        // Distribute the data into the buckets.
        for col in self.data.chunks_mut(size) {
            let mut remaining_col = col;
            for (bucket, &count) in thread_buckets.iter_mut().zip(&rows_per_thread_counts) {
                let (chunk, rest) = remaining_col.split_at_mut(count);
                bucket.push(chunk);
                remaining_col = rest;
            }
        }

        // Apply the min-max transformation
        thread::scope(|s| {
            let mins = &global_mins;
            let maxes = &global_maxes;

            for (idx, bucket) in thread_buckets.into_iter().enumerate() {
                // Determine the core to use for this thread
                let core_id = available_cores[idx % core_count];

                s.spawn(move || {
                    core_affinity::set_for_current(core_id);

                    // Iterate over the columns assigned to this thread.
                    for (col_idx, chunk) in bucket.into_iter().enumerate() {
                        let min = mins[col_idx];
                        let max = maxes[col_idx];

                        for val in chunk {
                            if max - min > 0.0 {
                                *val = (*val - min) / (max - min);
                            } else {
                                *val = 0.0;
                            }
                        }
                    }
                });
            }
        });

        Ok(())
    }

    /// Applies min-max scaling on the columns of a matrix in parallel by column
    pub fn par_column_mmt_core_affine(&mut self, threads: usize) -> Result<(), MatrixError> {
        if threads == 0 {
            return Err(MatrixError::ZeroThreads);
        }

        let size = self.size;
        let columns_per_thread = size.div_ceil(threads);
        let chunk_size = columns_per_thread * size;

        // Obtain available cores and remove the last core
        let available_cores =
            core_affinity::get_core_ids().ok_or(MatrixError::CoreAffinityNotSupported)?;
        let len = available_cores.len();
        if len < 2 {
            return Err(MatrixError::InsufficientCores {
                detected: len,
                required: 2,
            });
        }
        let available_cores = available_cores[..len - 1].to_vec();
        let len = available_cores.len();

        thread::scope(|s| {
            // Split the data into chunks of column slices
            for (index, chunk) in self.data.chunks_mut(chunk_size).enumerate() {
                // Determine the core to use for this thread
                let core_id = available_cores[index % len];

                // Spawn a thread for each chunk and pin to a specific core
                s.spawn(move || {
                    core_affinity::set_for_current(core_id);
                    // Apply mmt for each column in the chunk
                    for column in chunk.chunks_mut(size) {
                        Self::slice_mmt(column);
                    }
                });
            }
        });

        Ok(())
    }

    /// Prints the matrix to standard output.
    ///
    /// If `truncate` is false, numbers are printed with 4 decimal places.
    /// Otherwise, numbers are printed without a fractional part.
    ///
    /// If the matrix is too large (size > 5), only the top-left 5x5 corner will be shown.
    pub fn print(&self, truncate: bool) {
        let n = self.size;
        let limit = 5;
        let display_size = n.min(limit);

        if n > limit {
            println!("(Matrix is {n}x{n}, showing top-left {limit}x{limit} corner)");
        }

        let precision = if truncate { 0 } else { 4 };

        // Calculate the maximum width required for alignment
        let mut max_width = 3;
        for col_idx in 0..display_size {
            for row_idx in 0..display_size {
                let val = self.data[col_idx * self.size + row_idx];
                let width = format!("{val:.precision$}").len();
                max_width = max_width.max(width);
            }
        }

        // Print the matrix or its top-left corner if over the limit
        for row_idx in 0..display_size {
            for col_idx in 0..display_size {
                let val = self.data[col_idx * self.size + row_idx];
                print!("{val:>max_width$.precision$} ");
            }
            if n > limit {
                print!("...");
            }
            println!();
        }
        if n > limit {
            for _ in 0..display_size {
                print!("{:>width$} ", "...", width = max_width);
            }
            println!("...");
        }
    }
}
