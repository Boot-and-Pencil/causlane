#![forbid(unsafe_code)]
#![deny(warnings)]

fn main() {
    match causlane_facade_kernel_ergonomics_example::run_facade_kernel_ergonomics() {
        Ok(summary) => {
            println!(
                "facade-kernel-ergonomics: facade admission and frontier verified ({} accepted, {} selected, {} rejected)",
                summary.accepted_admissions,
                summary.frontier_selected,
                summary.frontier_rejections,
            );
        }
        Err(error) => {
            eprintln!("facade-kernel-ergonomics: {error}");
            std::process::exit(1);
        }
    }
}
