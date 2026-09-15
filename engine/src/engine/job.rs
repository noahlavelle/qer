use uuid::Uuid;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct JobID(String);

impl JobID {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn generate() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<JobID> for String {
    fn from(id: JobID) -> String {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_wraps_the_given_value() {
        let id = JobID::new("custom-id".to_owned());
        assert_eq!(id.as_str(), "custom-id");
    }

    #[test]
    fn new_generates_nonempty_id() {
        let id = JobID::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn new_generates_unique_ids() {
        let a = JobID::generate();
        let b = JobID::generate();
        assert_ne!(a.as_str(), b.as_str());
    }
}
