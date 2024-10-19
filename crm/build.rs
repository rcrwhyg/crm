use anyhow::Result;
use std::fs;

fn main() -> Result<()> {
    fs::create_dir_all("src/pb")?;
    let config = tonic_build::configure();
    config
        .out_dir("src/pb")
        .compile_protos(&["../protos/crm.proto"], &["../protos"])?;

    Ok(())
}
