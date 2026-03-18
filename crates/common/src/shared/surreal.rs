use std::str::FromStr;

use fake::Dummy;
use fake::Fake;
use fake::Faker;
use fake::faker::lorem::en::Word;
use regex::Regex;
use serde::Deserialize;
use surrealdb::types::Datetime;
use surrealdb::types::RecordId;
use surrealdb::types::RecordIdKey;
use surrealdb::types::SurrealValue;
use surrealdb::types::ToSql;
use surrealdb::types::Uuid;
use uuid::Uuid as Uuid2;

use crate::errors::SerdeError;

#[allow(clippy::derived_hash_with_manual_eq)]
#[derive(Debug, Clone, Hash, Ord, PartialOrd, SurrealValue)]
pub struct SurrealId(pub RecordId);

impl SurrealId {
  pub fn new(table: String) -> Self {
    let key = Uuid::new_v7();
    Self(RecordId::new(table, key))
  }

  pub fn table(&self) -> String {
    self.0.table.to_string()
  }

  pub fn key(&self) -> Uuid {
    let key = self.inner().key.to_owned();

    let key = match key {
      RecordIdKey::Uuid(uuid) => uuid,
      _ => unreachable!("RecordIdKey is not a Uuid"),
    };

    Self::parse_uuid_string(key.to_string())
  }

  pub fn inner(&self) -> RecordId {
    self.0.clone()
  }

  fn parse_uuid_string(key: String) -> Uuid {
    match key.replace("u'", "").replace("⟨u'", "").replace("'⟩", "").replace("'", "").parse::<Uuid>() {
      Ok(uuid) => uuid,
      Err(e) => {
        tracing::error!("[SurrealId] failed to parse UUID string: {}: {}", key, e);
        Uuid::default()
      }
    }
  }

  pub fn is_default(&self) -> bool {
    self.table() == "default"
  }

  pub fn serialize_self<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    serializer.serialize_str(&self.0.to_sql())
  }

  pub fn looks_like(id: String) -> bool {
    // Regex: string + : + u' + valid uuid + '
    let regex =
      Regex::new(r"^[a-zA-Z0-9_-]+:[u]'[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}'$")
        .unwrap();

    regex.is_match(&id)
  }

  pub fn get_id_from_string(string: String) -> Option<SurrealId> {
    let regex =
      Regex::new(r"[a-zA-Z0-9_-]+:[u]'[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}'")
        .unwrap();

    let matches = regex.captures(&string);

    println!("Matches: {:?}", matches);

    if let Some(matches) = matches {
      SurrealId::try_from(matches.get(0).unwrap().as_str().to_string()).ok()
    } else {
      None
    }
  }

  pub fn get_uuid_from_string(string: String) -> Option<Uuid> {
    let regex = Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}").unwrap();
    let matches = regex.captures(&string);
    if let Some(matches) = matches { Uuid::from_str(matches.get(0).unwrap().as_str()).ok() } else { None }
  }
}

impl From<RecordId> for SurrealId {
  fn from(id: RecordId) -> Self {
    Self(id)
  }
}

impl From<SurrealId> for RecordId {
  fn from(id: SurrealId) -> Self {
    id.0
  }
}

impl From<(String, String)> for SurrealId {
  fn from((table, key): (String, String)) -> Self {
    Self(RecordId::new(table, key))
  }
}

impl From<(String, Uuid)> for SurrealId {
  fn from((table, key): (String, Uuid)) -> Self {
    Self(RecordId::new(table, key))
  }
}

impl From<(String, Uuid2)> for SurrealId {
  fn from((table, key): (String, Uuid2)) -> Self {
    let key = RecordIdKey::Uuid(key.into());
    Self(RecordId::new(table, key))
  }
}

impl From<SurrealId> for String {
  fn from(id: SurrealId) -> Self {
    id.0.to_sql()
  }
}

impl TryFrom<String> for SurrealId {
  type Error = anyhow::Error;

  fn try_from(id: String) -> Result<Self, Self::Error> {
    if !id.contains(':') {
      return Err(SerdeError::InvalidSurrealId(id).into());
    }

    let (table, key) = id.split_once(':').unwrap_or(("faulty", "faulty"));
    let key = Self::parse_uuid_string(key.to_string());
    Ok(Self(RecordId::new(table.to_string(), key)))
  }
}

impl TryFrom<&str> for SurrealId {
  type Error = anyhow::Error;

  fn try_from(id: &str) -> Result<Self, Self::Error> {
    if !id.contains(':') {
      return Err(SerdeError::InvalidSurrealId(id.to_string()).into());
    }

    let (table, key) = id.split_once(':').unwrap_or(("&faulty", "&faulty"));
    let key = Self::parse_uuid_string(key.to_string());
    Ok(Self(RecordId::new(table.to_string(), key)))
  }
}

impl PartialEq for SurrealId {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl Eq for SurrealId {}

impl Default for SurrealId {
  fn default() -> Self {
    let uuid = Uuid::new_v7_from_datetime(Datetime::from_timestamp(0, 0).unwrap());
    Self(RecordId::new("default", uuid))
  }
}

impl specta::Type for SurrealId {
  fn inline(_type_map: &mut specta::TypeMap, _generics: specta::Generics) -> specta::DataType {
    specta::DataType::Primitive(specta::datatype::PrimitiveType::String)
  }
}

struct SurrealIdVisitor;

impl serde::Serialize for SurrealId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where S: serde::Serializer {
    serializer.serialize_str(&self.0.to_sql())
  }
}

impl<'de> serde::de::Visitor<'de> for SurrealIdVisitor {
  type Value = SurrealId;

  fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "a string or RecordId")
  }

  fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
    SurrealId::try_from(v.to_owned()).map_err(E::custom)
  }

  fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
  where A: serde::de::MapAccess<'de> {
    let rid = RecordId::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
    Ok(SurrealId::from(rid))
  }
}

impl<'de> Deserialize<'de> for SurrealId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where D: serde::Deserializer<'de> {
    deserializer.deserialize_any(SurrealIdVisitor)
  }
}

impl std::fmt::Display for SurrealId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0.to_sql())
  }
}

pub trait DbId {
  fn id(self) -> SurrealId;
  fn inner(self) -> RecordId;
}

impl<T: DbId> From<T> for SurrealId {
  fn from(id: T) -> Self {
    id.id()
  }
}

impl Dummy<Faker> for SurrealId {
  fn dummy_with_rng<R: fake::Rng + ?Sized>(_config: &Faker, rng: &mut R) -> Self {
    let table: String = Word().fake_with_rng(rng);
    let key = Uuid::new_v7();

    Self(RecordId::new(table, key))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
  struct Test {
    id: SurrealId,
  }

  #[test]
  fn test_session_patch() {
    let personality_id_str = "personality:u'3f1d6a5a-9291-4a83-a840-93f9034b1ade'";
    let surreal_id = SurrealId::try_from(personality_id_str).unwrap_or_default();
    let test = Test { id: surreal_id.clone() };
    let test_json = serde_json::to_string(&test).unwrap_or_default();
    println!("Test JSON: {}", test_json);

    let test = match serde_json::from_str::<Test>(&test_json) {
      Ok(test) => test,
      Err(e) => panic!("Error: {}", e),
    };

    println!("Test: {:#?}", test);
    assert_eq!(test.id, surreal_id);
  }

  #[test]
  fn record_id_to_sql() {
    let record_id = RecordId::new("test", Uuid::new_v7());
    let sql = record_id.to_sql();
    println!("SQL: {}", sql);
  }

  #[test]
  fn test_surreal_id_looks_like() {
    let id = "personality:u'019bf534-cdda-7a63-9ccf-350ecd7e5024'";
    assert!(SurrealId::looks_like(id.to_string()));
  }

  #[test]
  fn test_surreal_id_get_id_from_string() {
    let malformed_id = "BAD TEXT personality:u'019bf534-cdda-7a63-9ccf-350ecd7e5024' BAD TEXT";
    let surreal_id = SurrealId::get_id_from_string(malformed_id.to_string());

    let id = "personality:u'019bf534-cdda-7a63-9ccf-350ecd7e5024'";
    let parsed_id = SurrealId::try_from(id.to_string()).ok();

    assert_eq!(surreal_id, parsed_id);
  }

  #[test]
  fn test_surreal_id_get_uuid_from_string() {
    let id = "019bf534-cdda-7a63-9ccf-350ecd7e5024";
    let expected_uuid = Uuid::from_str(id).unwrap();

    let uuid = SurrealId::get_uuid_from_string(id.to_string());

    assert_eq!(uuid, Some(expected_uuid));
  }

  #[test]
  fn test_surreal_id_get_uuid_from_string_malformed() {
    let id = "019bf534-cdda-7a63-9ccf-350ecd7e5024";
    let expected_uuid = Uuid::from_str(id).unwrap();

    let id = "BAD TEXT 019bf534-cdda-7a63-9ccf-350ecd7e5024 BAD TEXT";
    let uuid = SurrealId::get_uuid_from_string(id.to_string());

    assert_eq!(uuid, Some(expected_uuid));
  }
}
