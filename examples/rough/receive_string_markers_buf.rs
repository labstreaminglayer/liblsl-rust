/** Example program to demonstrate how to read string-valued markers from LSL.*/
use lsl;
use lsl::Pullable; // trait that provides the pull_sample method
use std::vec;

fn main() -> Result<(), lsl::Error> {
    // first we resolve a stream with content type Markers
    println!("Resolving Marker stream...");
    let res = lsl::resolve_byprop("type", "Markers", 1, lsl::FOREVER)?;

    // next we create an inlet to read from the stream, using the default parameter
    let inl = lsl::StreamInlet::new(&res[0], 360, 0, true)?;

    println!("Reading data...");
    let mut sample = vec::Vec::<String>::new();
    loop {
        // get the next successive event marker that was sent and its time stamp (here: blocking)
        // in theory the stream could be multi-channel (multiple strings per sample), which is why
        // a vector is returned -- though for marker streams there's nearly always only 1 channel
        let ts = inl.pull_sample_buf(&mut sample, lsl::FOREVER)?;
        // (the type hint ^ is needed since you could also pull into a Vec<Vec<u8>> for example)
        println!("got {:?} at time {}", sample, ts);
    }
}
