use lsl;

fn main() {
    // Query some version numbers from the library
    println!("LSL library version is: {}", lsl::library_version());
    println!("LSL protocol version is: {}", lsl::protocol_version());
    println!("LSL library info is: {}", lsl::library_info());

    // Read out the LSL clock.
    println!("The LSL clock reads {} seconds", lsl::local_clock());
}
