//use std::ffi::CString;
//use nix::unistd::{ForkResult, fork, execv, getppid, getpid};
use xcrond::*;

fn main() {
    //match fork() {
    //    Ok(ForkResult::Parent {child}) => {
    //        println!("new child has pid: {}, spawned by parent: {}", child, getppid());
    //    }
    //    Ok(ForkResult::Child) => {
    //        println!("I'm new child {}", getpid());
    //        let cmd: CString = CString::new("/usr/bin/touch").unwrap();
    //        let params: &[CString] = &[CString::new("/usr/bin/touch").unwrap(), CString::new("/tmp/rust_execv.test").unwrap()];
    //        execv(&cmd, params).unwrap();
    //    }
    //    Err(_) => eprintln!("Forking should never fail. If you are seeing this message, then you have more serious problems that this program failing."),
    //}
    let mut c = Cron::new();
    c.init();
    c.run();
}
