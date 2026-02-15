use crate::model::RecordedEvent;

pub fn save_events_to_file(path: &str, events: &[RecordedEvent]) -> Result<String, String> {
    let yaml = serde_yaml::to_string(events).map_err(|e| e.to_string())?;
    std::fs::write(path, yaml).map_err(|e| e.to_string())?;
    Ok(format!("Saved {} events to {path}", events.len()))
}

pub fn load_events_from_file(path: &str) -> Result<Vec<RecordedEvent>, String> {
    let yaml = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_yaml::from_str(&yaml).map_err(|e| e.to_string())
}
