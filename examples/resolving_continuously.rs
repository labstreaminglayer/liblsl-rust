/** Example program that shows how to continuously resolve streams on the network. */
use lsl;

fn main() -> Result<(), lsl::Error> {
    // we create a new resolver that resolves all streams on the network (see also new_with_prop()
    // and new_with_pred() for more targeted queries); the parameter is the duration after which
    // streams that have disappeared will be delisted from the results (5.0 is a good default)
    let resolver = lsl::ContinuousResolver::new(5.0)?;

    loop {
        // get the list of StreamInfo objects for all currently visible streams; this call is
        // instantaneous since it just returns a list that's updated internally by the resolver
        let infos = resolver.results()?;
        // get the names of those streams and print them
        let names: Vec<_> = infos.iter().map(|x| x.stream_name()).collect();
        println!("Currently visible: {:?}", names);
        // sleep a bit to not overload the cpu
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
