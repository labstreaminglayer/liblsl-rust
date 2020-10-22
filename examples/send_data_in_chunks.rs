/** Example program that shows how to send multi-channel data in chunks.
This application pretends to stream data from a BioSemi (tm) EEG device into LSL. */

use lsl;
use lsl::Pushable; // trait used by the outlet object
use rand::Rng; // since we're sending random data
use std::vec;

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
        // generate a chunk of random data (here 10 samples x 8 channels)
        let mut mychunk = vec::Vec::new();
        for _ in 0..10 {
            let sample: Vec<f32> = (0..8).map(|_| rng.gen_range(-15.0, 15.0)).collect();
            mychunk.push(sample);
        }
        // now send it (the chunk size is up to you and you can even vary it from call to call)
        outlet.push_chunk(&mychunk)?;
        // wait for a bit until our next chunk
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
