/** Example program to show Some advanced capabilities when reading data.*/

use lsl;
use lsl::Pullable; // trait used by the inlet
use ::std::*;  // for user input

fn main() -> Result<(), lsl::Error> {
    println!("Resolving EEG stream...");
    // First we're resolving a stream that matches a given predicate (see function documentation).
    // You're not limited to the default meta-data here but can query against anything under desc/
    // Some applications may use a finite timeout, e.g., 30 seconds, and some may also use a short
    // 2-3s timeout (assuming that the stream is already present), but allow more than one return
    // value and then to warn the user if there were multiple matching streams present.
    let res = lsl::resolve_bypred("name='BioSemi' and type='EEG'", 1,
                                                 lsl::FOREVER)?;

    // Next we're creating an inlet to read from it. Let's say this is a real-time processing tool,
    // and we have no use for more than 10 seconds of data backlog accumulating in case our program
    // stalls for a while, so we set the max buffer length to 10s. We'll also ask that the data be
    // transmitted in chunks of 20 samples at a time, e.g., to save network bandwidth.
    let inl = lsl::StreamInlet::new(&res[0], 10, 8, true)?;

    // now that we have the inlet we can use it to retrieve the full StreamInfo object from it
    // (since custom meta-data could in theory be gigabytes, this is not transmitted by the resolve
    // call)
    let info = inl.info(5.0)?;

    // we can now traverse the extended meta-data of the stream to get the information we need
    // (usually we'll want at least the channel labels, which are typically stored as below)
    println!("\nThe channel labels were:");
    let mut cursor = info.desc().child("channels").child("channel");
    for _ in 0..info.channel_count() {
        print!("  {}", cursor.child_value_named("label"));
        cursor = cursor.next_sibling();
    }
    // ... alternatively we could get an XML string and parse it using some other crate
    println!("\n\nThe StreamInfo's full XML dump is: {}", info.to_xml()?);

    println!("Press [Enter] to continue");
    let mut ret = String::new();
    io::stdin().read_line(&mut ret).expect("stdin read error");

    // let's also suppose that we want to sync the received data's time stamps with our local_clock(),
    // e.g., to relate the data to some local events. We can enable that via post-processing, but
    // see also the inlet's time_correction() method for the manual way that gives you full control
    inl.set_postprocessing(&[lsl::ProcessingOption::ClockSync, lsl::ProcessingOption::Dejitter]);


    // now we're reading data in a loop and print it as we go
    println!("Reading data...");
    loop {
        // do a blocking read to get the next successive sample and its time stamp (we need a type
        // hint since pull_sample() is defined for various data types)
        let (sample, ts): (Vec<f32>, _) = inl.pull_sample(lsl::FOREVER)?;
        println!("got {:?} at time {}", sample, ts);
    }
}
