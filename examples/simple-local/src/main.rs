#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_simple_local_example::run_simple_local() {
        Ok(summary) => {
            println!(
                "simple-local: replay verified ({} events, {} produced refs)",
                summary.event_count,
                summary.produced_refs.len()
            );
        }
        Err(error) => {
            eprintln!("simple-local: {error}");
            std::process::exit(1);
        }
    }
}
