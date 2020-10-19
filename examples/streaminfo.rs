use lsl;

fn main() {
    println!("Declaring a new stream info.\n");
    let info = lsl::StreamInfo::new(
        "My Stream", "EEG", 8, 128.0,
        lsl::ChannelFormat::Float32, "dsfge4646"
    ).unwrap();

    println!("The stream's information is:");
    println!("  name = {}", info.stream_name());
    println!("  type = {}", info.stream_type());
    println!("  channel_count = {}", info.channel_count());
    println!("  nominal_srate = {}", info.nominal_srate());
    println!("  channel_format = {:?}", info.channel_format());

}
