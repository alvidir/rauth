use std::error::Error;

fn main()->Result<(),Box<dyn Error>>{
    // compiling protos using path on build time
    tonic_build::compile_protos("proto/user/session.proto")?;
    tonic_build::compile_protos("proto/app/registry.proto")?;

    Ok(())
}