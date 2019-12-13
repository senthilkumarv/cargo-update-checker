extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::pin::Pin;

use futures::{Future, FutureExt};

use crate::cargo_definition::extract_packages;
use crate::upsteam_packages::Package;

mod cargo_definition;
mod upsteam_packages;

#[tokio::main]
async fn main() {
  let arguments = env::args();
  if arguments.len() != 2 {
    println!("Usage: cargo_update_checker <proj-dir>");
    return;
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
    upsteam_packages::latest_version_for_package(package_name.clone()).boxed_local()
  }).collect();
  let b: Vec<Package> = futures::future::join_all(upstream_package_results).await;
  packages.dependencies.iter().for_each(|(key, attr)| {
    match b.clone().iter().find(|package| package.name == key.clone()) {
      Some(package) => match (semver::Version::parse(package.version.as_str()), semver::Version::parse(attr.version.as_str())) {
        (Ok(upstream), Ok(local)) => {
          if upstream.gt(&local) {
            println!("Package {} has a newer version {:?}, local version is {}", package.name, package.version, attr.version)
          }
        }
        (Err(err), _) | (_, Err(err)) => {
          println!("Failed passing version for {} with error {:?}", package.name, err)
        }
      },
      None => println!("Package {} not found", key)
    }
  });
}