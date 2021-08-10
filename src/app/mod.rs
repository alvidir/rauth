pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::PostgresAppRepository = {
        framework::PostgresAppRepository
    }; 
}   

#[cfg(not(test))]
pub fn get_repository() -> Box<&'static dyn domain::AppRepository> {
    Box::new(&*REPO_PROVIDER)
}

#[cfg(test)]
pub fn get_repository() -> Box<dyn domain::AppRepository> {
    Box::new(tests::Mock)
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use crate::metadata::tests::new_metadata;
    use crate::secret::tests::new_secret;
    use super::domain::{App, AppRepository};

    pub struct Mock;
    impl AppRepository for Mock {
        fn find(&self, _url: &str) -> Result<App, Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn create(&self, app: &mut App) -> Result<(), Box<dyn Error>> {
            app.id = 999;
            Ok(())
        }

        fn save(&self, _app: &App) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }

        fn delete(&self, _app: &App) -> Result<(), Box<dyn Error>> {
            Err("unimplemeted".into())
        }
    }

    pub fn new_app() -> App {
        App{
            id: 999,
            url: "http://testing.com".to_string(),
            secret: new_secret(),
            meta: new_metadata(),
        }
    }

    #[test]
    fn app_new_ok() {
        const URL: &str = "http://testing.com";
        let secret = new_secret();

        let meta = new_metadata();
        let app = App::new(secret,
                           meta,
                           URL).unwrap();

        assert_eq!(app.id, 999); 
        assert_eq!(app.url, URL);
    }

    #[test]
    fn app_new_ko() {
        const URL: &str = "not_an_url";
        let secret = new_secret();
        
        let meta = new_metadata();
        let app = App::new(secret,
                           meta,
                           URL);
    
        assert!(app.is_err());
    }
}