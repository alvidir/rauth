// Import the generated rust code into module
pub mod client_proto {
   tonic::include_proto!("user");
}

// Proto session status enum
pub use client_proto::Status;