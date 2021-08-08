pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresMetadataRepository = {
        framework::PostgresMetadataRepository
    }; 
}   

#[cfg(not(test))]
pub fn get_repository() -> Box<&'static dyn domain::MetadataRepository> {
    Box::new(&*REPO_PROVIDER)
}

#[cfg(test)]
pub fn get_repository() -> Box<dyn domain::MetadataRepository> {
    Box::new(tests::Mock)
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::time::SystemTime;
    use super::domain::{InnerMetadata, Metadata, MetadataRepository};

    pub struct Mock;    
    impl MetadataRepository for Mock {
        fn find(&self, _id: i32) -> Result<Metadata, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn create(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
            meta.id = 999;
            Ok(())
        }

        fn save(&self, _meta: &Metadata) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }  

        fn delete(&self, _meta: &Metadata) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }  
    }

    pub fn new_metadata() -> Metadata {
        Metadata::new().unwrap()
    }

    #[test]
    fn metadata_new_ok() {
        let before = SystemTime::now();
        let meta = Metadata::new().unwrap();
        let after = SystemTime::now();

        assert_eq!(meta.id, 999);
        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }

    #[test]
    fn inner_metadata_ok() {        
        let before = SystemTime::now();
        let meta = InnerMetadata::new();
        let after = SystemTime::now();

        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }
}