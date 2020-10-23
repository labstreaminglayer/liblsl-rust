/** Example program that demonstrates how to send a multi-channel time series to LSL.
This application pretends to stream data from a BioSemi (tm) EEG device into LSL. See also
send_data_advanced.rs for more advanced use cases. */
use lsl;
use lsl::Pushable; // trait used by the outlet object
use rand::Rng; // since we're sending random data

fn main() -> Result<(), lsl::Error> {
    // This will be our stream declaration: we set the stream name to BioSemi, the content-type to
    // EEG, 8 channels, 100 Hz, and float-valued data. The last argument uniquely identifies your
    // data source so that your recipients can seamlessly resume when you restart your program (
    // optional but recommended).
    let info = lsl::StreamInfo::new(
        "BioSemi", "EEG", 8, 100.0,
        lsl::ChannelFormat::Float32, "myid234365")?;

    // Next we create a stream outlet. This makes our stream visible on the network.
    // We use the recommended defaults for the chunk size and max buffer size.
    let outlet = lsl::StreamOutlet::new(&info, 0, 360)?;

    println!("Now streaming data...");
    let mut rng = rand::thread_rng();
    loop {
        // make a new random 8-channel sample
        let sample: Vec<f32> = (0..8).map(|_| rng.gen_range(-15.0, 15.0)).collect();
        // now send it
        outlet.push_sample(&sample)?;
        // wait a bit until we send the next sample
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
