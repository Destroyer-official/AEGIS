fn main() -> Result<(), Box<dyn std::error::Error>> {
    // If PROTOC is not set, try common Windows user profile fallback if present
    if std::env::var("PROTOC").is_err() {
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let candidate = std::path::PathBuf::from(user_profile).join("protoc\\bin\\protoc.exe");
            if candidate.exists() {
                std::env::set_var("PROTOC", candidate);
            }
        }
    }
    tonic_build::compile_protos("proto/gateway.proto")?;
    Ok(())
}
