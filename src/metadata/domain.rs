use chrono::naive::NaiveDateTime;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

impl Metadata {
    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now().naive_utc();
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            id: 0,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            deleted_at: None,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::Metadata;
    use chrono::Utc;

    pub fn new_metadata() -> Metadata {
        Metadata {
            id: 999,
            ..Default::default()
        }
    }

    #[test]
    fn metadata_new_should_not_fail() {
        let before = Utc::now().naive_utc();
        let meta = Metadata::default();
        let after = Utc::now().naive_utc();

        assert_eq!(meta.id, 0);
        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }
}
