use nix::unistd::{execv, fork, getpid, ForkResult};
use std::cmp::Ordering;
use std::ffi::CString;
use std::thread;
use std::time;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Job {
    prev: time::SystemTime,
    cmd: String,
    params: Vec<CString>,
    next: time::SystemTime,
}

impl Job {
    pub fn new(cmd: String, next: time::SystemTime) -> Self {
        // Build params
        let mut p: Vec<CString> = vec![];
        for a in cmd.split(' ') {
            p.push(CString::new(a).unwrap());
        }

        Job {
            cmd,
            next,
            prev: time::SystemTime::now(),
            params: p,
        }
    }
}

#[derive(Debug, Eq, Clone)]
pub struct Event {
    time: time::SystemTime,
    pub jobs: Vec<Job>,
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
    pub fn new(t: time::SystemTime) -> Self {
        Event {
            time: t,
            jobs: vec![],
        }
    }
}

#[derive(Default, Debug)]
pub struct EventQueue {
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
                    self.queue[pos].jobs.push(j);
                }
                Err(pos) => {
                    // Not in the vector
                    let mut e = Event::new(j.next);
                    e.jobs.push(j);
                    self.queue.insert(pos, e);
                }
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<Event> {
        self.queue.pop()
    }

    pub fn top(&self) -> Option<Event> {
        self.queue.last().cloned()
    }
}

#[derive(Default, Debug)]
pub struct Cron {
    job_list: EventQueue,
    wakeup_after: time::Duration,
}

impl Cron {
    /// Create a new instance of Cron struct
    pub fn new(e: EventQueue) -> Self {
        Cron {
            job_list: e,
            wakeup_after: time::Duration::new(0, 0),
        }
    }

    /// Initialize the cron instance.
    /// This function reads all schedule files and prepares
    /// all the necessary data structures for proper operations.
    /// Any configuration related work for cron daemon should be done
    /// in this function.
    pub fn init(&mut self) {
        // 1. TODO: Read cron schedule files
        // 2. TODO: Parse each file
        // 3. TODO: Enqueue jobs in job_list

        let d = time::Duration::new(30, 0);

        let t1 = time::SystemTime::now() + d;
        let t2 = t1 + d * 2;
        let t3 = t2 + d * 4;
        let t4 = t2;

        let j1 = Job::new("/usr/bin/touch /tmp/1".to_string(), t1);
        let j2 = Job::new("/usr/bin/touch /tmp/2".to_string(), t2);
        let j3 = Job::new("/usr/bin/touch /tmp/3".to_string(), t3);
        let j4 = Job::new("/usr/bin/touch /tmp/4".to_string(), t4);

        self.job_list.enqueue(j1);
        self.job_list.enqueue(j2);
        self.job_list.enqueue(j3);
        self.job_list.enqueue(j4);
    }

    /// This starts the actual cron server
    pub fn run(&mut self) {
        loop {
            // 1. Calculate wakeup after
            let top = self.job_list.top().unwrap();
            self.wakeup_after = top
                .time
                .duration_since(time::SystemTime::now())
                .expect("sleep time calculation failed");
            self.wakeup_after = time::Duration::new(self.wakeup_after.as_secs(), 0);

            println!("Next exec after time {:?}", self.wakeup_after);

            // 2. sleep for wakeup_after duration
            thread::sleep(self.wakeup_after);

            // 3. dequeue element from queue
            let e = match self.job_list.dequeue() {
                Some(e) => e,
                None => {
                    eprintln!("Failed to dequeue element");
                    continue;
                }
            };

            for j in e.jobs {
                // 4. fork process
                match fork() {
                    Ok(ForkResult::Child) => {
                        let path = &j.params[0];

                        // 5. execve job on forked process
                        match execv(path, &j.params[..]) {
                            Ok(_) => {
                                println!("Ran job {} in process {}", j.cmd, getpid());
                            }
                            Err(err) => {
                                eprintln!("Failed to execute `{:?}` in pid `{}`: {:?}", path, getpid(), err);
                            }
                        }
                    }
                    Ok(ForkResult::Parent {child}) => {
                        println!("Spawned child {}", child);

                        let mut j_new = Job::new(j.cmd, j.next + time::Duration::new(120, 0));
                        j_new.prev = j.next;
                        self.job_list.enqueue(j_new);

                        // 6. goto 1
                        continue;
                    }
                    Err(_) => eprintln!("Forking should never fail!!!.
                    If you are seeing this message, then you have much more serious problems than this server failing."),
                }
            }
        }
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
        let d = time::Duration::new(120, 0); // 2 minutes

        let t1 = time::SystemTime::now();
        let t2 = t1 + d;
        let t3 = t2 + d;
        let t4 = t2.clone();

        let j1 = Job::new("Job 1".to_string(), t1);
        let j2 = Job::new("Job 2".to_string(), t2);
        let j3 = Job::new("Job 3".to_string(), t3);
        let j4 = Job::new("Job 4".to_string(), t4);

        let mut q = EventQueue::new();

        assert_eq!(q.queue.is_empty(), true);

        // Check enqueue operations
        q.enqueue(j1);
        q.enqueue(j2);
        q.enqueue(j3);
        q.enqueue(j4);

        // Check Top, Pop operations and ordering of queue

        assert_eq!(q.queue.len(), 3);

        assert_eq!(q.top().unwrap().time, t1);
        assert_eq!(q.dequeue().unwrap().time, t1);

        assert_eq!(q.top().unwrap().jobs.len(), 2);
        assert_eq!(q.dequeue().unwrap().time, t2);

        assert_eq!(q.top().unwrap().time, t3);
        assert_eq!(q.dequeue().unwrap().time, t3);

        assert_eq!(q.top(), Option::None);

        assert_eq!(q.queue.is_empty(), true);
    }
}
