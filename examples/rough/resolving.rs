
fn main() -> Result<(), lsl::Error> {
    println!("Resolving streams...");
    let res = lsl::resolve_streams(2.0).unwrap();
    println!("Found {} streams:", res.len());
    for stream in &res {
        println!("\n{}\n", stream.to_xml());
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

    println!("Launching a continuous resolver...");
    let resolver = lsl::ContinuousResolver::new(5.0)?;
    let dur = std::time::Duration::from_millis(100);
    loop {
        let list = resolver.results()?;
        let names: Vec<_> = list.iter().map(|x| { x.stream_name() }).collect();
        println!("Streams on the network: {:?}", names);
        std::thread::sleep(dur);
    }

}
