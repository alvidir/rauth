pub mod framework;
pub mod application;
pub mod domain;

#[cfg(test)]
mod tests {
    use std::error::Error;
    use crate::metadata::domain::Metadata;
    use crate::secret::domain::Secret;
    use super::domain::{App, AppRepository};

    struct Mock {}
    
    impl AppRepository for &Mock {
        fn find(&self, url: &str) -> Result<App, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn save(&self, app: &mut App) -> Result<(), Box<dyn Error>> {
            app.id = 999;
            Ok(())
        }

        fn delete(&self, app: &App) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }

    #[test]
    fn app_new_ok() {
        const URL: &str = "http://testing.com";
        let mock_impl = &Mock{};

        let meta = Metadata::now();
        let secret = Secret {
            id: "testing".to_string(),
            data: "this is a secret".as_bytes().to_vec(),
            meta: meta,
        };

        let app = App::new(Box::new(mock_impl),
                           secret,
                           Metadata::now(),
                           URL).unwrap();

        assert_eq!(app.id, 999); 
        assert_eq!(app.url, URL);
    }

    #[test]
    fn user_new_ko() {
        const URL: &str = "not_an_url";
        let mock_impl = &Mock{};

        let meta = Metadata::now();
        let secret = Secret {
            id: "testing".to_string(),
            data: "this is a secret".as_bytes().to_vec(),
            meta: meta,
        };
        
        let app = App::new(Box::new(mock_impl),
                           secret,
                           Metadata::now(),
                           URL);
    
        assert!(app.is_err());
    }
}