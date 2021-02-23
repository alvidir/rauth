// Import the generated rust code into module
pub mod user_proto {
   tonic::include_proto!("user");
}

// Import the generated rust code into module
pub mod app_proto {
   tonic::include_proto!("app");
}


// Proto session status enum
pub use user_proto::Status;