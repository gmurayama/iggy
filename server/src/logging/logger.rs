//! A rolling file appender with customizable rolling conditions.
//! Includes built-in support for rolling conditions on date/time
//! (daily, hourly, every minute) and/or size.
//!
//! Follows a Debian-style naming convention for logfiles,
//! using basename, basename.1, ..., basename.N where N is
//! the maximum number of allowed historical logfiles.
//!
//! This is useful to combine with the tracing crate and
//! tracing_appender::non_blocking::NonBlocking -- use it
//! as an alternative to tracing_appender::rolling::RollingFileAppender.
//!
//! # Examples
//!
//! ```rust
//! # fn docs() {
//! # use rolling_file::*;
//! let file_appender = BasicRollingFileAppender::new(
//!     "/var/log/myprogram",
//!     RollingConditionBasic::new().daily(),
//!     9
//! ).unwrap();
//! # }
//! ```
//#![deny(warnings)]

use chrono::prelude::*;
use std::{
    convert::TryFrom,
    ffi::OsString,
    fs,
    fs::{File, OpenOptions},
    io,
    io::{BufWriter, Write},
    path::Path,
};

/// Determines when a file should be "rolled over".
pub trait RollingCondition {
    /// Determine and return whether or not the file should be rolled over.
    fn should_rollover(&mut self, now: &DateTime<Local>, current_filesize: u64) -> bool;
}

/// Determines how often a file should be rolled over
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RollingFrequency {
    EveryDay,
    EveryHour,
    EveryMinute,
}

impl RollingFrequency {
    /// Calculates a datetime that will be different if data should be in
    /// different files.
    pub fn equivalent_datetime(&self, dt: &DateTime<Local>) -> DateTime<Local> {
        match self {
            RollingFrequency::EveryDay => Local
                .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
                .unwrap(),
            RollingFrequency::EveryHour => Local
                .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), dt.hour(), 0, 0)
                .unwrap(),
            RollingFrequency::EveryMinute => Local
                .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), dt.hour(), dt.minute(), 0)
                .unwrap(),
        }
    }
}

#[derive(Debug)]
struct Writer {
    path: OsString,
    file: Option<BufWriter<File>>,
    buffer_capacity: Option<usize>,
}

impl Writer {
    pub fn new(path: OsString) -> Self {
        Writer {
            path,
            file: None,
            buffer_capacity: None,
        }
    }

    pub fn new_with_buffer_capacity(path: OsString, buffer_capacity: usize) -> Self {
        Writer {
            path,
            file: None,
            buffer_capacity: Some(buffer_capacity),
        }
    }

    fn open(&mut self) -> io::Result<()> {
        if self.file.is_none() {
            let f = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&self.path)?;
            self.file = Some(match self.buffer_capacity {
                Some(capacity) => BufWriter::with_capacity(capacity, f),
                None => BufWriter::new(f),
            })
        }
        Ok(())
    }

    pub fn try_write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.open()?;
        match self.file.as_mut() {
            Some(file) => file.write(buf),
            None => Err(io::Error::new(
                io::ErrorKind::Other,
                "unexpected condition: writer is missing",
            )),
        }
    }

    pub fn open_new_file(&mut self, path: OsString) -> io::Result<()> {
        self.close();
        self.path = path;
        self.open()
    }

    pub fn close(&mut self) {
        self.file.take();
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if let Some(file) = self.file.as_mut() {
            file.flush()?;
        }

        Ok(())
    }
}

/// Implements a rolling condition based on a certain frequency
/// and/or a size limit. The default condition is to rotate daily.
///
/// # Examples
///
/// ```rust
/// use rolling_file::*;
/// let c = RollingConditionBasic::new().daily();
/// let c = RollingConditionBasic::new().hourly().max_size(1024 * 1024);
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RollingConditionBasic {
    last_write_opt: Option<DateTime<Local>>,
    frequency_opt: Option<RollingFrequency>,
    max_size_opt: Option<u64>,
}

impl RollingConditionBasic {
    /// Constructs a new struct that does not yet have any condition set.
    pub fn new() -> RollingConditionBasic {
        RollingConditionBasic {
            last_write_opt: None,
            frequency_opt: None,
            max_size_opt: None,
        }
    }

    /// Sets a condition to rollover on the given frequency
    pub fn frequency(mut self, x: RollingFrequency) -> RollingConditionBasic {
        self.frequency_opt = Some(x);
        self
    }

    /// Sets a condition to rollover when the date changes
    pub fn daily(mut self) -> RollingConditionBasic {
        self.frequency_opt = Some(RollingFrequency::EveryDay);
        self
    }

    /// Sets a condition to rollover when the date or hour changes
    pub fn hourly(mut self) -> RollingConditionBasic {
        self.frequency_opt = Some(RollingFrequency::EveryHour);
        self
    }

    /// Sets a condition to rollover when a certain size is reached
    pub fn max_size(mut self, x: u64) -> RollingConditionBasic {
        self.max_size_opt = Some(x);
        self
    }
}

impl Default for RollingConditionBasic {
    fn default() -> Self {
        RollingConditionBasic::new().frequency(RollingFrequency::EveryDay)
    }
}

impl RollingCondition for RollingConditionBasic {
    fn should_rollover(&mut self, now: &DateTime<Local>, current_filesize: u64) -> bool {
        let mut rollover = false;
        if let Some(frequency) = self.frequency_opt.as_ref() {
            if let Some(last_write) = self.last_write_opt.as_ref() {
                if frequency.equivalent_datetime(now) != frequency.equivalent_datetime(last_write) {
                    rollover = true;
                }
            }
        }
        if let Some(max_size) = self.max_size_opt.as_ref() {
            if current_filesize >= *max_size {
                rollover = true;
            }
        }
        self.last_write_opt = Some(*now);
        rollover
    }
}

/// Writes data to a file, and "rolls over" to preserve older data in
/// a separate set of files. Old files have a Debian-style naming scheme
/// where we have base_filename, base_filename.1, ..., base_filename.N
/// where N is the maximum number of rollover files to keep.
#[derive(Debug)]
pub struct RollingFileAppender<RC>
where
    RC: RollingCondition,
{
    condition: RC,
    base_filename: OsString,
    max_files: usize,
    current_foldersize: u64,
    writer: Writer,
}

impl<RC> RollingFileAppender<RC>
where
    RC: RollingCondition,
{
    /// Creates a new rolling file appender with the given condition.
    /// The parent directory of the base path must already exist.
    pub fn new<P>(path: P, condition: RC, max_files: usize) -> io::Result<RollingFileAppender<RC>>
    where
        P: AsRef<Path>,
    {
        Self::_new(path, condition, max_files, None)
    }

    /// Creates a new rolling file appender with the given condition and write buffer capacity.
    /// The parent directory of the base path must already exist.
    pub fn new_with_buffer_capacity<P>(
        path: P,
        condition: RC,
        max_files: usize,
        buffer_capacity: usize,
    ) -> io::Result<RollingFileAppender<RC>>
    where
        P: AsRef<Path>,
    {
        Self::_new(path, condition, max_files, Some(buffer_capacity))
    }

    fn _new<P>(
        path: P,
        condition: RC,
        max_files: usize,
        buffer_capacity: Option<usize>,
    ) -> io::Result<RollingFileAppender<RC>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().as_os_str().to_os_string();
        let writer = match buffer_capacity {
            Some(capacity) => Writer::new_with_buffer_capacity(path.clone(), capacity),
            None => Writer::new(path.clone()),
        };
        let mut rfa = RollingFileAppender {
            condition,
            base_filename: path,
            max_files,
            current_foldersize: 0,
            writer,
        };

        // Fail if we can't open the file initially...
        rfa.writer.open()?;
        rfa.current_foldersize = rfa.folder_size();
        Ok(rfa)
    }

    /// Determines the final filename, where n==0 indicates the current file
    fn filename_for(&self, n: usize) -> OsString {
        let mut f = self.base_filename.clone();
        if n > 0 {
            f.push(OsString::from(format!(".{}", n)))
        }
        f
    }

    /// Rotates old files to make room for a new one.
    /// This may result in the deletion of the oldest file
    fn rotate_files(&mut self) -> io::Result<()> {
        // ignore any failure removing the oldest file (may not exist)
        let _ = fs::remove_file(self.filename_for(self.max_files.max(1)));
        let mut r = Ok(());
        for i in (0..self.max_files.max(1)).rev() {
            let rotate_from = self.filename_for(i);
            let rotate_to = self.filename_for(i + 1);
            if let Err(e) = fs::rename(rotate_from, rotate_to).or_else(|e| match e.kind() {
                io::ErrorKind::NotFound => Ok(()),
                _ => Err(e),
            }) {
                // capture the error, but continue the loop,
                // to maximize ability to rename everything
                r = Err(e);
            }
        }
        r
    }

    fn folder_size(&self) -> u64 {
        let mut size = 0;

        for i in 0..self.max_files.max(1) {
            match fs::metadata(self.filename_for(i)) {
                Ok(metadata) => {
                    size += metadata.len();
                }
                Err(err) => {
                    if err.kind() != io::ErrorKind::NotFound {
                        eprintln!("ERROR can not access metadata from file: {err}");
                    }
                }
            };
        }

        size
    }

    /// Forces a rollover to happen immediately.
    pub fn rollover(&mut self) -> io::Result<()> {
        // Before closing, make sure all data is flushed successfully.
        self.flush()?;
        // We must close the current file before rotating files
        self.writer.close();
        self.rotate_files()?;
        self.current_foldersize = self.folder_size();
        self.writer.open_new_file(self.filename_for(0))
    }

    /// Returns a reference to the rolling condition
    pub fn condition_ref(&self) -> &RC {
        &self.condition
    }

    /// Returns a mutable reference to the rolling condition, possibly to mutate its state dynamically.
    pub fn condition_mut(&mut self) -> &mut RC {
        &mut self.condition
    }

    /// Writes data using the given datetime to calculate the rolling condition
    pub fn write_with_datetime(&mut self, buf: &[u8], now: &DateTime<Local>) -> io::Result<usize> {
        if self.condition.should_rollover(now, self.current_foldersize) {
            if let Err(e) = self.rollover() {
                // If we can't rollover, just try to continue writing anyway
                // (better than missing data).
                // This will likely used to implement logging, so
                // avoid using log::warn and log to stderr directly
                eprintln!(
                    "WARNING: Failed to rotate logfile {}: {}",
                    self.base_filename.to_string_lossy(),
                    e
                );
            }
        }

        let buf_len = buf.len();
        self.writer.try_write(buf).map(|_| {
            self.current_foldersize += u64::try_from(buf_len).unwrap_or(u64::MAX);
            buf_len
        })
    }
}

impl<RC> io::Write for RollingFileAppender<RC>
where
    RC: RollingCondition,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let now = Local::now();
        self.write_with_datetime(buf, &now)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// A rolling file appender with a rolling condition based on date/time or size.
pub type BasicRollingFileAppender = RollingFileAppender<RollingConditionBasic>;

// LCOV_EXCL_START
#[cfg(test)]
mod t {
    // use super::*;
}
// LCOV_EXCL_STOP
