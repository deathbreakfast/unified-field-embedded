//! Declared data uses for this deployment, installed once at boot.
//!
//! `server/build.rs` scans everything the server links and writes the catalog;
//! [`install_data_use_catalog`] hands it to Valence so the `/valence` Data uses
//! pages can list every use this host makes. [`run`](crate::run) calls it before
//! building the router.
//!
//! # Examples
//!
//! ```rust,ignore
//! use server::data_use_catalog::{data_use_catalog, install_data_use_catalog};
//!
//! assert!(data_use_catalog().crate_names().contains("counter-app-worker"));
//! install_data_use_catalog()?;
//! assert!(valence::data_use::DataUseCatalog::global().is_some());
//! ```

mod generated {
    include!(concat!(env!("OUT_DIR"), "/data_uses.rs"));
}

pub use generated::data_use_catalog;

/// Install the build-time catalog as the process-wide Valence data-use catalog.
///
/// # Errors
///
/// Returns [`valence::data_use::CatalogInstallError::AlreadyInstalled`] when a
/// catalog is already installed in this process.
pub fn install_data_use_catalog() -> Result<(), valence::data_use::CatalogInstallError> {
    let catalog = data_use_catalog().install()?;
    log::info!(
        "[server] data-use catalog installed: {} uses from {} crates",
        catalog.len(),
        catalog.crate_names().len()
    );
    Ok(())
}
