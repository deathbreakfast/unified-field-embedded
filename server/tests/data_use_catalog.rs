//! Contracts for the site's declared data-use catalog: which crates it covers,
//! that test fixtures and build paths stay out, the one-install boot rule, and
//! that every binary in this workspace has a deployment decision.

#![cfg(feature = "server-embedded")]

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use server::data_use_catalog::data_use_catalog;
use valence::data_use::{CatalogInstallError, DataUseCatalog, DataUseTarget};

/// Workspace binaries this deployment never runs, with the reason.
const NOT_DEPLOYED: &[(&str, &str)] = &[];

#[test]
fn catalog_includes_counter_worker_and_lepton() {
    let catalog = data_use_catalog();
    let crates = catalog.crate_names();
    assert!(
        crates.contains("counter-app-worker"),
        "counter worker is linked by the server; got {crates:?}"
    );
    assert!(
        crates.iter().any(|c| c.starts_with("lepton-")),
        "lepton auth crates are linked by the server; got {crates:?}"
    );
    assert!(
        catalog
            .entries()
            .iter()
            .any(|e| e.crate_name == "counter-app-worker"
                && e.target == DataUseTarget::Schema("counter".into())),
        "counter worker must declare at least one use on the counter schema"
    );
}

#[test]
fn catalog_has_no_e2e_fixtures() {
    let catalog = data_use_catalog();
    let leaked: Vec<&str> = catalog
        .crate_names()
        .into_iter()
        .filter(|c| c.ends_with("-e2e") || c.starts_with("data-use-probe-"))
        .collect();
    assert_eq!(
        leaked,
        Vec::<&str>::new(),
        "e2e or probe crates leaked into the catalog"
    );
    let fixture_purposes: Vec<&str> = catalog
        .entries()
        .iter()
        .map(|e| e.purpose.as_str())
        .filter(|p| p.contains("E2E_TEST_ONLY") || p.contains("data_use_catalog_fixtures"))
        .collect();
    assert_eq!(fixture_purposes, Vec::<&str>::new());
}

#[test]
fn catalog_paths_are_relative() {
    let catalog = data_use_catalog();
    let absolute: Vec<&str> = catalog
        .entries()
        .iter()
        .map(|e| e.file.as_str())
        .filter(|f| Path::new(f).is_absolute() || f.contains("/.cargo/") || f.contains("/home/"))
        .collect();
    assert_eq!(
        absolute,
        Vec::<&str>::new(),
        "catalog files must be repo-relative"
    );
}

#[test]
fn install_twice_fails_boot_contract() {
    let first = data_use_catalog()
        .install()
        .expect("first install in this process");
    assert!(!first.is_empty());
    let second = data_use_catalog().install();
    assert!(
        matches!(second, Err(CatalogInstallError::AlreadyInstalled { .. })),
        "second install must be rejected; got {second:?}"
    );
    assert!(DataUseCatalog::global().is_some());
}

#[test]
fn every_workspace_binary_is_accounted_for() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("server crate has a parent");
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let output = Command::new(cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(workspace.join("Cargo.toml"))
        .output()
        .expect("run cargo metadata");
    assert!(output.status.success(), "cargo metadata failed: {output:?}");
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata JSON");
    let packages = metadata["packages"].as_array().expect("packages array");

    let server = packages
        .iter()
        .find(|p| p["name"] == "server")
        .expect("server package in workspace");
    let mut linked = BTreeSet::new();
    let mut inventory = BTreeSet::new();
    for dep in server["dependencies"]
        .as_array()
        .expect("server dependencies")
    {
        let name = dep["name"].as_str().expect("dependency name");
        match (dep["kind"].as_str(), dep["target"].as_str()) {
            (None, Some("cfg(any())")) => {
                inventory.insert(name);
            }
            (None, _) => {
                linked.insert(name);
            }
            _ => {}
        }
    }
    let not_deployed: BTreeSet<&str> = NOT_DEPLOYED.iter().map(|(name, _)| *name).collect();

    let mut bin_packages = BTreeSet::new();
    let mut unaccounted = Vec::new();
    for package in packages {
        let name = package["name"].as_str().expect("package name");
        let has_bin = package["targets"]
            .as_array()
            .expect("targets")
            .iter()
            .any(|t| {
                t["kind"]
                    .as_array()
                    .is_some_and(|k| k.iter().any(|k| k == "bin"))
            });
        if !has_bin {
            continue;
        }
        bin_packages.insert(name);
        let accounted = name == "server"
            || linked.contains(name)
            || inventory.contains(name)
            || not_deployed.contains(name);
        if !accounted {
            unaccounted.push(name);
        }
    }
    assert_eq!(
        unaccounted,
        Vec::<&str>::new(),
        "each workspace binary must be the server, linked by it, declared under \
         [target.'cfg(any())'.dependencies] in server/Cargo.toml, or listed in NOT_DEPLOYED"
    );
    let stale: Vec<&str> = not_deployed.difference(&bin_packages).copied().collect();
    assert_eq!(
        stale,
        Vec::<&str>::new(),
        "NOT_DEPLOYED names a binary that no longer exists"
    );
}
