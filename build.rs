use std::error::Error;

fn main()->Result<(),Box<dyn Error>>{
    // compiling protos using path on build time
    tonic_build::compile_protos("proto/user/user.proto")?;
    tonic_build::compile_protos("proto/app/app.proto")?;
    tonic_build::compile_protos("proto/client/client.proto")?;

    Ok(())
}