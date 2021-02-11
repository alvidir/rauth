use std::error::Error;

fn main()->Result<(),Box<dyn Error>>{
    // compiling protos using path on build time
    tonic_build::compile_protos("proto/user/login.proto")?;
    tonic_build::compile_protos("proto/user/logout.proto")?;
    tonic_build::compile_protos("proto/user/signup.proto")?;
    tonic_build::compile_protos("proto/user/session.proto")?;

    tonic_build::compile_protos("proto/dashboard/app_manager.proto")?;
    tonic_build::compile_protos("proto/dashboard/dashboard.proto")?;

    tonic_build::compile_protos("proto/app/close.proto")?;
    tonic_build::compile_protos("proto/app/open.proto")?;
    tonic_build::compile_protos("proto/app/gateway.proto")?;

    Ok(())
}