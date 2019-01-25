use xcrond::*;

fn main() {
    let mut c = Cron::new();
    c.init();
    c.run();
}
