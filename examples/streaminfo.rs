use lsl;

fn main() {
    println!("Declaring a new stream info...");
    let info = lsl::StreamInfo::new(
        "My Stream", "EEG", 8, 128.0,
        lsl::ChannelFormat::Float32, "dsfge4646"
    ).unwrap();

    // add metadata
    let desc = info.desc();
    let chns = desc.append_child("channels");
    for &chn in &["C3", "C4", "Cz"] {
        chns.append_child("channel")
            .append_child_value("label", chn)
            .append_child_value("unit", "microvolts");
    }

    println!("\nThe stream's core information is:");
    println!("  name = {:?}", info.stream_name());
    println!("  type = {:?}", info.stream_type());
    println!("  channel_count = {}", info.channel_count());
    println!("  nominal_srate = {}", info.nominal_srate());
    println!("  channel_format = {:?}", info.channel_format());
    println!("  source_id = {:?}", info.source_id());

    println!("\nThe stream's hosting information is:");
    println!("  version = {}", info.version());
    println!("  created_at = {}", info.created_at());
    println!("  uid = {:?}", info.uid());
    println!("  session_id = {:?}", info.session_id());
    println!("  hostname = {:?}", info.hostname());

    println!("\nThe stream's equivalent XML representation is:");
    println!("{}", info.to_xml());

    println!("The following example queries evaluate as:");
    println!("  matches query \"type='EEG'\": {}", info.matches_query("type='EEG'"));
    println!("  matches query \"name='Stuff'\": {}", info.matches_query("name='Stuff'"));

    println!("\nThe stream's miscellaneous info is:");
    println!("  channel_bytes = {}", info.channel_bytes());
    println!("  sample_bytes = {}", info.sample_bytes());
    println!("  native_handle = {:?}", info.native_handle());

    println!();
}
