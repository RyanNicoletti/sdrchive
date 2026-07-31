use jiff::{ToSpan, Zoned, civil::time};

use crate::config::Schedule;

fn next_run(schedule: &Schedule, now: &Zoned) -> anyhow::Result<Zoned> {
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
