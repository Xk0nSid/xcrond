use chrono::{DateTime, Local};
use cron::Schedule;
use std::{ffi::CString, str::FromStr};

#[derive(Eq, PartialEq, Clone)]
pub struct Job {
    name: String,
    prev: DateTime<Local>,
    cmd: String,
    params: Vec<CString>,
    schedule: Schedule,
    expression: String,
    next: DateTime<Local>,
}

impl Job {
    pub fn new(name: String, cmd: String, expr: &str) -> Option<Self> {
        // Build params
        let mut p: Vec<CString> = vec![];
        for a in cmd.split(' ') {
            p.push(CString::new(a).unwrap());
        }

        let schedule = match Schedule::from_str(expr) {
            Ok(t) => t,
            Err(err) => {
                error!("[{}] Invalid schedule: {}", name, err);
                return None;
            }
        };

        let next = match schedule.upcoming(Local).next() {
            Some(t) => t,
            None => {
                error!("[{}] Failed to calculate upcoming schedule", name);
                return None;
            }
        };

        Some(Job {
            name,
            cmd,
            next,
            expression: expr.to_string(),
            schedule,
            prev: Local::now(),
            params: p,
        })
    }

    /// Getters

    /// get_name returns the name of this job instance
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_next(&self) -> DateTime<Local> {
        self.next
    }

    pub fn get_params(&self) -> &Vec<CString> {
        &self.params
    }

    pub fn get_schedule(&self) -> &Schedule {
        &self.schedule
    }

    /// Setters

    pub fn set_prev(&mut self, prev: DateTime<Local>) {
        self.prev = prev;
    }

    pub fn set_next(&mut self, next: DateTime<Local>) {
        self.next = next;
    }
}

impl std::fmt::Debug for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Job({} -> {})", self.name, self.next)
    }
}
