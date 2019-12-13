#[derive(Debug, Clone)]
pub struct Package {
  pub name: String,
  pub version: String,
}

#[derive(Deserialize, Debug, Clone)]
struct Versions {
  versions: Vec<Version>,
}

impl Versions {
  pub fn latest(self) -> Option<Package> {
    self.versions.into_iter()
        .find(|version| !version.yanked)
        .map(|version| Package { name: version.name, version: version.num })
  }
}

#[derive(Deserialize, Clone, Debug)]
struct Version {
  num: String,
  #[serde(rename = "crate")]  name: String,
  yanked: bool,
}

pub async fn latest_version_for_package(package_name: String) -> Package {
  let url = format!("https://crates.io/api/v1/crates/{}/versions", package_name);

  let resp = reqwest::get(url.as_str())
      .await
      .map(|response| response.json::<Versions>());
  let unknown = Package { name: package_name.clone(), version: format!("0.0.0") };
  match resp {
    Ok(v) => match v.await {
      Ok(versions) => versions.latest().unwrap_or(unknown.clone()),
      Err(err) => {
        println!("Error while fetching versions for {} {:?}", package_name, err);
        unknown.clone()
      }
    },
    Err(err) => {
      println!("Error while fetching versions for {} {:?}", package_name, err);
      unknown.clone()
    },
  }
}