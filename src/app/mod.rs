pub mod framework;
pub mod application;
pub mod domain;

#[cfg(test)]
mod tests {
    use std::error::Error;
    use crate::metadata::domain::{InnerMetadata, Metadata, MetadataRepository};
    use crate::secret::tests as SecretTests;
    use super::domain::{App, AppRepository};

    struct Mock {}
    
    impl AppRepository for Mock {
        fn find(&self, _url: &str) -> Result<App, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, app: &mut App) -> Result<(), Box<dyn Error>> {
            app.id = 999;
            Ok(())
        }

        fn delete(&self, _app: &App) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
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

    #[test]
    fn domain_app_new_ok() {
        const URL: &str = "http://testing.com";
        let mock_impl = Mock{};

        let inner_meta = InnerMetadata::new();
        let secret = SecretTests::new_secret();

        let meta = Metadata::new(&mock_impl).unwrap();
        let app = App::new(&mock_impl,
                           secret,
                           meta,
                           URL).unwrap();

        assert_eq!(app.id, 999); 
        assert_eq!(app.url, URL);
    }

    #[test]
    fn domain_user_new_ko() {
        const URL: &str = "not_an_url";
        let mock_impl = Mock{};

        let inner_meta = InnerMetadata::new();
        let secret = SecretTests::new_secret();
        
        let meta = Metadata::new(&mock_impl).unwrap();
        let app = App::new(&mock_impl,
                           secret,
                           meta,
                           URL);
    
        assert!(app.is_err());
    }
}