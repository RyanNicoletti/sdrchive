use crate::{
    config::{Location, Schedule},
    resolve::{ResolvedConfig, ResolvedJob},
    runner,
    sdr::SdrDevice,
};
use jiff::{ToSpan, Zoned, civil::time};
use std::{cmp::Ordering, collections::BinaryHeap, path::PathBuf, time::Duration};

#[derive(Debug)]
pub struct ScheduledJob {
    pub job: ResolvedJob,
    pub next_run: Zoned,
}

pub struct Scheduler {
    heap: BinaryHeap<ScheduledJob>,
    pub output_dir: PathBuf,
    location: Option<Location>,
}

impl Scheduler {
    pub fn new(config: ResolvedConfig) -> anyhow::Result<Self> {
        let mut job_heap = BinaryHeap::new();
        let now = Zoned::now();
        for job in config.resolved_jobs {
            let next = next_occurrence(&job.schedule, &now)?;
            job_heap.push(ScheduledJob {
                job,
                next_run: next,
            });
        }
        Ok(Self {
            heap: job_heap,
            output_dir: config.output_dir,
            location: config.location,
        })
    }
    pub fn run(&mut self, sdr: &mut dyn SdrDevice) -> anyhow::Result<()> {
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
            let dur_seconds = next_job.job.schedule.duration_minutes() * 60;
            runner::run_job(&next_job.job, dur_seconds, sdr)?;
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
    use crate::config::StartTime;

    use super::*;

    #[test]
    fn test_daily_today() {
        let test_sched = Schedule::Daily {
            start: StartTime {
                minute: 00,
                hour: 06,
            },
            duration_minutes: 60,
        };
        let now: Zoned = "2026-07-30T05:00:00[America/Chicago]".parse().unwrap();
        let next = next_occurrence(&test_sched, &now).unwrap();
        let expected_next: Zoned = "2026-07-30T06:00:00[America/Chicago]".parse().unwrap();
        assert_eq!(next, expected_next);
    }

    #[test]
    fn test_daily_tmrw() {
        let test_sched = Schedule::Daily {
            start: StartTime {
                minute: 00,
                hour: 06,
            },
            duration_minutes: 60,
        };
        let now: Zoned = "2026-07-30T14:20:00[America/Chicago]".parse().unwrap();
        let next = next_occurrence(&test_sched, &now).unwrap();
        let expected_next: Zoned = "2026-07-31T06:00:00[America/Chicago]".parse().unwrap();
        assert_eq!(next, expected_next);
    }

    #[test]
    fn test_every_today() {
        let test_sched = Schedule::Every {
            interval_minutes: 240,
            duration_minutes: 60,
        };
        let now: Zoned = "2026-07-30T10:37:00[America/Chicago]".parse().unwrap();
        let next = next_occurrence(&test_sched, &now).unwrap();
        let expected_next: Zoned = "2026-07-30T12:00:00[America/Chicago]".parse().unwrap();
        assert_eq!(next, expected_next);
    }

    #[test]
    fn test_every_tmrw() {
        let test_sched = Schedule::Every {
            interval_minutes: 240,
            duration_minutes: 60,
        };
        let now: Zoned = "2026-07-30T22:15:00[America/Chicago]".parse().unwrap();
        let next = next_occurrence(&test_sched, &now).unwrap();
        let expected_next: Zoned = "2026-07-31T00:00:00[America/Chicago]".parse().unwrap();
        assert_eq!(next, expected_next);
    }

    #[test]
    fn test_daily_dst() {
        let test_sched = Schedule::Daily {
            start: StartTime {
                minute: 30,
                hour: 02,
            },
            duration_minutes: 60,
        };
        let now: Zoned = "2026-03-08T01:00:00[America/Chicago]".parse().unwrap();
        let next = next_occurrence(&test_sched, &now).unwrap();
        // 02:30 doesn't exist on this night due to daylight savings (clocks jump 02:00 -> 03:00)
        // jiff accounts for this automatically
        let expected_next: Zoned = "2026-03-08T03:30:00[America/Chicago]".parse().unwrap();
        assert_eq!(next, expected_next);
    }
}
