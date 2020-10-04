pub mod server;
mod model;

use structopt::StructOpt;

// These are the options used by the `server` subcommand
#[derive(Debug, StructOpt)]
pub struct ServerOptions {
   /// The address of the server that will run commands.
   #[structopt(long, default_value = "127.0.0.1:50051")]
   pub server_listen_addr: String,
}

// These are the options used by the `run` subcommand
#[derive(Debug, StructOpt)]
pub struct RemoteCommandOptions {
   /// The address of the server that will run commands.
   #[structopt(long = "server", default_value = "http://127.0.0.1:50051")]
   pub server_addr: String,
   /// The full command and arguments for the server to execute
   pub command: Vec<String>,
}

// These are the only valid values for our subcommands
#[derive(Debug, StructOpt)]
pub enum SubCommand {
   /// Start the remote command gRPC server
   #[structopt(name = "server")]
   StartServer(ServerOptions),
   /// Send a remote command to the gRPC server
   #[structopt(setting = structopt::clap::AppSettings::TrailingVarArg)]
   Run(RemoteCommandOptions),
}

// This is the main arguments structure that we'll parse from
#[derive(StructOpt, Debug)]
#[structopt(name = "remotecli")]
struct ApplicationArguments {
   #[structopt(flatten)]
   pub subcommand: SubCommand,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   let opts: ServerOptions;
   println!("Start the server on: {:?}", opts.server_listen_addr);
   server::session::server::start_server(opts).await?;

   Ok(())
}