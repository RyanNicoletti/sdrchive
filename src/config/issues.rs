#[derive(Debug)]
pub struct Issue {
    pub at: String,
    pub problem: String,
}

#[derive(Debug, Default)]
pub struct Issues {
    pub items: Vec<Issue>,
}

impl Issues {
    pub fn check(&mut self, ok: bool, at: impl Into<String>, problem: impl Into<String>) {
        if !ok {
            self.items.push(Issue {
                at: at.into(),
                problem: problem.into(),
            });
        }
    }

    pub fn into_result(self) -> Result<(), Issues> {
        if self.items.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl std::fmt::Display for Issues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} problem(s) in config:", self.items.len())?;
        for issue in &self.items {
            write!(f, "\n  {}: {}", issue.at, issue.problem)?;
        }
        Ok(())
    }
}

impl std::error::Error for Issues {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issues_empty_is_ok() {
        assert!(Issues::default().into_result().is_ok());
    }

    #[test]
    fn test_issues_collects_all_failures() {
        let mut issues = Issues::default();
        issues.check(false, "jobs[0].name", "must not be empty");
        issues.check(
            false,
            "jobs[1].schedule.duration_minutes",
            "must be less than 1440",
        );

        let msg = issues.into_result().unwrap_err().to_string();
        assert!(msg.contains("jobs[0].name"));
        assert!(msg.contains("jobs[1].schedule.duration_minutes"));
    }
}
