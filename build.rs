use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &["Protobufs/steam/webuimessages_gamerecordingfiles.proto"],
        &["Protobufs/steam/"],
    )?;
    Ok(())
}
