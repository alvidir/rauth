// Import the generated rust code into module
pub mod user_proto {
   tonic::include_proto!("user");
}

// Proto session status enum
pub use user_proto::Status;

// Import the generated rust code into module
pub mod app_proto {
   tonic::include_proto!("app");
}

// Import the generated rust code into module
pub mod client_proto {
   tonic::include_proto!("client");
}

// Proto ticket type enum
pub use client_proto::TicketKind;