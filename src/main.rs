#[macro_use]
extern crate serde_derive;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::pin::Pin;

use futures::{Future, FutureExt};

use crate::cargo_definition::extract_packages;
use crate::upstream_packages::Package;

mod cargo_definition;
mod upstream_packages;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let arguments = env::args();
  if arguments.len() != 2 {
    println!("Usage: cargo_update_checker <proj-dir>");
    return Ok(());
  }
  let path_from_arg = arguments.last().unwrap();
  let path_str = if path_from_arg.ends_with(".toml") {
    path_from_arg
  } else {
    format!("{}/Cargo.toml", path_from_arg)
  };
  let path = Path::new(&path_str);
  let mut file = File::open(path).unwrap();
  let mut content = String::new();
  let _ = file.read_to_string(&mut content);
  let packages = extract_packages(content);
  let upstream_package_results: Vec<Pin<Box<dyn Future<Output=Package>>>> = packages.dependencies.keys().map(|package_name| {
    upstream_packages::latest_version_for_package(package_name.clone()).boxed_local()
  }).collect();
  let b: Vec<Package> = futures::future::join_all(upstream_package_results).await;
  packages.dependencies.iter().for_each(|(key, attr)| {
    match b.clone().iter().find(|package| package.name == key.clone()) {
      Some(package) => match (version_compare::Version::from(package.version.as_str()), version_compare::Version::from(attr.version.as_str())) {
        (Some(upstream), Some(local)) => {
          if upstream > local {
            println!("Package {} has a newer version {:?}, local version is {}", package.name, package.version, attr.version)
          }
        }
        (None, _) | (_, None) => {
          println!("Failed parsing version for {} with error", package.name)
        }
      },
      None => println!("Package {} not found", key)
    }
  });
  Ok(())
}