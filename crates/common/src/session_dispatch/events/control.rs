#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ControlEvent {
  TurnStart,
  TurnStop,
  CompactingStart,
  CompactingDone,
}
