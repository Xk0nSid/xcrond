use chrono::{Local, DateTime};
use std::cmp::Ordering;
use crate::job::Job;

#[derive(Eq, Clone)]
pub struct Event {
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

    pub fn get_jobs(&self) -> &Vec<Job> {
        &self.jobs
    }

    pub fn get_time(&self) -> DateTime<Local> {
        self.time
    }
}

#[derive(Default)]
pub struct EventQueue {
    queue: Vec<Event>,
}

impl EventQueue {

    pub fn enqueue(&mut self, j: Job) {
        if self.queue.is_empty() {
            let mut e = Event::new(j.get_next());
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
            match self.queue.binary_search_by(|probe| j.get_next().cmp(&probe.time)) {
                Ok(pos) => {
                    // Already in the vector
                    self.queue[pos].push_job(j);
                }
                Err(pos) => {
                    // Not in the vector
                    let mut e = Event::new(j.get_next());
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
