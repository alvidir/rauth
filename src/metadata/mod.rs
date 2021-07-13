pub mod framework;
pub mod application;
pub mod domain;

// lazy_static! {
//     pub static ref METADATA_REPOSITORY: Box<dyn domain::MetadataRepository> = {
//         Box::new(framework::PostgresMetadataRepository{})
//     };
// }