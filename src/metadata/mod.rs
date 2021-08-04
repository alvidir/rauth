pub mod framework;
pub mod application;
pub mod domain;

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::time::SystemTime;
    use super::domain::{Metadata, MetadataRepository};

    struct Mock;
    
    impl MetadataRepository for Mock {
        fn find(&self, _id: i32) -> Result<Metadata, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
            meta.id = 999;
            Ok(())
        }

        fn delete(&self, _meta: &Metadata) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }  
    }

    #[test]
    fn domain_metadata_new_ok() {
        let mock_impl = Mock{};

        let before = SystemTime::now();
        let meta = Metadata::new(&mock_impl).unwrap();
        let after = SystemTime::now();

        assert_eq!(meta.id, 999);
        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }

    #[test]
    fn domain_metadata_now_ok() {        
        let before = SystemTime::now();
        let meta = Metadata::new(&Mock).unwrap();
        let after = SystemTime::now();

        assert_eq!(meta.id, 0);
        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }
}