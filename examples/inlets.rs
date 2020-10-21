use lsl;
use lsl::{Pullable, StreamInlet};

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
        // show_stuff_inlet_can_do(&inl);

        println!("Reading data...");
        loop {
            let (sample, ts): (Vec<f32>, _) = inl.pull_sample(lsl::FOREVER)?;
            println!("got {:?} at time {}", sample, ts);
        }
    }
    Ok(())
}

fn receive_markers() -> Result<(), lsl::Error> {
    println!("Resolving EEG stream...");
    let res = lsl::resolve_byprop("type", "Markers", 1, 5.0)?;

    if !res.is_empty() {
        if res.len() > 1 {
            println!("More than one stream found, opening first one...")
        }
        println!("Opening inlet...");
        let inl = lsl::StreamInlet::new(&res[0], 360, 0, true)?;

        // most of the following is not done by most applications
        // show_stuff_inlet_can_do(&inl);

        println!("Reading data...");
        loop {
            let (sample, ts): (Vec<String>, _) = inl.pull_sample(lsl::FOREVER)?;
            // this one definitely wins the ugliness contest
            // let (sample, stamp) = Pullable::<f32>::pull_sample(&inl, 5.0)?;
            println!("got {:?} at time {}", sample, ts);
        }
    }
    Ok(())
}


fn receive_data_chunks() -> Result<(), lsl::Error> {
    println!("Resolving EEG stream...");
    let res = lsl::resolve_byprop("type", "EEG", 1, 5.0)?;

    if !res.is_empty() {
        if res.len() > 1 {
            println!("More than one stream found, opening first one...")
        }
        println!("Opening inlet...");
        let inl = lsl::StreamInlet::new(&res[0], 360, 0, true)?;

        // most of the following is not done by most applications
        // show_stuff_inlet_can_do(&inl);

        println!("Reading data...");
        let dur = std::time::Duration::from_millis(300);
        loop {
            let (samples, stamps): (Vec<Vec<f32>>, _) = inl.pull_chunk()?;
            for k in 0..samples.len() {
                println!("got {:?} at time {}", samples[k], stamps[k]);
            }
            println!("---");
            std::thread::sleep(dur);
        }
    }
    Ok(())
}

fn main() {
    receive_data_chunks().expect("receive data in chunks failed.");
    //receive_markers().expect("Receive markers failed.");
    //receive_data().expect("Receive data failed.")
}
