use lsl::*;

fn main() {
    println!("Resolving streams...");
    let res = lsl::resolve_streams(2.0);
    println!("Found {} streams:", res.len());
    for stream in res {
        println!("\n{}\n", stream.to_xml());
    }

    let prop = "type";
    let value = "EEG";
    println!("Resolving streams by {}={:?}...", prop, value);
    let res = lsl::resolve_byprop(prop, value, 1, 5.0);
    println!("Found {} streams.", res.len());

    let pred = "type='EEG'";
    println!("Resolving streams by {:?}...", pred);
    let res = lsl::resolve_bypred(pred, 1, 5.0);
    println!("Found {} streams.", res.len());

}
