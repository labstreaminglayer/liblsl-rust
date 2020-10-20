/*!
Rust API for the [Lab Streaming Layer](https://github.com/sccn/labstreaminglayer) (LSL).

The lab streaming layer is a pub/sub system that allows for real-time exchange of multi-channel
time series (plus arbitrary meta-data) between applications and machines on a local network (or in
a multicast group) with time synchronization. LSL is peer-to-peer and implements service discovery
to allow streams to be discoverable on the network.

The typical use case is in lab spaces to make, e.g., instrument data from a variety of devices
aaccessible in real time to client programs (e.g., experimentation scripts, recording programs,
stream viewers, or live processing software). Since LSL provides a simple uniform API, a few-line
client can receive data from devices across many device types and suppliers/vendors (such as EEG,
eye tracking, audio, human input devices, events, etc).

The API covers two areas:
- The "push API" (aka publish) allows to create stream outlets and to push data (regular or
  irregular measurement time series, event data, coded audio/video frames, etc.) into them.
- The "pull API" (aka subscribe) allows to create stream inlets and read time-synched experiment
  data from them (for recording, viewing or experiment control).

This crate provides safe bindings to the [liblsl](https://github.com/sccn/liblsl) system library
via the low-level/raw `lsl-sys` crate.
*/

use lsl_sys::*;
use std::ffi;
use std::convert::TryFrom;

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
Protocol version number.
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
Version number of the liblsl library.
- The major version is library_version() / 100;
- The minor version is library_version() % 100;
*/
pub fn library_version() -> i32 {
    unsafe {
        lsl_library_version()
    }
}


/**
Get a string containing library/build information.

The format of the string shouldn't be used for anything important except giving a debugging person
a good idea which exact library version is used.
*/
pub fn library_info() -> String {
    unsafe {
        ffi::CStr::from_ptr(lsl_library_info()).to_string_lossy().into_owned()
    }
}


/**
Obtain a local system time stamp in seconds.

The resolution is better than a millisecond. This reading can be used to assign time stamps to
samples as they are being acquired. If the "age" of a sample is known at a particular time
(e.g., from USB transmission delays), it can be used as an offset to `local_clock()` to obtain a
better estimate of when a sample was actually captured. See `StreamOutlet::push_sample()` for a
use case.
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

It represents the following information:
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
        unsafe {
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
        if let Ok(query) = ffi::CString::new(query) {
            unsafe {
                lsl_stream_info_matches_query(self.handle, query.as_ptr()) != 0
            }
        } else {
            false
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
    pub fn to_xml(&self) -> String {
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
    pub fn from_blank() -> StreamInfo {
        StreamInfo::new("untitled", "", 0, 0.0,
                        ChannelFormat::Undefined, "").unwrap()
    }

    /// Create a `StreamInfo` from an XML string.
    pub fn from_xml(xml: &str) -> Result<StreamInfo, &'static str> {
        let xml = match ffi::CString::new(xml) {
            Ok(x) => x,
            Err(_) => return Err("XML string must not contain embedded null bytes."),
        };
        unsafe {
            let handle = lsl_streaminfo_from_xml(xml.as_ptr());
            match handle.is_null() {
                false => Ok(StreamInfo { handle }),
                true => Err("Failed to create StreamInfo from XML.")
            }
        }
    }

    /// Create a `StreamInfo` from a native handle.
    /// The info object takes ownership of the handle and will deallocate it on drop.
    pub fn from_handle(handle: lsl_streaminfo) -> StreamInfo {
        if handle.is_null() {
            // panicking since this would be either a programmer error or a probably fatal lib error
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

The actual sample-pushing functionality is provided via the `Pushable` and `ExPushable` traits
below.
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
       nominal sampling rate, otherwise x100 in samples). A good default is 360, which corresponds
       to 6 minutes of data. Note that, for high-bandwidth data you should consider using a lower
       value here to avoid running out of RAM in case data have to be buffered unexpectedly.
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

    To have no timeout, you can use the value `lsl::FOREVER` here.
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
                // the handle already refers to a copy the outlet's info object so this operation
                // is trivial
                false => Ok(StreamInfo::from_handle(info_handle)),
                true => Err("Could not obtain stream info for this outlet.")
            }
        }
    }

    // Internal utility function that checks whether a given length value matches the channel count
    fn assert_len(&self, len: usize) {
        assert_eq!(len, self.channel_count, "StreamOutlet received data whose length {} does not \
                   match the outlet's channel count {}", len, self.channel_count);
    }
}

/**
A trait that enables the methods `push_sample<T>()` and `push_chunk<T>()`. Implemented by
StreamOutlet.

See also the `ExPushable` trait for the extended-argument versions of these methods,
`push_sample_ex<T>()` and `push_chunk_ex<T>()`.
*/
pub trait Pushable<T> {
    /**
    Push a vector of values of some type as a sample into the outlet.
    Each entry in the vector corresponds to one channel. The function handles type checking &
    conversion.

    The data are time-stamped with the current time (using `local_clock()`), and immediately
    transmitted (unless a `chunk_size` was provided at outlet construction, which overrides in what
    granularity data are transmitted). See also `push_chunk_ex()` (provided by `ExPushable` trait)
    for a variant that allows for overriding the timestamp and implicit push-through (queue flush)
    behavior.
    */
    fn push_sample(&self, data: &T);


    /**
    Push a chunk of samples (batched into a `Vec`) into the outlet. Each element of the given
    vector must itself be in a format accepted by `push_sample()` (e.g., `Vec`).

    The data are time-stamped with the current time (using `local_clock()`), and immediately
    transmitted (unless a `chunk_size` was provided at outlet construction, which causes the data
    to be internally re-aggregated into chunks of that specified size for transmission). See also
    `push_chunk_ex()` (provided by `ExPushable` trait) for a variant that allows for overriding the
    timestamp and implicit push-through (queue flush) behavior.
    */
    fn push_chunk(&self, data: &std::vec::Vec<T>);

    /**
    Push a chunk of samples (batched into a `Vec`) along with a separate time stamp for each
    sample (for irregular-rate streams) into the outlet.

    Arguments:
    * `samples`: A `Vec` of samples, each in a format accepted by `push_sample()` (e.g., `Vec`).
    * `timestamps`: A `Vec` of capture times for each sample, in agreement with `local_clock()`.

    The data are immediately transmitted (unless a `chunk_size` was provided at outlet
    construction, which causes the data to be internally re-aggregated into chunks of that
    specified size for ttransmission). See also `push_chunk_ex()` (provided by `ExPushable` trait)
    for a variant that allows for overriding this behavior.
    */
    fn push_chunk_stamped(&self, samples: &std::vec::Vec<T>, stamps: &std::vec::Vec<f64>);
}

// Pushable is basically a convenience layer on top of ExPushable
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

/**
A trait that enables the methods `push_sample_ex<T>()` and `push_chunk_ex<T>()`.
Implemented by StreamOutlet.

See also the `Pushable` trait for the simpler methods `push_sample<T>()` and `push_chunk<T>()`.
*/
pub trait ExPushable<T>: HasNominalRate {

    /**
    Push a vector of values of some type as a sample into the outlet.
    Each entry in the vector corresponds to one channel. The function handles type checking &
    conversion.

    Arguments:
    * `data`: A vector of values to push (one for each channel).
    * `timestamp`: Optionally the capture time of the sample, in agreement with `local_clock()`;
       if passed as 0.0, the current time is used.
    * `pushthrough`: Whether to push the sample through to the receivers instead of buffering it
       with subsequent samples. Typically this would be `true`. Note that the `chunk_size`, if
       specified at outlet construction, takes precedence over the pushthrough flag.

    See also `push_sample()` for a simpler variant with default values for `timestamp` and
    `pushthrough` (defined in `Pushable` trait).
    */
    fn push_sample_ex(&self, data: &T, timestamp: f64, pushthrough: bool);

    /**
    Push a chunk of samples (batched into a `Vec`) into the outlet.

    Arguments:
    * `samples`: A `Vec` of samples, each in a format accepted by `push_sample()` (e.g., `Vec`).
    * `timestamp`: Optionally the capture time of the most recent sample, in agreement with
       `local_clock()`; if specified as 0.0, the current time is used. The time stamps of other
       samples are automatically derived according to the sampling rate of the stream.
    * `pushthrough`: Whether to push the chunk through to the receivers instead of buffering it
       with subsequent samples. Typically this would be `true`. Note that the `chunk_size`, if
       specified at outlet construction, takes precedence over the pushthrough flag.

    See also `push_chunk()` for a simpler variant with default values for `timestamp` and
    `pushthrough` (defined in `Pushable` trait).
    */
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

    /**
    Push a chunk of samples (batched into a `Vec`) into the outlet.
    Allows for specifying a separate time stamp for each sample (for irregular-rate streams).

    Arguments:
    * `samples`: A `Vec` of samples, each in a format accepted by `push_sample()` (e.g., `Vec`).
    * `timestamps`: A `Vec` of capture times for each sample, in agreement with `local_clock()`.
    * `pushthrough`: Whether to push the chunk through to the receivers instead of buffering it
       with subsequent samples. Typically this would be `true`. Note that the `chunk_size`, if
       specified at outlet construction, takes precedence over the pushthrough flag.
    */
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

impl ExPushable<std::vec::Vec<String>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<String>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        let ptrs: Vec<_> = data.iter().map(|x| {x.as_ptr()}).collect();
        let lens: Vec<_> = data.iter().map(|x| {u32::try_from(x.len()).unwrap()}).collect();
        unsafe {
            lsl_push_sample_buftp(self.handle,
                                  ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                                  lens.as_ptr(),
                                  timestamp,
                                  pushthrough as i32);
        }
    }
}

impl ExPushable<std::vec::Vec<&str>> for StreamOutlet {
    fn push_sample_ex(&self, data: &std::vec::Vec<&str>, timestamp: f64, pushthrough: bool) {
        self.assert_len(data.len());
        let ptrs: Vec<_> = data.iter().map(|x| {x.as_ptr()}).collect();
        let lens: Vec<_> = data.iter().map(|x| {u32::try_from(x.len()).unwrap()}).collect();
        unsafe {
            lsl_push_sample_buftp(self.handle,
                                  ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                                  lens.as_ptr(),
                                  timestamp,
                                  pushthrough as i32);
        }
    }
}

impl Drop for StreamOutlet {
    fn drop(&mut self) {
        unsafe {
            lsl_destroy_outlet(self.handle);
        }
    }
}

/// Exposes a sampling rate via the property nominal_srate().
pub trait HasNominalRate {
    fn nominal_srate(&self) -> f64;
}

impl HasNominalRate for StreamOutlet {
    fn nominal_srate(&self) -> f64 {
        self.nominal_rate
    }
}


// ===========================
// ==== Resolve Functions ====
// ===========================

/**
Resolve all streams on the network.

This function returns all currently available streams from any outlet on the network.
The network is usually the subnet specified at the local router, but may also include
a multicast group of machines (given that the network supports it), or list of hostnames.
These details may optionally be customized by the experimenter in a configuration file
(see Network Connectivity in the LSL wiki).
This is the default mechanism used by the browsing programs and the recording program.

Arguments:
* `wait_time`: The waiting time for the operation, in seconds, to search for streams. A good value
   is around 1.0 or 2.0 seconds. *Warning*: If this is too short (<0.5s) only a subset (or none) of
   the outlets that are present on the network may be returned.

Returns a `Vec` of `StreamInfo` objects (excluding their desc field), any of which can subsequently
be used to open an inlet. The full info can be retrieved from the inlet if needed.
*/
pub fn resolve_streams(wait_time: f64) -> std::vec::Vec<StreamInfo> {
    // the fixed-size buffer is safe since the native function uses it as the max number of results
    let mut buffer = [0 as lsl_streaminfo; 1024];
    unsafe {
        let num_resolved = lsl_resolve_all(
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            wait_time) as usize;
        let results: Vec<_> = buffer[0..num_resolved].iter().map(
            |x| {StreamInfo::from_handle(*x)}).collect();
        results
    }
}

/**
Resolve all streams with a specific value for a given property.

If the goal is to resolve a specific stream, this method is preferred over resolving all streams
and then selecting the desired one.

Arguments:
* `prop`: The `StreamInfo` property that should have a specific value (e.g., "name", "type",
  "source_id", or "desc/manufaturer").
* `value`: The string value that the property should have (e.g., "EEG" as the type property).
* `minimum`: Return at least this number of streams.
* `timeout`: A timeout for the operation, in seconds. If the timeout expires, less than the desired
   number of streams (possibly none) will be returned. To have no timeout you can use the value
   `lsl::FOREVER` here, otherwise use at least 1.0 to 2.0 seconds to allow for results to come in
   on a busy network. *Warning*: If this is too short (<0.5s) only a subset (or none) of the
   outlets that are present on the network may be returned.

Returns a `Vec` of `StreamInfo` objects (excluding their desc field), any of which can subsequently
be used to open an inlet. The full info can be retrieved from the inlet if needed.
*/
pub fn resolve_byprop(prop: &str, value: &str, minimum: i32, wait_time: f64) -> std::vec::Vec<StreamInfo> {
    // the fixed-size buffer is safe since the native function uses it as the max number of results
    let mut buffer = [0 as lsl_streaminfo; 1024];
    let prop = match ffi::CString::new(prop) {
        Ok(s) => s,
        Err(_) => return std::vec::Vec::new(),
    };
    let value = match ffi::CString::new(value) {
        Ok(s) => s,
        Err(_) => return std::vec::Vec::new(),
    };
    unsafe {
        let num_resolved = lsl_resolve_byprop(
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            prop.as_ptr(),
            value.as_ptr(),
            minimum,
            wait_time) as usize;
        let results: Vec<_> = buffer[0..num_resolved].iter().map(
            |x| {StreamInfo::from_handle(*x)}).collect();
        results
    }
}


/**
Resolve all streams that match a given predicate.

Advanced query that allows to impose more conditions on the retrieved streams; the given
string is an [XPath 1.0](http://en.wikipedia.org/w/index.php?title=XPath_1.0) predicate evaluated
against the `<info>` element of the `StreamInfo`'s equivalent XML body (omitting the
surrounding []'s), for each stream that's on the network.

Arguments:
* `pred`: The predicate string, e.g. `name='BioSemi'` or
     `type='EEG' and starts-with(name,'BioSemi') and count(info/desc/channel)=32`
* `minimum`: Return at least this many streams.
* `timeout`: A timeout for the operation, in seconds. If the timeout expires, less than the desired
   number of streams (possibly none) will be returned. To have no timeout you can use the value
   `lsl::FOREVER` here, otherwise use at least 1.0 to 2.0 seconds to allow for results to come in
   on a busy network. *Warning*: If this is too short (<0.5s) only a subset (or none) of the
   outlets that are present on the network may be returned.

Returns a `Vec` of `StreamInfo` objects (excluding their desc field), any of which can subsequently
be used to open an inlet. The full info can be retrieved from the inlet if needed.
*/
pub fn resolve_bypred(pred: &str, minimum: i32, wait_time: f64) -> std::vec::Vec<StreamInfo> {
    // the fixed-size buffer is safe since the native function uses it as the max number of results
    let mut buffer = [0 as lsl_streaminfo; 1024];
    let pred = match ffi::CString::new(pred) {
        Ok(s) => s,
        Err(_) => return std::vec::Vec::new(),
    };
    unsafe {
        let num_resolved = lsl_resolve_bypred(
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            pred.as_ptr(),
            minimum,
            wait_time) as usize;
        let results: Vec<_> = buffer[0..num_resolved].iter().map(
            |x| {StreamInfo::from_handle(*x)}).collect();
        results
    }
}


// ======================
// ==== Stream Inlet ====
// ======================

/**
A stream inlet.
Inlets are used to receive streaming data (and meta-data) from the lab network.
*/
pub struct StreamInlet {
    // internal fields used by the Rust wrapper
    handle: lsl_inlet,
    channel_count: usize,
}

impl StreamInlet {
    /**
    Construct a new stream inlet from a resolved stream info.

    Arguments:
    * `info`: A resolved stream info object (as coming from one of the resolver functions).
       Note: the `StreamInlet` may also be constructed with a manually-constructed `StreamInfo`, if
       the desired channel format and count is already known up-front, but this is strongly
       discouraged and should only ever be done if there is no time to resolve the stream up-front
       (e.g., due to limitations in the client program).
    * `max_buflen`: The maximum amount of data to buffer (in seconds if there is a nominal sampling
       rate, otherwise x100 in samples). Recording applications want to use a fairly large buffer
       size here (a good default would be 360, which corresponds to 6 minutes of data), while
       real-time applications would only buffer as much as they need to perform their next
       calculation (e.g., 1-10).
    * `max_chunklen`: The maximum size, in samples, at which chunks are transmitted (the default
       corresponds to the chunk sizes used by the sender). If specified as 0, the chunk sizes
       preferred by the sender are used. Recording applications can use a generous size here
       (leaving it to the network how to pack things), while real-time applications may want a
       finer (perhaps 1-sample) granularity.
    * `recover`: Try to silently recover lost streams that are recoverable (those that that
       have a `source_id` set). In all other cases (`recover` is `false` or the stream is not
       recoverable) inlet methods may throw a `LostError` if the stream's source is lost (e.g.,
       due to an app or computer crash).
    */
    pub fn new(info: &StreamInfo, max_buflen: i32, max_chunklen: i32, recover: bool) -> Result<StreamInlet, &'static str> {
        let channel_count = info.channel_count() as usize;
        unsafe {
            let handle = lsl_create_inlet(info.handle(), max_buflen, max_chunklen,
                                          recover as i32);
            match handle.is_null() {
                false => Ok(StreamInlet { handle, channel_count }),
                true => Err("Could not create inlet from provided StreamInfo."),
            }
        }
    }

    /**
    Retrieve the complete information of the given stream, including the extended description.
    Can be invoked at any time of the stream's lifetime.

    Arguments:
    * `timeout`: Timeout of the operation. You can use the value `lsl::FOREVER` to have no timeout.

    */
    pub fn info(&self, timeout: f64) -> Result<StreamInfo, &'static str> {
        unsafe {
            let mut ec = [0 as i32];
            let handle = lsl_get_fullinfo(self.handle, timeout, ec.as_mut_ptr());
            StreamInfo // TODO: finish
            //check_error(ec);
            //return stream_info(res);
        }
    }
    // stream_info info(double timeout=FOREVER) { int32_t ec=0; lsl_streaminfo res = lsl_get_fullinfo(obj,timeout,&ec); check_error(ec); return stream_info(res); }
    // int32_t ec=0; lsl_streaminfo res = lsl_get_fullinfo(obj,timeout,&ec); check_error(ec); return stream_info(res);

}


//    * `timeout_error` (if the timeout expires), or lost_error (if the stream source has been lost).

impl Drop for StreamInlet {
    fn drop(&mut self) {
        unsafe {
            lsl_destroy_inlet(self.handle);
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

/*// Check an error code and translate into a Result
fn check_error(ec: i32) -> Result<Ok, > {
    Ok(Ok)
}
*/