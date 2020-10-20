use lsl::*;

fn main() {
    println!("Resolving streams...");
    let res = lsl::resolve_streams(2.0).unwrap();
    println!("Found {} streams:", res.len());
    for stream in &res {
        println!("\n{}\n", stream.to_xml());
    }

    if !res.is_empty() {
        println!("Attempting to open inlet...");
        let inl = lsl::StreamInlet::new(&res[0], 360, 0, true).unwrap();
        let info = inl.info(5.0).unwrap();
        println!("Full XML was: {}", info.to_xml());

        println!("Querying time correction for a while...");
        let dur = std::time::Duration::from_secs(1);
        for _ in 0..5 {
            let corr = inl.time_correction_ex(5.0).unwrap();
            println!("  {:?}", corr);
            std::thread::sleep(dur);
        }

        println!("Enabling post-processing...");
        inl.set_postprocessing(&[ProcessingOption::ALL]);
        println!("Setting smoothing halftime...");
        inl.smoothing_halftime(90.0);
        println!("Samples available: {}", inl.samples_available());
        println!("Was clock reset: {}", inl.was_clock_reset());

    }


    let prop = "type";
    let value = "EEG";
    println!("Resolving streams by {}={:?}...", prop, value);
    let res = lsl::resolve_byprop(prop, value, 1, 5.0).unwrap();
    println!("Found {} streams.", res.len());

    let pred = "type='EEG'";
    println!("Resolving streams by {:?}...", pred);
    let res = lsl::resolve_bypred(pred, 1, 5.0).unwrap();
    println!("Found {} streams.", res.len());

}
