use xcrond::*;

fn main() {
    // Intitialize signal handler
    ctrlc::set_handler(move || {
        println!("Terminate signal received. Exiting.");
        std::process::exit(0);
    })
    .expect("Failed to set SIGINT handler");

    let mut c = Cron::default();
    c.init();
    c.run();
}
