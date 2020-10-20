use lsl;
use lsl::Pullable;

fn show_stuff_inlet_can_do(inl: &lsl::StreamInlet) {
    let info = inl.info(5.0).unwrap();
    println!("Full XML was: {}", info.to_xml());

    println!("Querying time correction for a while because we feel like it...");
    let dur = std::time::Duration::from_secs(1);
    for _ in 0..5 {
        let corr = inl.time_correction_ex(5.0).unwrap();
        println!("  {:?}", corr);
        std::thread::sleep(dur);
    }

    println!("Enabling post-processing...");
    inl.set_postprocessing(&[lsl::ProcessingOption::ALL]);
    println!("Setting smoothing halftime...");
    inl.smoothing_halftime(90.0);
    println!("Samples available: {}", inl.samples_available());
    println!("Was clock reset: {}", inl.was_clock_reset());
}

fn receive_data() -> Result<(), lsl::Error> {
    println!("Resolving EEG stream...");
    let res = lsl::resolve_byprop("type", "EEG", 1, 5.0)?;

    if !res.is_empty() {
        if res.len() > 1 {
            println!("More than one stream found, opening first one...")
        }
        println!("Opening inlet...");
        let inl = lsl::StreamInlet::new(&res[0], 360, 0, true)?;

        // most of the following is not done by most applications
        show_stuff_inlet_can_do(&inl);

        println!("Reading data...");
        loop {
            let (sample, stamp) = inl.pull_sample(5.0)?;
            println!("got {:?} at {}", sample, stamp);
        }
    }
    Ok(())
}


fn main() {
    receive_data().expect("Receive data failed.")
}
