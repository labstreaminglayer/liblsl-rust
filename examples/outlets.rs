use lsl;

fn main() {
    println!("Declaring a new stream info...");
    let info = lsl::StreamInfo::new(
        "My Stream", "EEG", 8, 128.0,
        lsl::ChannelFormat::Float32, "dsfge4646"
    ).unwrap();

    println!("Opening outlet...");
    let outlet = lsl::StreamOutlet::new(&info, 0, 360).unwrap();

    let info2 = outlet.info().unwrap();
    let xml = info2.as_xml();
    println!("XML was: {}", xml);

    let info3 = lsl::StreamInfo::from_xml(&xml).unwrap();
    println!("  name after round-tripping through .info().as_xml(): {}", info3.stream_name());
    println!("  have consumers: {}", outlet.have_consumers());

    let dur = std::time::Duration::from_secs(30);
    std::thread::sleep(dur);
}
