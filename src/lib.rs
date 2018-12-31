use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Job {
    // prev: u32,
    cmd: String,
    // next: u32,
}

impl Job {
    pub fn new(c: &str) -> Self {
        Job { cmd: c.to_string() }
    }
}

#[derive(Debug, Eq, Clone)]
pub struct Event {
    time: u32,
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
    pub fn new(t: u32) -> Self {
        Event {
            time: t,
            jobs: vec![],
        }
    }
}

#[derive(Debug)]
pub struct EventQueue {
    queue: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue { queue: vec![] }
    }

    pub fn enqueue(&mut self, j: Job, t: u32) {
        if self.queue.is_empty() {
            let mut e = Event::new(t);
            e.jobs.push(j);
            self.queue.push(e);
        } else {
            // Algorithm for enqueuing
            // 1. if event exists in queue, append job(s) from event into existing event
            // 2. else push the event in correct position

            // Note that the binary search is done using t.cmp and not probe.cmp
            // This is done because we want the binary search to work in reverse order
            // rather than traditional order because we are maintainig the dequeue
            // in reverse order
            match self.queue.binary_search_by(|probe| t.cmp(&probe.time)) {
                Ok(pos) => {
                    // Already in the vector
                    self.queue[pos].jobs.push(j);
                }
                Err(pos) => {
                    // Not in the vector
                    let mut e = Event::new(t);
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    /// This is base test for the queue that we use as core of the cron
    /// If this passes, it means the core data structure and it's operations
    /// are performed successfully
    fn enqueue_basic_functionality() {
        let j1 = Job::new("Job 1");
        let j2 = Job::new("Job 2");
        let j3 = Job::new("Job 3");
        let j4 = Job::new("Job 4");

        let mut q = EventQueue::new();

        assert_eq!(q.queue.is_empty(), true);

        // Check enqueue operations
        q.enqueue(j1, 16);
        q.enqueue(j2, 12);
        q.enqueue(j3, 13);
        q.enqueue(j4, 13);

        // Check Top, Pop operations and ordering of queue

        assert_eq!(q.queue.len(), 3);

        assert_eq!(q.top().unwrap().time, 12);
        assert_eq!(q.dequeue().unwrap().time, 12);

        assert_eq!(q.top().unwrap().jobs.len(), 2);
        assert_eq!(q.dequeue().unwrap().time, 13);

        assert_eq!(q.top().unwrap().time, 16);
        assert_eq!(q.dequeue().unwrap().time, 16);

        assert_eq!(q.top(), Option::None);

        assert_eq!(q.queue.is_empty(), true);
    }
}
