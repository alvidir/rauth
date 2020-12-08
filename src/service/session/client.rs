//pub mod session_proto {
//    tonic::include_proto!("session");
// }
// 
//// Proto generated client
//use session_proto::session_client::SessionClient;
//
//// Proto message structs
//use session_proto::{LoginRequest};
//
//pub async fn client_run(addr: String) -> Result<(), Box<dyn std::error::Error>> {
//    // Connect to server
//    // Use server addr if given, otherwise use default
//    let mut client = SessionClient::connect(addr).await?;
// 
//    let request = tonic::Request::new(LoginRequest {
//        cookie: "".to_string(),
//        username: "".to_string(),
//        email: "".to_string(),  
//        password: "".to_string(),
//    });
// 
//    let response = client.login(request).await?;
// 
//    println!("RESPONSE={:?}", response);
// 
//    Ok(())
// }