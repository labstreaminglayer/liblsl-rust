/** Example program to show how to read a multi-channel time series from LSL. */
use lsl;
use lsl::Pullable; // trait used by the inlet

fn main() -> Result<(), lsl::Error> {
    // first we're resolving a stream with content type EEG on the network
    println!("Resolving EEG stream...");
    let res = lsl::resolve_byprop("type", "EEG", 1, lsl::FOREVER)?;

    // next we're creating an inlet to read from it, using the recommended default arguments
    // for the max buffer length, chunk size, and whether LSL should attempt to recover the stream
    let inl = lsl::StreamInlet::new(&res[0], 360, 0, true)?;

    // now we're reading data in a loop and print it as we go
    println!("Reading data...");
    loop {
        // do a blocking read to get the next successive sample and its time stamp (we need a type
        // hint since pull_sample() is defined for various data types)
        let (sample, ts): (Vec<f32>, _) = inl.pull_sample(lsl::FOREVER)?;
        println!("got {:?} at time {}", sample, ts);
    }
}
