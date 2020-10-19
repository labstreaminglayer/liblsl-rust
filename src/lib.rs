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
mod utils;  // TODO: we can prob remove this

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
                channel_format.to_native(),
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
    Number of channels of the stream.
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

    /**
    Channel format of the stream.
    All channels in a stream have the same format. However, a device might offer multiple
    time-synched streams each with its own format.
    */
    pub fn channel_format(&self) -> ChannelFormat {
        unsafe {
            ChannelFormat::from_native(lsl_get_channel_format(self.handle))
        }
    }

    /** Unique identifier of the stream's source, if available.
    The unique source (or device) identifier is an optional piece of information that, if
    available, allows that endpoints (such as the recording program) can re-acquire a stream
    automatically once it is back online.
    */
    pub fn source_id(&self) -> String {
        unsafe {
            ffi::CStr::from_ptr(lsl_get_source_id(self.handle)).to_string_lossy().into_owned()
        }
    }

    // ======================================
    // === Additional Hosting Information ===
    // ======================================
    // (these fields are implicitly assigned once bound to an outlet/inlet)


    /**
    Protocol version used to deliver the stream. Formatted like `lsl::protocol_version()`.
    */
    pub fn version(&self) -> i32 {
        unsafe {
            lsl_get_version(self.handle)
        }
    }

    /**
    Creation time stamp of the stream.
    This is the time stamp when the stream was first created
    (as determined via `lsl::local_clock()` on the providing machine).
    */
    pub fn created_at(&self) -> f64 {
        unsafe {
            lsl_get_created_at(self.handle)
        }
    }

    /**
    Unique ID of the stream outlet instance (once assigned).
    This is a unique identifier of the stream outlet, and is guaranteed to be different
    across multiple instantiations of the same outlet (e.g., after a re-start).
    */
    pub fn uid(&self) -> String {
        unsafe {
            ffi::CStr::from_ptr(lsl_get_uid(self.handle)).to_string_lossy().into_owned()
        }
    }

    /**
    Session ID for the given stream.
    The session id is an optional human-assigned identifier of the recording session.
    While it is rarely used, it can be used to prevent concurrent recording activitites
    on the same sub-network (e.g., in multiple experiment areas) from seeing each other's streams
    (assigned via a configuration file by the experimenter, see Network Connectivity in the LSL
    wiki).
    */
    pub fn session_id(&self) -> String {
        unsafe {
            ffi::CStr::from_ptr(lsl_get_session_id(self.handle)).to_string_lossy().into_owned()
        }
    }

    /**
    Hostname of the providing machine.
    */
    pub fn hostname(&self) -> String {
        unsafe {
            ffi::CStr::from_ptr(lsl_get_hostname(self.handle)).to_string_lossy().into_owned()
        }
    }

    // ========================
    // === Data Description ===
    // ========================

    // TODO: desc() function


    /**
    Test whether the stream information matches the given query string.
    The query is evaluated using the same rules that govern `lsl::resolve_bypred()`.
    */
    pub fn matches_query(&self, query: &str) -> bool {
        unsafe {
            if let Ok(query) = ffi::CString::new(query) {
                lsl_stream_info_matches_query(self.handle, query.as_ptr()) != 0
            } else {
                false
            }
        }
    }

    // ===============================
    // === Miscellaneous Functions ===
    // ===============================

    /**
    Retrieve the entire streaminfo in XML format.
    This yields an XML document (in string form) whose top-level element is `<info>`. The info
    element contains one element for each field of the streaminfo class, including:

       * the core elements `<name>`, `<type>`, `<channel_count`, `<nominal_srate>`,
         `<channel_format>`, `<source_id>`
       * the misc elements `<version>`, `<created_at>`, `<uid>`, `<session_id>`,
         `<v4address>`, `<v4data_port>`, `<v4service_port>`, `<v6address>`, `<v6data_port>`,
         `<v6service_port>`
       * the extended description element `<desc>` with user-defined sub-elements.
    */
    pub fn as_xml(&self) -> String {
        unsafe {
            let tmpstr = lsl_get_xml(self.handle);
            let result = ffi::CStr::from_ptr(tmpstr).to_string_lossy().into_owned();
            lsl_destroy_string(tmpstr);
            result
        }
    }

    /// Number of bytes occupied by a channel (0 for string-typed channels).
    pub fn channel_bytes(&self) -> i32 {
        unsafe {
            lsl_get_channel_bytes(self.handle)
        }
    }

    /// Number of bytes occupied by a sample (0 for string-typed channels).
    pub fn sample_bytes(&self) -> i32 {
        unsafe {
            lsl_get_sample_bytes(self.handle)
        }
    }

    /// Get the native implementation handle.
    pub fn handle(&self) -> lsl_streaminfo {
        self.handle
    }

    /// Construct a blank `StreamInfo`.
    pub fn blank() -> StreamInfo {
        StreamInfo::new("untitled", "", 0, 0.0, ChannelFormat::Undefined, "").unwrap()
    }

    /// Create a `StreamInfo` from an XML string.
    pub fn from_xml(xml: &str) -> Result<StreamInfo, &'static str> {
        unsafe {
            let xml = match ffi::CString::new(xml) {
                Ok(x) => x,
                Err(_) => return Err("XML string must not contain embedded null bytes."),
            };
            let handle = lsl_streaminfo_from_xml(xml.as_ptr());
            match handle.is_null() {
                false => Ok(StreamInfo { handle }),
                true => Err("Failed to create StreamInfo from XML.")
            }
        }
    }

    /// Create a `StreamInfo` from a native handle.
    pub fn from_handle(handle: lsl_streaminfo) -> StreamInfo {
        if handle.is_null() {
            panic!("Attempted to create a `StreamInfo` from a NULL handle.")
        }
        StreamInfo { handle }
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


// =======================
// ==== Stream Outlet ====
// =======================

/**
A stream outlet.
Outlets are used to make streaming data (and the meta-data) available on the lab network.
*/
pub struct StreamOutlet {
    // internal fields used by the Rust wrapper
    handle: lsl_outlet,
    channel_count: usize,
    nominal_rate: f64,
}


impl StreamOutlet {
    /**
    Establish a new stream outlet. This makes the stream discoverable.

    Arguments:
    * `info`: The stream information to use for creating this stream. Stays constant over the
       lifetime of the outlet.
    * `chunk_size`: The desired chunk granularity (in samples) for transmission.
       If specified as 0, each push operation yields one chunk. Inlets can override this setting.
    * `max_buffered`: The maximum amount of data to buffer (in seconds if there is a
       nominal sampling rate, otherwise x100 in samples).  good default is 360, which corresponds
       to 6 minutes of data. Note that, for high-bandwidth data you should consider using a lower
       value here to avoid running out of RAM.
    */
    pub fn new(info: &StreamInfo, chunk_size: i32, max_buffered: i32) -> Result<StreamOutlet, &'static str> {
        unsafe {
            let handle = lsl_create_outlet(info.handle(), chunk_size, max_buffered);
            let channel_count = info.channel_count() as usize;
            let nominal_rate = info.nominal_srate();
            match handle.is_null() {
                false => Ok(StreamOutlet { handle, channel_count, nominal_rate }),
                true => Err("Could not create outlet from provided StreamInfo."),
            }
        }
    }

    // ===============================
    // === Miscellaneous Functions ===
    // ===============================

    /**
    Check whether consumers are currently registered.
    While it does not hurt, there is technically no reason to push samples if there is no consumer.
    */
    pub fn have_consumers(&self) -> bool {
        unsafe {
            lsl_have_consumers(self.handle) != 0
        }
    }

    /**
    Wait until some consumer shows up (without wasting resources).
    Returns True if the wait was successful, false if the timeout expired.
    */
    pub fn wait_for_consumers(&self, timeout: f64) -> bool {
        unsafe {
            lsl_wait_for_consumers(self.handle, timeout) != 0
        }
    }

    /**
    Retrieve the stream info provided by this outlet.
    This is what was used to create the stream (and also has the Additional Network Information
    fields assigned).
    */
    pub fn info(&self) -> Result<StreamInfo, &'static str> {
        unsafe {
            let info_handle = lsl_get_info(self.handle);
            match info_handle.is_null() {
                false => Ok(StreamInfo::from_handle(info_handle)),
                true => Err("Could not obtain stream info for this outlet.")
            }
        }
    }

    // Utility function that checks whether a given length value matches the channel count
    fn assert_len(&self, len: usize) {
        assert_eq!(len, self.channel_count, "StreamOutlet received data whose length {} does not \
                   match the outlet's channel count {}", len, self.channel_count);
    }
}

/// Exposes a sampling rate via the property nominal_srate()
pub trait HasNominalRate {
    fn nominal_srate(&self) -> f64;
}

impl HasNominalRate for StreamOutlet {
    fn nominal_srate(&self) -> f64 {
        self.nominal_rate
    }
}

/**
A trait that enables the methods push_*_ex<T>() for different values of T that are understood by
LSL (i32, f64, str, etc.) on some object. This is implemented by StreamOutlet.
*/
pub trait ExPushable<T>: HasNominalRate {

    fn push_sample_ex(&self, data: &T, timestamp: f64, pushthrough: bool);

    fn push_chunk_ex(&self, samples: &std::vec::Vec<T>, timestamp: f64, pushthrough: bool) {
        if !samples.is_empty() {
            let mut timestamp = if timestamp == 0.0 { local_clock() } else { timestamp };
            let srate = self.nominal_srate();
            let max_k = samples.len() - 1;
            // push first sample with calulated timestamp
            if srate != IRREGULAR_RATE {
                timestamp -= (max_k as f64) / srate;
            }
            self.push_sample_ex(&samples[0], timestamp,
                                pushthrough && (samples.len() == 1));
            // push successive samples with deduced stamp
            for k in 1..=max_k {
                self.push_sample_ex(&samples[k], DEDUCED_TIMESTAMP,
                                    pushthrough && (k == max_k));
            }
        }
    }

    fn push_chunk_stamped_ex(&self, samples: &std::vec::Vec<T>, timestamps: &std::vec::Vec<f64>, pushthrough: bool) {
        assert_eq!(samples.len(), timestamps.len());
        let max_k = samples.len()-1;
        // send all except last sample
        for k in 0..max_k {
            self.push_sample_ex(&samples[k], timestamps[k], false);
        }
        // send last sample with given pushthrough flag
        if !samples.is_empty() {
            self.push_sample_ex(&samples[max_k], timestamps[max_k], pushthrough);
        }
    }


}

// LSL outlets do type conversion to the defined channel format on push (even numbers to string
// if so desired), so we can safely add all push_sample_ex() overloads (n.b. except from string to
// number if the string doesn't actually represent a number, then the sample will be dropped)
impl ExPushable<std::vec::Vec<f32>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<f32>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        unsafe {
            lsl_push_sample_ftp(self.handle,data.as_ptr(), timestamp, pushthrough as i32);
        }
    }
}

impl ExPushable<std::vec::Vec<f64>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<f64>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        unsafe {
            lsl_push_sample_dtp(self.handle,data.as_ptr(), timestamp, pushthrough as i32);
        }
    }
}

impl ExPushable<std::vec::Vec<i8>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<i8>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        unsafe {
            lsl_push_sample_ctp(self.handle,data.as_ptr(), timestamp, pushthrough as i32);
        }
    }
}

impl ExPushable<std::vec::Vec<i16>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<i16>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        unsafe {
            lsl_push_sample_stp(self.handle,data.as_ptr(), timestamp, pushthrough as i32);
        }
    }
}

impl ExPushable<std::vec::Vec<i32>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<i32>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        unsafe {
            lsl_push_sample_itp(self.handle,data.as_ptr(), timestamp, pushthrough as i32);
        }
    }
}

impl ExPushable<std::vec::Vec<i64>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<i64>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        unsafe {
            lsl_push_sample_ltp(self.handle,data.as_ptr(), timestamp, pushthrough as i32);
        }
    }
}

// TODO: use lsl_push_sample_butp here and require no zero-termination
//       this saves us all the allocation work on the Rust side
impl ExPushable<std::vec::Vec<String>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<String>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        unsafe {
            let c_strings: Vec<_> = data.iter().map(|x| {ffi::CString::new(x.as_str()).unwrap()}).collect();
            let c_ptrs: Vec<_> = c_strings.iter().map(|x| {x.as_ptr()}).collect();
            let c_arr = c_ptrs.as_ptr() as *mut *const std::os::raw::c_char;
            lsl_push_sample_strtp(self.handle,c_arr, timestamp, pushthrough as i32);
        }
    }
}

// TODO: see above
impl ExPushable<std::vec::Vec<&str>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<&str>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        unsafe {
            let c_strings: Vec<_> = data.iter().map(|x| {ffi::CString::new(*x).unwrap()}).collect();
            let c_ptrs: Vec<_> = c_strings.iter().map(|x| {x.as_ptr()}).collect();
            let c_arr = c_ptrs.as_ptr() as *mut *const std::os::raw::c_char;
            lsl_push_sample_strtp(self.handle,c_arr, timestamp, pushthrough as i32);
        }
    }
}


pub trait Pushable<T> {
    fn push_sample(&self, data: &T);
    fn push_chunk(&self, data: &std::vec::Vec<T>);
    fn push_chunk_stamped(&self, samples: &std::vec::Vec<T>, stamps: &std::vec::Vec<f64>);
}

impl<T, U: ExPushable<T>> Pushable<T> for U {
    fn push_sample(&self, data: &T) {
        self.push_sample_ex(data, 0.0, true);
    }

    fn push_chunk(&self, data: &std::vec::Vec<T>) {
        self.push_chunk_ex(data, 0.0, true);
    }

    fn push_chunk_stamped(&self, samples: &std::vec::Vec<T>, stamps: &std::vec::Vec<f64>) {
        self.push_chunk_stamped_ex(samples, stamps, true);
    }
}


impl Drop for StreamOutlet {
    fn drop(&mut self) {
        unsafe {
            lsl_destroy_outlet(self.handle);
        }
    }
}


// helper functions for interop with native data types in the lsl_sys module
impl ChannelFormat {
    /// Convert to corresponding native data type.
    pub fn to_native(&self) -> lsl_channel_format_t {
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

    /// Convert from the corresponding native data type.
    pub fn from_native(fmt: lsl_channel_format_t) -> ChannelFormat {
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
