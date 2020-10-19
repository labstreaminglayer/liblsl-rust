/*!
Rust API for the [Lab Streaming Layer](https://github.com/sccn/labstreaminglayer) (LSL).
The lab streaming layer provides a set of functions to make instrument data accessible
in real time within a lab's network. From there, streams can be picked up by recording programs,
viewing programs or other applications that access data streams in real time.

The API covers two areas:
- The "push API" allows to create stream outlets and to push data (regular or irregular measurement
  time series, event data, coded audio/video frames, etc.) into them.
- The "pull API" allows to create stream inlets and read time-synched experiment data from them
  (for recording, viewing or experiment control).

This is also sometimes called a pub/sub (as in publish/subscribe) system.

This create provides safe bindings to the [liblsl](https://github.com/sccn/liblsl) system library
via the low-level `lsl-sys` crate.
*/

use lsl_sys::*;
use std::ffi;
mod utils;

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
    /// For variable-length strings or data blobs, such as video frames, complex event
    /// descriptions, etc.
    String = 3,
    /// For high-rate digitized formats that require 32-bit precision. Depends critically on
    /// meta-data to represent meaningful units. Useful for application event codes or other
    /// coded data.
    Int32 = 4,
    /// For very high rate signals (40Khz+) such as consumer-grade audio
    /// (for professional audio, float is recommended).
    Int16 = 5,
    /// For binary signals or other coded data. Not recommended for encoding string data.
    Int8 = 6,
    /// Note that support for this type is not yet exposed in all languages.
    /// Also, some builds of liblsl (e.g., on 32-bit systems) will not be able to send or receive
    /// data of this type.
    Int64 = 7,
    /// Can not be transmitted. This is treated as an error/unknown value when used in conjunction
    /// with any of the API methods.
    Undefined = 0,
}

// internal implementation for channel format
impl ChannelFormat {
    // Convert to corresponding low-level data type
    fn to_ffi(&self) -> lsl_channel_format_t {
        match self {
            ChannelFormat::Float32 => lsl_channel_format_t_cft_float32,
            ChannelFormat::Double64 => lsl_channel_format_t_cft_double64,
            ChannelFormat::String => lsl_channel_format_t_cft_string,
            ChannelFormat::Int32 => lsl_channel_format_t_cft_int32,
            ChannelFormat::Int16 => lsl_channel_format_t_cft_int16,
            ChannelFormat::Int8 => lsl_channel_format_t_cft_int8,
            ChannelFormat::Int64 => lsl_channel_format_t_cft_int64,
            ChannelFormat::Undefined => lsl_channel_format_t_cft_undefined,
        }
    }

    // Convert from the corresponding low-level data type
    fn from_ffi(fmt: lsl_channel_format_t) -> ChannelFormat {
        #[allow(non_upper_case_globals)]
        match fmt {
            lsl_channel_format_t_cft_float32 => ChannelFormat::Float32,
            lsl_channel_format_t_cft_double64 => ChannelFormat::Double64,
            lsl_channel_format_t_cft_string => ChannelFormat::String,
            lsl_channel_format_t_cft_int32 => ChannelFormat::Int32,
            lsl_channel_format_t_cft_int16 => ChannelFormat::Int16,
            lsl_channel_format_t_cft_int8 => ChannelFormat::Int8,
            lsl_channel_format_t_cft_int64 => ChannelFormat::Int64,
            // Note that this will convert any unknown values that come ouf of the lib into Undefined
            _ => ChannelFormat::Undefined
        }
    }
}


/// Post-processing options for stream inlets.
#[derive(Copy, Clone, Debug)]
pub enum ProcessingOptions {
    /// No automatic post-processing; return the ground-truth time stamps for manual post-
    /// processing (this is the default behavior of the inlet).
    None = 0,
    /// Perform automatic clock synchronization; equivalent to manually adding the value returned
    /// by the `time_correction()` method to the received time stamps.
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


/**
Protocol version.
- The major version is protocol_version() / 100;
- The minor version is protocol_version() % 100;

Clients with different minor versions are protocol-compatible with each other
while clients with different major versions will refuse to work together.
*/
pub fn protocol_version() -> i32 {
    unsafe {
        lsl_protocol_version()
    }
}

/**
Version of the liblsl library.
- The major version is library_version() / 100;
- The minor version is library_version() % 100;
*/
pub fn library_version() -> i32 {
    unsafe {
        lsl_library_version()
    }
}


/**
Get a string containing library information. The format of the string shouldn't be used
for anything important except giving a a debugging person a good idea which exact library
version is used.
*/
pub fn library_info() -> String {
    unsafe {
        ffi::CStr::from_ptr(lsl_library_info()).to_string_lossy().into_owned()
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


// ==========================
// === Stream Declaration ===
// ==========================

/**
The StreamInfo object stores the declaration of a data stream.
Represents the following information:
* stream data format (number of channels, channel format)
* core information (stream name, content type, sampling rate)
* optional meta-data about the stream content (channel labels, measurement units, etc.)

Whenever a program wants to provide a new stream on the lab network it will typically first
create a `StreamInfo` to describe its properties and then construct a `StreamOutlet` with it to
create the stream on the network. The stream can then be discovered based on any of its meta-data,
and recipients who discover the stream on the network can then query the full stream information.
The information herein is also typically written to disk when recording the stream (playing a
similar role as a file header).
*/
pub struct StreamInfo {
    handle: lsl_streaminfo,
}


impl StreamInfo {
    /** Construct a new `StreamInfo` object.
    Core stream information is specified here. Any remaining meta-data can be added subsequently.

    Arguments:
    * `stream_name`: Name of the stream. Describes the device (or product series) that this stream
       makes available (for use by programs, experimenters or data analysts). Cannot be empty.
    * `stream_type`: Content type of the stream.
       Please see [here](https://github.com/sccn/xdf/wiki/Meta-Data) (or web search for: XDF
       meta-data) for pre-defined content-type names that LSL adheres to, but you can also make
       up your own. The content type is the preferred way to find streams (as opposed to searching
       by name).
    * `channel_count`: Number of channels per sample. This stays constant for the lifetime of the
       stream.
    * `nominal_srate`: The sampling rate (in Hz) as advertised by the data source, if regular
       (otherwise set to `lsl::IRREGULAR_RATE`).
    * `channel_format`: Format/type of each channel. If your channels have different formats,
       consider supplying multiple streams or use the largest type that can hold them all (such as
       `ChannelFormat::Double64`).
    * `source_id`: Unique identifier of the device or source of the data, if available (such as
       the serial number). This is critical for system robustness since it allows recipients to
       recover from failure even after the serving app, device or computer crashes (just by finding
       a stream with the same source id on the network again). Therefore, it is highly recommended
       to always try to provide whatever information can uniquely identify the data source itself.
       If you don't have a unique id, you may use an empty str here.
    */
    pub fn new(stream_name: &str, stream_type: &str, channel_count: i32, nominal_srate: f64,
               channel_format: ChannelFormat, source_id: &str) -> Result<StreamInfo, &'static str>
    {
        unsafe {
            let stream_name = match ffi::CString::new(stream_name) {
                Ok(n) => n,
                Err(_) => return Err("could not convert stream_name to CString \
                                      (contained 0 bytes?)"),
            };
            let stream_type = match ffi::CString::new(stream_type) {
                Ok(t) => t,
                Err(_) => return Err("could not convert stream_type to CString \
                                      (contained 0 bytes?)"),
            };
            let source_id = match ffi::CString::new(source_id) {
                Ok(i) => i,
                Err(_) => return Err("could not convert source_id to CString \
                                      (contained 0 bytes?)"),
            };
            let handle = lsl_create_streaminfo(
                stream_name.as_ptr(),
                stream_type.as_ptr(),
                channel_count,
                nominal_srate,
                channel_format.to_ffi(),
                source_id.as_ptr()
            );
            match handle.is_null() {
                false => Ok(StreamInfo { handle }),
                true => Err("Failed to create StreamInfo struct.")
            }
        }
    }

    // ========================
    // === Core Information ===
    // ========================
    // (these fields are assigned at construction)

    /**
    Name of the stream.
    This is a human-readable name. For streams offered by device modules, it refers to the type of
    device or product series that is generating the data of the stream. If the source is an
    application, the name may be a more generic or specific identifier. Multiple streams with the
    same name can coexist, though potentially at the cost of ambiguity (for the recording app or
    experimenter).
    */
    pub fn stream_name(&self) -> String {
        unsafe {
            ffi::CStr::from_ptr(lsl_get_name(self.handle)).to_string_lossy().into_owned()
        }
    }

    /**
    Content type of the stream.
    The content type is a short string such as "EEG", "Gaze" which describes the content carried
    by the channel (if known). If a stream contains mixed content this value need not be assigned
    but may instead be stored in the description of channel types. To be useful to applications and
    automated processing systems, using the recommended content types is preferred. Content types
    usually follow those pre-defined [here](https://github.com/sccn/xdf/wiki/Meta-Data) (or web
    search for: XDF meta-data).
    */
    pub fn stream_type(&self) -> String {
        unsafe {
            ffi::CStr::from_ptr(lsl_get_type(self.handle)).to_string_lossy().into_owned()
        }
    }

    /**
    Number of channels of the stream.let fm
    A stream has at least one channel; the channel count stays constant for all samples.
    */
    pub fn channel_count(&self) -> i32 {
        unsafe {
            lsl_get_channel_count(self.handle)
        }
    }

    /**
    Sampling rate of the stream, according to the source (in Hz).
    If a stream is irregularly sampled, this should be set to `lsl::IRREGULAR_RATE`.

    Note that no data will be lost even if this sampling rate is incorrect or if a device has
    temporary hiccups, since all samples will be recorded anyway (except for those dropped by the
    device itself). However, when the recording is imported into an application, a good importer
    may correct such errors more accurately if the advertised sampling rate was close to the specs
    of the device.
    */
    pub fn nominal_srate(&self) -> f64 {
        unsafe {
            lsl_get_nominal_srate(self.handle)
        }
    }

    /** Channel format of the stream.
    All channels in a stream have the same format. However, a device might offer multiple
    time-synched streams each with its own format.
    */
    pub fn channel_format(&self) -> ChannelFormat {
        unsafe {
            ChannelFormat::from_ffi(lsl_get_channel_format(self.handle))
        }
    }

}


impl Drop for StreamInfo {
    fn drop(&mut self) {
        unsafe {
            lsl_destroy_streaminfo(self.handle);
        }
    }
}


impl Clone for StreamInfo {
    fn clone(&self) -> StreamInfo {
        unsafe {
            let handle = lsl_copy_streaminfo(self.handle);
            if handle.is_null() {
                // this should really only happen if there's an out of memory error
                panic!("Failed to clone underlying lsl_streaminfo object.")
            }
            StreamInfo { handle }
        }
    }
}
