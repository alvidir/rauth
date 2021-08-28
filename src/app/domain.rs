use std::error::Error;

use crate::regex;
use crate::secret::domain::Secret;
use crate::metadata::domain::Metadata;

pub trait AppRepository {
    fn find(&self, id: i32) -> Result<App, Box<dyn Error>>;
    fn find_by_url(&self, url: &str) -> Result<App, Box<dyn Error>>;
    fn create(&self, app: &mut App) -> Result<(), Box<dyn Error>>;
    fn save(&self, app: &App) -> Result<(), Box<dyn Error>>;
    fn delete(&self, app: &App) -> Result<(), Box<dyn Error>>;
}

pub struct App {
    pub(super) id: i32,
    pub(super) url: String,
    pub(super) secret: Secret,
    pub(super) meta: Metadata,
}

impl App {
    pub fn new(secret: Secret,
               meta: Metadata,
               url: &str) -> Result<Self, Box<dyn Error>> {
        
        regex::match_regex(regex::URL, url)?;

        let app = App {
            id: 0,
            url: url.to_string(),
            secret: secret,
            meta: meta,
        };
        
        Ok(app)
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_url(&self) -> &str {
        &self.url
    }
}


#[cfg(test)]
pub mod tests {
    use crate::metadata::domain::tests::new_metadata;
    use crate::secret::domain::tests::new_secret;
    use super::App;

    pub fn new_app() -> App {
        App{
            id: 999,
            url: "http://testing.com".to_string(),
            secret: new_secret(),
            meta: new_metadata(),
        }
    }

    pub fn new_app_custom(id: i32, url: &str) -> App {
        App{
            id: id,
            url: url.to_string(),
            secret: new_secret(),
            meta: new_metadata(),
        }
    }

    #[test]
    fn app_new_should_not_fail() {
        const URL: &str = "http://testing.com";
        let secret = new_secret();

        let meta = new_metadata();
        let app = App::new(secret,
                           meta,
                           URL).unwrap();

        assert_eq!(app.id, 0); 
        assert_eq!(app.url, URL);
    }

    #[test]
    fn app_new_with_wrong_url_should_fail() {
        const URL: &str = "not_an_url";
        let secret = new_secret();
        
        let meta = new_metadata();
        let app = App::new(secret,
                           meta,
                           URL);
    
        assert!(app.is_err());
    }
}