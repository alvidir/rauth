use std::time::{SystemTime};

#[derive(Clone)]
pub struct Metadata {
    pub(super) id: i32,
    pub(super) created_at: SystemTime,
    pub(super) updated_at: SystemTime,
    pub(super) deleted_at: Option<SystemTime>,
}

impl Metadata {
    pub fn new() -> Self {
        Metadata {
            id: 0,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            deleted_at: None,
        }
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn touch(&mut self) {
        self.updated_at = SystemTime::now();
    }
}


#[cfg(test)]
pub mod tests {
    use std::time::SystemTime;
    use super::Metadata;

    pub fn new_metadata() -> Metadata {
        Metadata{
            id: 999,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            deleted_at: None,
        }
    }

    #[test]
    fn metadata_new_should_not_fail() {
        let before = SystemTime::now();
        let meta = Metadata::new();
        let after = SystemTime::now();

        assert_eq!(meta.id, 0);
        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }
}