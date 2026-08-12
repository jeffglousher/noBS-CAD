//! Session directories under `NBCAD_SESSION_DIR` (or temp `nbcad-sessions`).
//!
//! Supports:
//! - **read-only snapshot** attach (MCP loads model/focus; never claims writer)
//! - **live** attach (MCP claims `writer.json`, writebacks `model.json` after mutating tools)
//!
//! Layout: `<session_dir>/<uuid>/{model.json,focus.json,heartbeat.json,writer.json}`.
//! Session ids must be UUID v4 strings.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

/// Heartbeats older than this are marked `stale` in list metadata (no auto-delete).
pub const HEARTBEAT_STALE_MS: u64 = 30_000;

pub fn session_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("NBCAD_SESSION_DIR") {
        if !custom.trim().is_empty() {
            return PathBuf::from(custom);
        }
    }
    std::env::temp_dir().join("nbcad-sessions")
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

/// UUID v4 string form (8-4-4-4-12 hex with version nibble `4` and RFC variant).
pub fn is_valid_session_id(session_id: &str) -> bool {
    let bytes = session_id.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *byte != b'-' {
                    return false;
                }
            }
            14 => {
                if *byte != b'4' {
                    return false;
                }
            }
            19 => {
                let lower = byte.to_ascii_lowercase();
                if !matches!(lower, b'8' | b'9' | b'a' | b'b') {
                    return false;
                }
            }
            _ => {
                if !byte.is_ascii_hexdigit() {
                    return false;
                }
            }
        }
    }
    true
}

pub fn require_valid_session_id(session_id: &str) -> Result<(), String> {
    if is_valid_session_id(session_id) {
        Ok(())
    } else {
        Err(format!(
            "session_id must be a UUID v4 string (got '{session_id}')"
        ))
    }
}

/// List attachable session directories. Skips control dirs (`_*`) and non-UUID names.
pub fn list_sessions() -> Result<Vec<String>, String> {
    let root = session_dir();
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut sessions = Vec::new();
    for entry in fs::read_dir(&root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('_') || !is_valid_session_id(&name) {
                continue;
            }
            sessions.push(name);
        }
    }
    sessions.sort();
    Ok(sessions)
}

pub fn read_session_file(session_id: &str, filename: &str) -> Result<String, String> {
    let path = session_path(session_id, filename)?;
    fs::read_to_string(&path).map_err(|error| format!("could not read {}: {error}", path.display()))
}

/// Require `model.json` for the session. Missing file → hard error (Jack §3).
pub fn require_model_json(session_id: &str) -> Result<String, String> {
    require_valid_session_id(session_id)?;
    read_session_file(session_id, "model.json").map_err(|error| {
        format!("session '{session_id}' has no valid model.json ({error}); attach refused")
    })
}

/// Write a session file via temp + rename so readers never see a partial file.
pub fn write_session(session_id: &str, filename: &str, content: &str) -> Result<(), String> {
    let path = session_path(session_id, filename)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension(format!(
        "{}.tmp.{}",
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("json"),
        std::process::id()
    ));
    {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)
            .map_err(|error| format!("could not create temp {}: {error}", temporary.display()))?;
        file.write_all(content.as_bytes())
            .map_err(|error| format!("could not write temp {}: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("could not flush temp {}: {error}", temporary.display()))?;
    }
    fs::rename(&temporary, &path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("could not replace {}: {error}", path.display())
    })
}

/// Read `writer.json`, or a default `{ writer: "none", ... }` when missing/invalid.
pub fn read_writer(session_id: &str) -> Value {
    match read_session_file(session_id, "writer.json") {
        Ok(body) => {
            let parsed: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
            let writer = parsed
                .get("writer")
                .and_then(Value::as_str)
                .unwrap_or("none");
            let writer = match writer {
                "ui" | "mcp" | "none" => writer,
                _ => "none",
            };
            json!({
                "writer": writer,
                "updated_ms": parsed.get("updated_ms").and_then(Value::as_u64).unwrap_or(0),
                "generation": parsed.get("generation").and_then(Value::as_u64).unwrap_or(0),
            })
        }
        Err(_) => json!({
            "writer": "none",
            "updated_ms": 0,
            "generation": 0,
        }),
    }
}

/// Atomically claim the session writer lock (`writer.json`).
pub fn claim_writer(session_id: &str, writer: &str, generation: u64) -> Result<(), String> {
    require_valid_session_id(session_id)?;
    if !matches!(writer, "ui" | "mcp" | "none") {
        return Err(format!(
            "writer must be 'ui', 'mcp', or 'none' (got '{writer}')"
        ));
    }
    let body = json!({
        "writer": writer,
        "updated_ms": now_ms(),
        "generation": generation,
    });
    write_session(
        session_id,
        "writer.json",
        &serde_json::to_string(&body).map_err(|error| error.to_string())?,
    )
}

/// Claim the writer lock unless another party already holds it.
pub fn try_claim_writer(session_id: &str, writer: &str, generation: u64) -> Result<(), String> {
    let existing = read_writer(session_id)
        .get("writer")
        .and_then(Value::as_str)
        .unwrap_or("none")
        .to_string();
    if existing != "none" && existing != writer {
        return Err(format!(
            "session writer conflict: {existing} holds the writer lock; call cad_refresh or wait"
        ));
    }
    claim_writer(session_id, writer, generation)
}

/// Release the writer lock (sets `writer` to `"none"`).
pub fn release_writer(session_id: &str) -> Result<(), String> {
    let current = read_writer(session_id);
    let generation = current
        .get("generation")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    claim_writer(session_id, "none", generation)
}

/// Write `model.json` and bump heartbeat for a live session revision.
pub fn write_model_revision(
    session_id: &str,
    model_json: &str,
    generation: u64,
    source: &str,
) -> Result<(), String> {
    require_valid_session_id(session_id)?;
    write_session(session_id, "model.json", model_json)?;
    let heartbeat = json!({
        "updated_ms": now_ms(),
        "generation": generation,
        "source": source,
        "session_mode": "live",
        "session_id": session_id,
    });
    write_session(
        session_id,
        "heartbeat.json",
        &serde_json::to_string(&heartbeat).map_err(|error| error.to_string())?,
    )
}

/// Heartbeat age / staleness for a session directory (no auto-delete).
pub fn heartbeat_meta(session_id: &str) -> Value {
    match read_session_file(session_id, "heartbeat.json") {
        Ok(body) => {
            let parsed: Value = serde_json::from_str(&body).unwrap_or(json!({}));
            let updated_ms = parsed
                .get("updated_ms")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let age_ms = now_ms().saturating_sub(updated_ms);
            json!({
                "updated_ms": updated_ms,
                "age_ms": age_ms,
                "stale": age_ms > HEARTBEAT_STALE_MS,
                "generation": parsed.get("generation").cloned().unwrap_or(Value::Null),
            })
        }
        Err(_) => json!({
            "updated_ms": null,
            "age_ms": null,
            "stale": true,
            "generation": null,
        }),
    }
}

pub fn sessions_list_json() -> Value {
    match list_sessions() {
        Ok(sessions) => {
            let detailed: Vec<Value> = sessions
                .iter()
                .map(|session_id| {
                    let has_model = session_path(session_id, "model.json")
                        .map(|path| path.is_file())
                        .unwrap_or(false);
                    json!({
                        "session_id": session_id,
                        "has_model": has_model,
                        "heartbeat": heartbeat_meta(session_id),
                        "writer": read_writer(session_id),
                    })
                })
                .collect();
            json!({
                "session_mode": "snapshot_or_live",
                "supports_live": true,
                "sessions": sessions,
                "session_details": detailed,
                "session_dir": session_dir().display().to_string(),
                "heartbeat_stale_ms": HEARTBEAT_STALE_MS,
            })
        }
        Err(error) => json!({
            "session_mode": "snapshot_or_live",
            "supports_live": true,
            "sessions": [],
            "session_details": [],
            "session_dir": session_dir().display().to_string(),
            "heartbeat_stale_ms": HEARTBEAT_STALE_MS,
            "error": error,
        }),
    }
}

fn session_path(session_id: &str, filename: &str) -> Result<PathBuf, String> {
    require_valid_session_id(session_id)?;
    if filename.is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
    {
        return Err("invalid filename".to_string());
    }
    Ok(session_dir().join(session_id).join(filename))
}

/// Deterministic-looking UUID v4 for tests (unique via `now_ms` nibble).
#[cfg(test)]
pub fn test_session_uuid() -> String {
    format!("00000000-0000-4000-8000-{:012x}", now_ms() & 0xffffffffffff)
}

/// Serialize tests that mutate `NBCAD_SESSION_DIR`.
#[cfg(test)]
pub static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Acquire [`ENV_LOCK`], recovering if a prior test panicked while holding it.
#[cfg(test)]
pub fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_v4_validation_accepts_and_rejects() {
        assert!(is_valid_session_id("123e4567-e89b-42d3-a456-426614174000"));
        assert!(!is_valid_session_id("123e4567-e89b-12d3-a456-426614174000")); // not version 4
        assert!(!is_valid_session_id("My Document"));
        assert!(!is_valid_session_id("../escape"));
        assert!(!is_valid_session_id(""));
    }

    #[test]
    fn session_snapshot_roundtrip_skips_control_and_non_uuid() {
        let _guard = lock_env();
        let unique = test_session_uuid();
        let dir = std::env::temp_dir().join(format!("nbcad-sessions-test-{unique}"));
        std::env::set_var("NBCAD_SESSION_DIR", &dir);
        write_session(&unique, "model.json", "{\"version\":1}").unwrap();
        write_session(
            &unique,
            "heartbeat.json",
            &format!(r#"{{"updated_ms":{},"generation":1}}"#, now_ms()),
        )
        .unwrap();
        fs::create_dir_all(dir.join("_ui")).unwrap();
        fs::create_dir_all(dir.join("document-name")).unwrap();
        let listed = list_sessions().unwrap();
        assert_eq!(listed, vec![unique.clone()]);
        assert!(!listed.iter().any(|session| session == "_ui"));
        let body = require_model_json(&unique).unwrap();
        assert!(body.contains("\"version\":1"));
        let list = sessions_list_json();
        assert_eq!(list["sessions"][0], unique);
        assert_eq!(list["session_details"][0]["has_model"], true);
        assert_eq!(list["session_details"][0]["heartbeat"]["stale"], false);
        assert_eq!(list["supports_live"], true);
        assert_eq!(list["session_mode"], "snapshot_or_live");
        assert_eq!(list["session_details"][0]["writer"]["writer"], "none");
        std::env::remove_var("NBCAD_SESSION_DIR");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_session_rejects_non_uuid() {
        let _guard = lock_env();
        let dir = std::env::temp_dir().join(format!("nbcad-sessions-bad-{}", now_ms()));
        std::env::set_var("NBCAD_SESSION_DIR", &dir);
        assert!(write_session("not-a-uuid", "model.json", "{}").is_err());
        std::env::remove_var("NBCAD_SESSION_DIR");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn claim_release_writer_and_conflict_meta() {
        let _guard = lock_env();
        let unique = test_session_uuid();
        let dir = std::env::temp_dir().join(format!("nbcad-sessions-writer-{unique}"));
        std::env::set_var("NBCAD_SESSION_DIR", &dir);

        let default = read_writer(&unique);
        assert_eq!(default["writer"], "none");
        assert_eq!(default["generation"], 0);

        claim_writer(&unique, "mcp", 3).unwrap();
        let claimed = read_writer(&unique);
        assert_eq!(claimed["writer"], "mcp");
        assert_eq!(claimed["generation"], 3);
        assert!(claimed["updated_ms"].as_u64().unwrap() > 0);

        // UI can overwrite the lock file (conflict is enforced by try_claim / callers).
        claim_writer(&unique, "ui", 4).unwrap();
        assert_eq!(read_writer(&unique)["writer"], "ui");
        assert_eq!(read_writer(&unique)["generation"], 4);
        assert!(try_claim_writer(&unique, "mcp", 5)
            .unwrap_err()
            .contains("session writer conflict"));

        release_writer(&unique).unwrap();
        let released = read_writer(&unique);
        assert_eq!(released["writer"], "none");
        assert_eq!(released["generation"], 4);

        write_model_revision(&unique, "{\"version\":2}", 5, "mcp").unwrap();
        let model = require_model_json(&unique).unwrap();
        assert!(model.contains("\"version\":2"));
        let heartbeat: Value =
            serde_json::from_str(&read_session_file(&unique, "heartbeat.json").unwrap()).unwrap();
        assert_eq!(heartbeat["generation"], 5);
        assert_eq!(heartbeat["source"], "mcp");
        assert_eq!(heartbeat["session_mode"], "live");

        assert!(claim_writer(&unique, "agent", 1).is_err());

        std::env::remove_var("NBCAD_SESSION_DIR");
        let _ = fs::remove_dir_all(&dir);
    }
}
