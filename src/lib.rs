#[macro_use]
extern crate log;

use cron::Schedule;
use chrono::Local;
use chrono::DateTime;
use env_logger::{Builder, Target};
use log::{error, info};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{execv, fork, getpid, ForkResult, Pid};
use std::cmp::Ordering;
use std::ffi::CString;
use std::str::FromStr;
use std::thread;
use std::time;

#[derive(Eq, PartialEq, Clone)]
struct Job {
    name: String,
    prev: DateTime<Local>,
    cmd: String,
    params: Vec<CString>,
    schedule: Schedule,
    expression: String,
    next: DateTime<Local>,
}

impl Job {
    pub fn new(name: String, cmd: String, expr: &str) -> Self {
        // Build params
        let mut p: Vec<CString> = vec![];
        for a in cmd.split(' ') {
            p.push(CString::new(a).unwrap());
        }

        let schedule = Schedule::from_str(expr).unwrap();
        let next = schedule.upcoming(Local).next().unwrap();

        Job {
            name,
            cmd,
            next,
            expression: expr.to_string(),
            schedule: schedule,
            prev: Local::now(),
            params: p,
        }
    }
}

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Job: {} ({})", self.name, self.next)
    }
}

#[derive(Eq, Clone)]
struct Event {
    time: DateTime<Local>,
    jobs: Vec<Job>,
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Time: {} -> Jobs: {:?}", self.time, self.jobs)
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.time == other.time
    }
}

impl Event {
    pub fn new(t: DateTime<Local>) -> Self {
        Event {
            time: t,
            jobs: vec![],
        }
    }

    pub fn push_job(&mut self, j: Job) {
        self.jobs.push(j)
    }
}

struct EventQueue {
    queue: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue { queue: vec![] }
    }

    pub fn enqueue(&mut self, j: Job) {
        if self.queue.is_empty() {
            let mut e = Event::new(j.next);
            e.jobs.push(j);
            self.queue.push(e);
        } else {
            // Algorithm for enqueuing
            // 1. if event exists in queue, append job(s) from event into existing event
            // 2. else push the event in correct position

            // Note that the binary search is done using j.next.cmp and not probe.cmp
            // This is done because we want the binary search to work in reverse order
            // rather than traditional order because we are maintainig the queue
            // in reverse order
            match self.queue.binary_search_by(|probe| j.next.cmp(&probe.time)) {
                Ok(pos) => {
                    // Already in the vector
                    self.queue[pos].push_job(j);
                }
                Err(pos) => {
                    // Not in the vector
                    let mut e = Event::new(j.next);
                    e.push_job(j);
                    self.queue.insert(pos, e);
                }
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<Event> {
        self.queue.pop()
    }

    pub fn debug_print(&self) {
        // print queue for debugging purpose
        debug!("Queue: {:?}", self.queue);
    }
}

pub struct Cron {
    job_list: EventQueue,
    wakeup_after: time::Duration,
}

impl Cron {
    /// Create a new instance of Cron struct
    pub fn new() -> Self {
        Cron {
            job_list: EventQueue::new(),
            wakeup_after: time::Duration::new(0, 0),
        }
    }

    /// Initialize the cron instance.
    /// This function reads all schedule files and prepares
    /// all the necessary data structures for proper operations.
    /// Any configuration related work for cron daemon should be done
    /// in this function.
    pub fn init(&mut self) {
        // 1. TODO: Read cron job files
        // Cron's `Jobsfile` format
        // - Job 1:
        //     - cmd: /usr/bin/touch /tmp/1
        //     - schedule: 0 0/1 * * * *
        // - Job 2:
        //     - cmd: /usr/bin/touch /tmp/2
        //     - schedule: 0 0/2 * * * *
        // - Job 3:
        //     - cmd: /usr/bin/touch /tmp/3
        //     - schedule: 0 0/3 * * * *

        // 2. TODO: Parse each file
        // 3. TODO: Enqueue jobs in job_list

        // Initialize logger
        let mut log_builder = Builder::from_default_env();
        log_builder.target(Target::Stdout);
        log_builder.init();

        let j1 = Job::new("Job 1".to_string(), "/usr/bin/touch /tmp/1".to_string(), "@minute");
        let j2 = Job::new("Job 2".to_string(), "/usr/bin/touch /tmp/2".to_string(), "0 0/2 * * * *");
        let j3 = Job::new("Job 3".to_string(), "/usr/bin/touch /tmp/3".to_string(), "0 0/3 * * * *");

        self.job_list.enqueue(j1);
        self.job_list.enqueue(j2);
        self.job_list.enqueue(j3);
    }

    /// This starts the actual cron server
    pub fn run(&mut self) {
        // spawn a thread for reaping zombie processes
        self.zombie_reaper();

        loop {
            self.job_list.debug_print();

            // Check if there is any thing in the queue
            let top = match self.job_list.dequeue() {
                Some(t) => t,
                None => {
                    // if queue is empty, sleep for a minute and try again
                    thread::sleep(time::Duration::from_secs(60));
                    continue;
                }
            };

            // 1. Calculate wakeup after
            let wakeup_after = match top.time.signed_duration_since(Local::now()).to_std() {
                Ok(t) => t,
                Err(err) => {
                    error!("Failed to calculate time difference for time {}: {}", top.time, err);
                    thread::sleep(time::Duration::from_secs(60));
                    continue;
                }
            };
            self.wakeup_after = time::Duration::new(wakeup_after.as_secs(), 0);

            info!("Next exec after time {:?}", self.wakeup_after);

            // 2. sleep for wakeup_after duration
            thread::sleep(self.wakeup_after);

            for j in top.jobs {
                // 4. fork process
                match fork() {
                    Ok(ForkResult::Child) => {
                        let path = &j.params[0];

                        // 5. execve job on forked process
                        match execv(path, &j.params[..]) {
                            Ok(_) => {
                                info!("Ran job {} in process {}", j.name, getpid());
                            }
                            Err(err) => {
                                error!("Failed to execute `{:?}` in pid `{}`: {:?}", path, getpid(), err);
                            }
                        }
                    }
                    Ok(ForkResult::Parent {child}) => {
                        info!("Spawned child {} for job {}", child, j.name);

                        if !j.schedule.upcoming(Local).peekable().peek().is_some() {
                            info!("Job Schedule Finished: {:?}", j.name);
                            continue;
                        }

                        // Requeue /w new `next`
                        let mut j_new = j.clone();
                        j_new.prev = j.next;
                        // In theory this unwrap should not fail because we peek into the iterator above
                        // and if it's empty we continue the loop without requeueing
                        j_new.next = j.schedule.after(&DateTime::from(time::SystemTime::now() + time::Duration::from_secs(1))).next().unwrap();
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
                    WaitStatus::Signaled(pid, signal, _) => {
                        info!("[Reaper] Process {} signaled to stop with {:?}", pid, signal)
                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    /// This is base test for the queue that we use as core of the cron
    /// If this passes, it means the core data structure and it's operations
    /// are performed successfully
    fn enqueue_basic_functionality() {
    }
}
