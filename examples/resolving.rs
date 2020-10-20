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
