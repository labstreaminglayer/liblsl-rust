use lsl;
use lsl::Pushable;

use std::vec;
use rand::Rng;

// send event markers
fn send_markers() {
    println!("Declaring a new stream info...");
    let info = lsl::StreamInfo::new(
        "My Markers", "Markers", 1, lsl::IRREGULAR_RATE,
        lsl::ChannelFormat::String, "gogj45tg"
    ).unwrap();

    println!("Opening outlet...");
    let outlet = lsl::StreamOutlet::new(&info, 0, 360).unwrap();

    println!("Now streaming...");
    let dur = std::time::Duration::from_millis(1000);
    loop {
        outlet.push_sample(&vec!("Hello!")).expect("push_sample failed!");
        std::thread::sleep(dur);
    }

}

// send streaming numeric data
fn send_data() {
    println!("Declaring a new stream info...");
    let info = lsl::StreamInfo::new(
        "My Stream", "EEG", 8, 100.0,
        lsl::ChannelFormat::Float32, "dsfge4646"
    ).unwrap();

    println!("Opening outlet...");
    let outlet = lsl::StreamOutlet::new(&info, 0, 360).unwrap();

    println!("Now streaming...");
    let dur = std::time::Duration::from_millis(10);
    loop {
        outlet.push_sample(&vec!(1,2,3,4,5,6,7,8)).expect("push_sample failed!");
        std::thread::sleep(dur);
    }
}

// send streaming numeric data in chunks
fn send_data_in_chunks() {
    println!("Declaring a new stream info...");
    let info = lsl::StreamInfo::new(
        "My Stream", "EEG", 8, 100.0,
        lsl::ChannelFormat::Float32, "dsfge4646"
    ).unwrap();

    println!("Opening outlet...");
    let outlet = lsl::StreamOutlet::new(&info, 0, 360).unwrap();

    println!("Now streaming...");
    // we'll be sending one chunk of 10 samples every 100ms (that gives a 100Hz data rate)
    let dur = std::time::Duration::from_millis(100);
    let mut rng = rand::thread_rng();
    loop {
        let mut mychunk = vec::Vec::new();
        for _ in 0..10 {
            let sample: Vec<i32> = (0..8).map(|_| rng.gen_range(-15, 15)).collect();
            mychunk.push(sample);
        }
        outlet.push_chunk(&mychunk).expect("push_sample failed!");
        std::thread::sleep(dur);
    }
}

// show off some gymnastics that are expected to work
fn type_gymnastics() {
    // create a streaminfo
    let info = lsl::StreamInfo::new(
        "My Stream", "EEG", 8, 100.0,
        lsl::ChannelFormat::Float32, "dsfge4646"
    ).unwrap();
    // create an outlet
    let outlet = lsl::StreamOutlet::new(&info, 0, 360).unwrap();
    // get the info of that outlet
    let info2 = outlet.info().unwrap();
    // xml repr of that info
    let xml = info2.to_xml();
    println!("XML was: {}", xml);
    // create a new info from that
    let info3 = lsl::StreamInfo::from_xml(&xml).unwrap();
    // now the fields should be eqiv to the original info
    println!("  name after round-tripping through .info().as_xml(): {}", info3.stream_name());
}


fn main() {
    send_data_in_chunks();
    send_markers();
    send_data();
    type_gymnastics();

}
