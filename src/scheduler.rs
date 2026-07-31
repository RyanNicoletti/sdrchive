use crate::config::{Config, Job, Schedule};
use jiff::{ToSpan, Zoned, civil::time};
use std::{cmp::Ordering, collections::BinaryHeap, time::Duration};

#[derive(Debug)]
pub struct ScheduledJob {
    pub job: Job,
    pub next_run: Zoned,
}
pub struct Scheduler {
    heap: BinaryHeap<ScheduledJob>,
}

impl Scheduler {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let mut job_heap = BinaryHeap::new();
        let now = Zoned::now();
        for job in config.jobs {
            let next = next_occurrence(&job.schedule, &now)?;
            job_heap.push(ScheduledJob {
                job,
                next_run: next,
            });
        }
        Ok(Self { heap: job_heap })
    }
    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let now = Zoned::now();
            let Some(due_at) = self.heap.peek().map(|e| e.next_run.clone()) else {
                break;
            };
            if due_at > now {
                let dur = Duration::try_from(due_at - now)?;
                std::thread::sleep(std::cmp::min(dur, Duration::from_secs(60)));
                continue;
            }

            let Some(mut next_job) = self.heap.pop() else {
                break;
            };
            next_job.next_run = next_occurrence(&next_job.job.schedule, &now)?;
            println!("{:?}", next_job);
            self.heap.push(next_job);
        }
        Ok(())
    }
}

impl PartialEq for ScheduledJob {
    fn eq(&self, other: &Self) -> bool {
        self.next_run == other.next_run
    }
}
impl Eq for ScheduledJob {}

impl PartialOrd for ScheduledJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledJob {
    fn cmp(&self, other: &Self) -> Ordering {
        self.next_run.cmp(&other.next_run).reverse()
    }
}

pub fn next_occurrence(schedule: &Schedule, now: &Zoned) -> anyhow::Result<Zoned> {
    match schedule {
        Schedule::Daily { start, .. } => {
            let next_today = now
                .with()
                .time(time(start.hour as i8, start.minute as i8, 0, 0))
                .build()?;
            if *now < next_today {
                Ok(next_today)
            } else {
                let next_tomorrow = next_today.checked_add(1.day())?;
                Ok(next_tomorrow)
            }
        }
        Schedule::Every {
            interval_minutes, ..
        } => {
            let t = now.time();
            let min_since_midnight = (t.hour() as u32 * 60) + t.minute() as u32;
            let next_interval = ((min_since_midnight / *interval_minutes) + 1) * *interval_minutes;
            if next_interval >= 1440 {
                let midnight = now.with().time(time(0, 0, 0, 0)).build()?;
                Ok(midnight.checked_add(1.day())?)
            } else {
                let next_hour = next_interval / 60;
                let next_min = next_interval % 60;
                Ok(now
                    .with()
                    .time(time(next_hour as i8, next_min as i8, 0, 0))
                    .build()?)
            }
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_daily_today() {
        todo!()
    }
    fn test_daily_tmrw() {
        todo!()
    }
    fn test_every_today() {
        todo!()
    }
    fn test_every_tmrw() {
        todo!()
    }
    fn test_daily_dst() {
        todo!()
    }
}
