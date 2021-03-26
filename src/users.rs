use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct User {
    issues: u32,
    estimate: f64,
    remaining: f64,
    actual: f64,
}

impl User {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn assignments(&self) -> u32 {
        self.issues
    }

    pub fn original_estimate_days(&self) -> f64 {
        self.estimate / 60.0 / 60.0 / 8.0
    }

    pub fn remaining_estimate_days(&self) -> f64 {
        self.remaining / 60.0 / 60.0 / 8.0
    }

    pub fn time_spent_days(&self) -> f64 {
        self.actual / 60.0 / 60.0 / 8.0
    }
}

pub struct Users(BTreeMap<String, User>);

impl Users {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn original_estimate_seconds(
        &mut self,
        assignee: String,
        estimate: Option<u64>,
    ) -> Option<u64> {
        if let Some(estimate) = estimate {
            let user = self.0.entry(assignee).or_insert(User::new());
            user.issues += 1;
            user.estimate += estimate as f64;
        }
        estimate
    }

    pub fn remaining_estimate_seconds(
        &mut self,
        assignee: String,
        remaining: Option<u64>,
    ) -> Option<u64> {
        if let Some(remaining) = remaining {
            let user = self.0.entry(assignee).or_insert(User::new());
            user.remaining += remaining as f64;
        }
        remaining
    }

    pub fn time_spent_seconds(&mut self, assignee: String, actual: Option<u64>) -> Option<u64> {
        if let Some(actual) = actual {
            let user = self.0.entry(assignee).or_insert(User::new());
            user.actual += actual as f64;
        }
        actual
    }
}

impl Iterator for Users {
    type Item = (String, User);

    fn next(&mut self) -> Option<(String, User)> {
        if let Some(k) = self.0.keys().next() {
            let k = k.clone();
            if let Some(v) = self.0.remove(&k) {
                return Some((k, v));
            }
        }
        None
    }
}

