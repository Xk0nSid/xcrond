use xcrond::*;

fn main() {
    let mut c = Cron::default();
    c.init();
    c.run();
}
