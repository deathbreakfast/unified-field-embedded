//! Scans this host's dependency graph for declared Valence data uses and writes
//! `OUT_DIR/data_uses.rs`, which `server::data_use_catalog` installs at boot.
//!
//! Every crate the server links (counter worker, lepton, gauge, ...) is covered
//! automatically. Components that run as their own binary must be listed under
//! `[target.'cfg(any())'.dependencies]` in `server/Cargo.toml`; this host has none.

use std::path::PathBuf;

use valence_data_use_scan::{generate, Config, HostPackage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var_os("CARGO_FEATURE_SERVER_EMBEDDED").is_none() {
        return Ok(());
    }
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let workspace_root = manifest_dir
        .parent()
        .ok_or("server crate has no parent directory")?
        .to_path_buf();
    generate(&Config {
        workspace_root,
        out_dir: PathBuf::from(std::env::var("OUT_DIR")?),
        host: HostPackage::FromBuildScript,
        exclude_tests_from_snapshot: true,
        connection_edges: vec![],
    })?;
    Ok(())
}
