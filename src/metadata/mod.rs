pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresMetadataRepository = {
        framework::PostgresMetadataRepository
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::MetadataRepository> {
    #[cfg(not(test))]
    return Box::new(&*REPO_PROVIDER);
    
    #[cfg(test)]
    return Box::new(&*tests::REPO_TEST);
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use std::time::SystemTime;
    use super::domain::{InnerMetadata, Metadata, MetadataRepository};

    pub struct Mock;
    lazy_static! {
        pub static ref REPO_TEST: Mock = Mock;
    } 
    
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

    pub fn new_metadata() -> Metadata {
        Metadata::new().unwrap()
    }

    #[test]
    fn domain_metadata_new_ok() {
        let before = SystemTime::now();
        let meta = Metadata::new().unwrap();
        let after = SystemTime::now();

        assert_eq!(meta.id, 999);
        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }

    #[test]
    fn domain_inner_metadata_ok() {        
        let before = SystemTime::now();
        let meta = InnerMetadata::new();
        let after = SystemTime::now();

        assert!(meta.created_at >= before && meta.created_at <= after);
        assert!(meta.updated_at >= before && meta.updated_at <= after);
    }
}