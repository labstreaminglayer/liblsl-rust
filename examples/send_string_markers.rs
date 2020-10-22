/** Example program showing how to send string-valued event markers into LSL.*/

use lsl;
use lsl::Pushable; // trait used by the outlet object
use rand::Rng;

fn main() -> Result<(), lsl::Error> {
    // This will be our stream declaration: we set the stream name to MyMarkerStream, the
    // content-type to Markers, 1 channel, irregular sampling rate, and string-valued data.
    // The last argument should be a stable unique identifier of the source of the events (if
    // available), or you might make something up.
    let info = lsl::StreamInfo::new(
        "MyMarkerStream", "Markers", 1, lsl::IRREGULAR_RATE,
        lsl::ChannelFormat::String, "myuid-5641237")?;

    // Next we create a stream outlet. This makes our stream visible on the network.
    // We use the recommended defaults for the chunk size and max buffer size.
    let outlet = lsl::StreamOutlet::new(&info, 0, 360)?;

    println!("Now sending markers...");
    let mut rng = rand::thread_rng();
    loop {
        // choose some random marker string to send
        let mymrk = ["Hello", "test", "blah"][rng.gen_range(0, 3)];
        // now send it (as a 1-element vector of strings, since we have 1 channel)
        // if you need to send binary blobs you can also pass a Vec<Vev<u8>> here, but you shouldn't
        // use the "Markers" content type then (which is intended for utf8-encoded string data)
        outlet.push_sample(&vec!(mymrk))?;
        // wait for a random amount of time until we send the next marker
        std::thread::sleep(std::time::Duration::from_millis(rng.gen_range(0, 3000)));
    }
}
