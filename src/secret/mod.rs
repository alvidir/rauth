pub mod framework;
pub mod application;
pub mod domain;

// lazy_static! {
//     pub static ref SECRET_REPOSITORY: Box<dyn domain::SecretRepository> = {
//         Box::new(framework::MongoSecretRepository{})
//     };
// }