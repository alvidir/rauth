use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Metadata {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Metadata {
    pub fn new() -> Self {
        Metadata {
            id: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
pub mod tests {
    use super::Metadata;
    use chrono::Utc;

    pub fn new_metadata() -> Metadata {
        Metadata {
            id: 999,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    #[test]
    fn metadata_new_should_not_fail() {
        let before = Utc::now();
        let meta = Metadata::new();
        let after = Utc::now();

        assert_eq!(meta.id, 0);
        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }
}
