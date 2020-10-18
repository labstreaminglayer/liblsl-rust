/*! Rust API for the [Lab Streaming Layer](https://github.com/sccn/labstreaminglayer) (LSL).
The lab streaming layer provides a set of functions to make instrument data accessible
in real time within a lab network. From there, streams can be picked up by recording programs,
viewing programs or custom experiment applications that access data streams in real time.

The API covers two areas:
- The "push API" allows to create stream outlets and to push data (regular or irregular measurement
  time series, event data, coded audio/video frames, etc.) into them.
- The "pull API" allows to create stream inlets and read time-synched experiment data from them
  (for recording, viewing or experiment control).

This module links to the [liblsl](https://github.com/sccn/liblsl) library via the low-level
`lsl-sys` crate.
*/

use lsl_sys::*;
use std::ffi::CStr;


/// Constant to indicate that a stream has variable sampling rate.
pub const IRREGULAR_RATE: f64 = 0.0;

/**
Constant to indicate that a sample has the next successive time stamp.
This is an optional optimization to transmit less data per sample.
The stamp is then deduced from the preceding one according to the stream's sampling rate
(in the case of an irregular rate, the same time stamp as before will is assumed).
*/
pub const DEDUCED_TIMESTAMP: f64 = -1.0;

/**
A very large time duration (> 1 year) for timeout values.
Note that significantly larger numbers can cause the timeout to be invalid on some
operating systems (e.g., 32-bit UNIX).
*/
pub const FOREVER: f64 = 32000000.0;


/// Data format of a channel (each transmitted sample holds an array of channels).
#[derive(Copy, Clone, Debug)]
pub enum ChannelFormat {
    /// For up to 24-bit precision measurements in the appropriate physical unit
    /// (e.g., microvolts). Integers from -16777216 to 16777216 are represented accurately.
    Float32 = 1,
    /// For universal numeric data as long as permitted by network & disk budget.
    /// The largest representable integer is 53-bit.
    Double64 = 2,
    /// For variable-length ASCII strings or data blobs, such as video frames,
    /// complex event descriptions, etc.
    String = 3,
    /// For high-rate digitized formats that require 32-bit precision. Depends critically on
    /// meta-data to represent meaningful units. Useful for application event codes or other
    /// coded data.
    Int32 = 4,
    /// For very high rate signals (40Khz+) or consumer-grade audio
    /// (for professional audio, float is recommended).
    Int16 = 5,
    /// For binary signals or other coded data. Not recommended for encoding string data.
    Int8 = 6,
    /// Note that support for this type is not yet exposed in all languages.
    /// Also, some builds of liblsl will not be able to send or receive data of this type.
    Int64 = 7,
    /// Can not be transmitted.
    Undefined = 0,
}


/// Post-processing options for stream inlets.
///#[derive(Copy, Clone, Debug)]
pub enum ProcessingOptions {
    /// No automatic post-processing; return the ground-truth time stamps for manual post-
    /// processing (this is the default behavior of the inlet).
    None = 0,
    /// Perform automatic clock synchronization; equivalent to manually adding the
    /// `time_correction()` value to the received time stamps.
    ClockSync = 1,
    /// Remove jitter from time stamps. This will apply a smoothing algorithm to the received time
    /// stamps; the smoothing needs to see a minimum number of samples (30-120 seconds worst-case)
    /// until the remaining jitter is consistently below 1ms.
    Dejitter = 2,
    /// Force the time-stamps to be monotonically ascending (only makes sense if timestamps are
    /// dejittered).
    Monotonize = 4,
    /// Post-processing is thread-safe (same inlet can be read from by multiple threads);
    /// uses somewhat more CPU.
    Threadsafe = 8,
    /// The combination of all possible post-processing options.
    ALL = 1|2|4|8
}


/** Protocol version.
The major version is protocol_version() / 100;
The minor version is protocol_version() % 100;
Clients with different minor versions are protocol-compatible with each other
while clients with different major versions will refuse to work together.
*/
pub fn protocol_version() -> i32 {
    unsafe {
        lsl_protocol_version()
    }
}

/** Version of the liblsl library.
* The major version is library_version() / 100;
* The minor version is library_version() % 100;
*/
pub fn library_version() -> i32 {
    unsafe {
        lsl_library_version()
    }
}


/** Get a string containing library information. The format of the string shouldn't be used
for anything important except giving a a debugging person a good idea which exact library
version is used. */
pub fn library_info() -> &'static str {
    unsafe {
        CStr::from_ptr(lsl_library_info()).to_str().unwrap()
    }
}




/**
Obtain a local system time stamp in seconds. The resolution is better than a millisecond.
This reading can be used to assign time stamps to samples as they are being acquired.
If the "age" of a sample is known at a particular time (e.g., from USB transmission
delays), it can be used as an offset to local_clock() to obtain a better estimate of
when a sample was actually captured. See StreamOutlet::push_sample() for a use case.
*/
pub fn local_clock() -> f64 {
    unsafe {
        lsl_local_clock()
    }
}
