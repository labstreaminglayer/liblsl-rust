/** Example program that demonstrates how to send a multi-channel time series with proper
meta-data, and also some other advanced features. */
use lsl;
use lsl::ExPushable; // trait used for push_sample_ex method
use rand::Rng; // since we're sending random data

fn main() -> Result<(), lsl::Error> {
    // This will be our stream declaration: we set the stream name to BioSemi, the content-type to
    // EEG, 8 channels, 100 Hz, and float-valued data. The last argument should be a serial number
    // or some stable identifier of the data source that's unique within your network (you can
    // omit it, but if you do, receivers of your data wouldn't seamlessly resume if you stop and
    // restart your program).
    let mut info = lsl::StreamInfo::new(
        "BioSemi", "EEG", 8, 100.0,
        lsl::ChannelFormat::Float32, "myid234365")?;

    // Next we append some meta-data. If use use a data type for which there are existing naming
    // conventions (see https://github.com/sccn/xdf/wiki/Meta-Data#stream-content-types), we
    // strongly advise to follow those for max interoperability. If you have other data and want to
    // contribute to standardizing LSL meta-data for it, PRs against that spec are always encouraged.
    let mut channels = info.desc().append_child("channels");
    // here we're declaring some channel names for our 8 channels
    for c in &["C3", "C4", "Cz", "FPz", "POz", "CPz", "O1", "O2"] {
        channels.append_child("channel")
            .append_child_value("label", c)
            .append_child_value("unit", "microvolts")
            .append_child_value("type", "EEG");
    }

    // Next we create a stream outlet. This makes our stream visible on the network.
    // We can have the data transmitted in chunks to reduce network bandwidth, and here we use 20
    // samples per chunk, and the default max buffer size of 360 seconds.
    let outlet = lsl::StreamOutlet::new(&info, 20, 360)?;

    println!("Now streaming data...");
    let mut rng = rand::thread_rng();
    loop {
        // make a new random 8-channel sample (if we used a different value type here, then LSL
        // will internally convert it to the type declared in the StreamInfo)
        let sample: Vec<f32> = (0..8).map(|_| rng.gen_range(-15.0, 15.0)).collect();
        // This is what you can do if you knew that your data was on average 53ms old by the time
        // you submitted it to LSL (e.g., due to driver delays): you back-date the timestamp
        let stamp = lsl::local_clock() - 0.053;
        // now let's send it with that stamp (the pushthrough flag (last argument) is overridden
        // by the chunk_size value that we declared on the outlet)
        outlet.push_sample_ex(&sample, stamp, true)?;
        // wait a bit until we send the next sample
        // note that, in practice, your actual samples per second isn't going to be exactly what
        // you declared in the stream info -- that's why that info calls it the "nominal" sampling
        // rate, as opposed to the *effective* sampling rate of the data
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
