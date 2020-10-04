fn main()->Result<(),Box<dyn std::error::Error>>{
    // compiling protos using path on build time
    tonic_build::compile_protos("proto/session/login.proto")?;
    tonic_build::compile_protos("proto/session/logout.proto")?;
    tonic_build::compile_protos("proto/session/signup.proto")?;
    tonic_build::compile_protos("proto/session/session.proto")?;
    Ok(())
}