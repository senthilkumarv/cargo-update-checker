use core::fmt;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer};
use serde::de::{MapAccess, Visitor};

#[derive(Debug, Serialize)]
pub struct PackageAttributes {
  pub version: String
}

impl FromStr for PackageAttributes {
  type Err = ();

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(PackageAttributes { version: String::from(s) })
  }
}

impl<'de> Deserialize<'de> for PackageAttributes {
  fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
      D: Deserializer<'de> {
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    #[derive(Deserialize)]
    struct PackageAttributeMirror {
      version: Option<String>
    }

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
      where
          T: Deserialize<'de> + FromStr<Err=()>,
    {
      type Value = T;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string or map")
      }

      fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
      {
        Ok(FromStr::from_str(value).unwrap())
      }

      fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
      {
        let attr: PackageAttributeMirror = Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;
        Ok(FromStr::from_str(attr.version.unwrap_or_else(|| format!("0.0.0")).as_str()).unwrap())
      }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
  }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CargoDefinition {
  pub dependencies: HashMap<String, PackageAttributes>
}

pub fn extract_packages(toml_content: String) -> CargoDefinition {
  let package_info: CargoDefinition = toml::from_str(toml_content.as_str()).unwrap();
  package_info
}