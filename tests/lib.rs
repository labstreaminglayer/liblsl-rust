use lsl;

#[test]
fn clock_is_working() {
    assert_ne!(lsl::local_clock(), 0.0);
}

#[test]
fn streaminfo_basic() {
    let info = lsl::StreamInfo::new("MyStream", "EEG", 8, 100.0, lsl::ChannelFormat::Float32, "12345").unwrap();
    assert_eq!(info.stream_name(), "MyStream");
    assert_eq!(info.stream_type(), "EEG");
    assert_eq!(info.channel_count(), 8);
    assert_eq!(info.nominal_srate(), 100.0);
    assert_eq!(info.channel_format(), lsl::ChannelFormat::Float32);
    assert_eq!(info.source_id(), "12345");
    assert!(info.matches_query("name='MyStream' and type='EEG'"));
    assert!(!info.matches_query("name='MyStream' and type='ECG'"));
    let info2 = info.clone();
    assert_eq!(info2.stream_name(), "MyStream");
}

#[test]
fn streaminfo_xml() {
    let mut info = lsl::StreamInfo::new("MyStream", "EEG", 8, 100.0, lsl::ChannelFormat::Float32, "12345").unwrap();

    let mut channels = info.desc().append_child("channels");
    let mut chn = channels.append_child("channel");
    chn.append_child_value("label", "MyChannel");
    assert_eq!(channels.child("channel").child_value_named("label"), "MyChannel");

    let xml = info.to_xml().unwrap();
    assert!(xml.contains("<name>MyStream</name>"));
    assert!(xml.contains("<label>MyChannel</label>"));
}
