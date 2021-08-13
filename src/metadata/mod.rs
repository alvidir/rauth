pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresMetadataRepository = {
        framework::PostgresMetadataRepository
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::MetadataRepository> {
    Box::new(&*REPO_PROVIDER)
}

#[cfg(test)]
pub mod tests {
    use std::time::SystemTime;
    use super::domain::{InnerMetadata, Metadata};

    pub fn new_metadata() -> Metadata {
        Metadata{
            id: 999,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }

    #[test]
    fn metadata_new() {
        let before = SystemTime::now();
        let meta = Metadata::new();
        let after = SystemTime::now();

        assert_eq!(meta.id, 0);
        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }

    #[test]
    fn inner_metadata_new() {        
        let before = SystemTime::now();
        let meta = InnerMetadata::new();
        let after = SystemTime::now();

        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.touch_at >= before && meta.touch_at <= after);
    }
}