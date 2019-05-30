#[macro_use]
extern crate log;

mod event;
mod job;

use chrono::DateTime;
use chrono::Local;
use env_logger::{Builder, Target};
use log::{error, info};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{execv, fork, getpid, ForkResult, Pid};
use std::thread;
use std::time;

use event::EventQueue;
use job::Job;

#[derive(Default)]
pub struct Cron {
    job_list: EventQueue,
    wakeup_after: time::Duration,
}

impl Cron {
    /// Initialize the cron instance.
    /// This function reads all schedule files and prepares
    /// all the necessary data structures for proper operations.
    /// Any configuration related work for cron daemon should be done
    /// in this function.
    pub fn init(&mut self) {
        // 1. TODO: Read cron job files
        // 2. TODO: Parse each file
        // 3. TODO: Enqueue jobs in job_list
        //
        // This is how it should look like
        //
        // let joblist = joblist::parse_config();
        // for j in joblist {
        //     self.job_list.enqueue(Job::from_cronjob(j));
        // }

        // Initialize logger
        // TODO: Change this when we can load server config
        let mut log_builder = Builder::from_default_env();
        log_builder.target(Target::Stdout);
        log_builder.init();

        let j1 = Job::new(
            "Job 1".to_string(),
            "/usr/bin/touch /tmp/1".to_string(),
            "@minute",
        );
        let j2 = Job::new(
            "Job 2".to_string(),
            "/usr/bin/touch /tmp/2".to_string(),
            "0 0/2 * * * *",
        );
        let j3 = Job::new(
            "Job 3".to_string(),
            "/usr/bin/touch /tmp/3".to_string(),
            "0 0/3 * * * *",
        );

        self.job_list.enqueue(j1.unwrap());
        self.job_list.enqueue(j2.unwrap());
        self.job_list.enqueue(j3.unwrap());
    }

    /// This starts the actual cron server
    pub fn run(&mut self) {
        // spawn a thread for reaping zombie processes
        self.zombie_reaper();

        loop {
            self.job_list.debug_print();

            // Try to dequeue
            let top = match self.job_list.dequeue() {
                Some(t) => t,
                None => {
                    // Shutdown server
                    // Why?
                    // Because if queue is empty, it means no jobs are scheduled.
                    // After adding job(s) to Jobfile, the server has to be restarted
                    // which will popluate the queue automatically. So, if queue
                    // is empty, then it will remain empty until next server restart.
                    //
                    // This will change when we add support for runtime population of queue.
                    // Till then we can simply shutdown the server, as it's basically of
                    // no use if the queue is empty.
                    info!("There are no jobs to execute");
                    break;
                }
            };

            // 1. Calculate wakeup after
            let wakeup_after = match top.get_time().signed_duration_since(Local::now()).to_std() {
                Ok(t) => t,
                Err(err) => {
                    error!(
                        "Failed to calculate time difference for time {}: {}",
                        top.get_time(),
                        err
                    );
                    thread::sleep(time::Duration::from_secs(60));
                    continue;
                }
            };
            self.wakeup_after = time::Duration::new(wakeup_after.as_secs(), 0);

            info!("Next exec after time {:?}", self.wakeup_after);

            // 2. sleep for wakeup_after duration
            thread::sleep(self.wakeup_after);

            for j in top.get_jobs() {
                // 4. fork process
                match fork() {
                    Ok(ForkResult::Child) => {
                        let path = &j.get_params()[0];

                        // 5. execve job on forked process
                        match execv(path, &j.get_params()[..]) {
                            Ok(_) => {
                                info!("[{}] Launched process {}", j.get_name(), getpid());
                            }
                            Err(err) => {
                                error!("Failed to execute `{:?}` in pid `{}`: {:?}", path, getpid(), err);
                            }
                        }
                    }
                    Ok(ForkResult::Parent {child}) => {
                        info!("[{}] Spawned child {}", j.get_name(), child);

                        let time_diff = DateTime::from(time::SystemTime::now() + time::Duration::from_secs(1));

                        if !j.get_schedule().after(&time_diff).peekable().peek().is_some() {
                            info!("Job Schedule Finished: {:?}", j.get_name());
                            continue;
                        }

                        // Requeue /w new `next`
                        let mut j_new = j.clone();
                        j_new.set_prev(j.get_next());
                        // This unwrap should fail in theory as we are checking `is_some` above
                        j_new.set_next(j.get_schedule().after(&time_diff).next().unwrap());
                        debug!("New Job: {:?}", j_new);
                        self.job_list.enqueue(j_new);
                    }
                    Err(_) => error!("Forking should never fail!!!.
                    If you are seeing this message, then you have much more serious problems than this server failing."),
                }
            }
        }
    }

    /// zombie_reaper spawns a thread to reap zombie processes
    fn zombie_reaper(&self) {
        thread::spawn(|| loop {
            match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
                Ok(s) => match s {
                    WaitStatus::Exited(pid, code) => {
                        info!("[Reaper] Process {} exited with code {}", pid, code)
                    }
                    WaitStatus::Stopped(pid, signal) => {
                        info!("[Reaper] Process {} stopped by signal {:?}", pid, signal)
                    }
                    WaitStatus::Signaled(pid, signal, _) => info!(
                        "[Reaper] Process {} signaled to stop with {:?}",
                        pid, signal
                    ),
                    _ => {
                        info!("[Reaper] Wait Signal: {:?}", s);
                        thread::sleep(time::Duration::from_secs(60));
                        continue;
                    }
                },
                Err(e) => {
                    info!("[Reaper] No childs present: {:?}", e);
                    thread::sleep(time::Duration::from_secs(60));
                    continue;
                }
            }
        });
    }
}
