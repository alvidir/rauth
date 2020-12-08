use std::error::Error;

fn main()->Result<(),Box<dyn Error>>{
    // compiling protos using path on build time
    tonic_build::compile_protos("proto/client/login.proto")?;
    tonic_build::compile_protos("proto/client/logout.proto")?;
    tonic_build::compile_protos("proto/client/signup.proto")?;
    tonic_build::compile_protos("proto/client/session.proto")?;
    Ok(())
}