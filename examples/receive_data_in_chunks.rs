/** Example program showing how to read a multi-channel time-series in LSL in a chunk-by-chunk
manner, which is an easy way to read in a non-blocking fashion.*/
use lsl;
use lsl::Pullable; // trait that provides the pull_chunk method

fn main() -> Result<(), lsl::Error> {
    // first we're resolving a stream with content type EEG on the network
    println!("Resolving EEG stream...");
    let res = lsl::resolve_byprop("type", "EEG", 1, lsl::FOREVER)?;

    // next we create an inlet to read from the stream, using default parameters
    let inl = lsl::StreamInlet::new(&res[0], 360, 0, true)?;

    println!("Reading data...");
    loop {
        // read all new samples and their time stamps since our last call
        let (samples, stamps): (Vec<Vec<f32>>, _) = inl.pull_chunk()?;
        // print them
        for k in 0..samples.len() {
            println!("got {:?} at time {}", samples[k], stamps[k]);
        }
        println!("---");
        // wait for some amount until we read a new chunk -- how long you wait depends on your
        // application, but if we don't sleep then the loop would spin very fast and pin a CPU core
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
