//! MCP 2026-07-28 product surfaces: resources and prompts.
//!
//! Tools remain the mutation path. Resources are the read path for the same
//! document / scene / drawing / session state. Prompts are the user-selectable
//! recipes for those surfaces. `subscriptions/listen` is out of this slice.

use serde_json::{json, Value};

pub const RESOURCE_MIME: &str = "application/json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKind {
    Document,
    Project,
    Scene,
    Drawing,
    Focus,
    Sessions,
    Session(String),
}

pub fn list_resources() -> Value {
    json!({
        "resultType": "complete",
        "resources": [
            resource("nbcad://document", "document", "CAD document", "Browser tree, settings, and feature history."),
            resource("nbcad://project", "project", "Project model", "Versioned model.json (sketches, solids, drawings, visibility)."),
            resource("nbcad://scene", "scene", "Solid scene", "Tessellated bodies, faces, and edges from the last recompute."),
            resource("nbcad://drawing", "drawing", "Drawing document", "Technical drawing sheets and view intent (not generated HLR curves)."),
            resource("nbcad://focus", "focus", "Disclosure focus", "Active focus pack and advertisement mode."),
            resource("nbcad://sessions", "sessions", "UI sessions", "Attachable session directories under NBCAD_SESSION_DIR."),
        ],
        "ttlMs": 5_000,
        "cacheScope": "private"
    })
}

pub fn list_resource_templates() -> Value {
    json!({
        "resultType": "complete",
        "resourceTemplates": [
            {
                "uriTemplate": "nbcad://session/{session_id}",
                "name": "session",
                "title": "Session model",
                "description": "Read model.json for a UUID session directory without attaching.",
                "mimeType": RESOURCE_MIME
            }
        ],
        "ttlMs": 3_600_000,
        "cacheScope": "public"
    })
}

pub fn list_prompts() -> Value {
    json!({
        "resultType": "complete",
        "prompts": [
            prompt_desc("model_box", "Model a box", "Sketch a rectangle on XY and extrude a new body.", &[]),
            prompt_desc(
                "model_hole",
                "Add a hole",
                "Put a hole on a face of the current solid. Requires an existing body.",
                &[("face_id", "Stable face id from solid_scene / nbcad://scene", false)]
            ),
            prompt_desc(
                "attach_ui",
                "Attach to the UI session",
                "Discover NBCAD_SESSION_DIR sessions and live-attach with writer lock.",
                &[("mode", "read_only or live (default live)", false)]
            ),
            prompt_desc(
                "print_3mf",
                "Export 3MF",
                "Preflight then export 3MF with materials/colors for a slicer target.",
                &[("slicer_target", "bambu, orca, prusa, or cura", false)]
            ),
            prompt_desc(
                "drawing_read",
                "Inspect drawings",
                "Read the drawing document, command tools, HLR projection, and MCP-native DXF/SVG export.",
                &[]
            ),
            prompt_desc(
                "drawing_sheet",
                "Create a drawing sheet",
                "Create a sheet and auto-layout standard views from the current 3D model.",
                &[]
            ),
        ],
        "ttlMs": 3_600_000,
        "cacheScope": "public"
    })
}

pub fn parse_resource_uri(uri: &str) -> Result<ResourceKind, String> {
    let uri = uri.trim();
    match uri {
        "nbcad://document" => Ok(ResourceKind::Document),
        "nbcad://project" => Ok(ResourceKind::Project),
        "nbcad://scene" => Ok(ResourceKind::Scene),
        "nbcad://drawing" => Ok(ResourceKind::Drawing),
        "nbcad://focus" => Ok(ResourceKind::Focus),
        "nbcad://sessions" => Ok(ResourceKind::Sessions),
        other => {
            const PREFIX: &str = "nbcad://session/";
            if let Some(session_id) = other.strip_prefix(PREFIX) {
                if session_id.is_empty()
                    || session_id.contains('/')
                    || session_id.contains('\\')
                    || session_id.contains("..")
                {
                    return Err(format!("invalid session resource '{other}'"));
                }
                return Ok(ResourceKind::Session(session_id.to_string()));
            }
            Err(format!("unknown resource '{other}'"))
        }
    }
}

pub fn resource_contents(uri: &str, body: &Value) -> Value {
    let text = match body {
        Value::String(text) => text.clone(),
        other => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    };
    json!({
        "resultType": "complete",
        "contents": [{
            "uri": uri,
            "mimeType": RESOURCE_MIME,
            "text": text
        }],
        "ttlMs": 0,
        "cacheScope": "private"
    })
}

pub fn get_prompt(name: &str, arguments: &Value) -> Result<Value, String> {
    let text = match name {
        "model_box" => {
            "Model a rectangular box on the headless document.\n\
             1. Read nbcad://document and nbcad://scene (or cad_document / solid_scene).\n\
             2. cad_set_focus focus=sketch.\n\
             3. sketch_begin on origin_plane xy.\n\
             4. sketch_add_rectangle then sketch_finish → sketch_profiles.\n\
             5. solid_extrude a new_body from profile 0.\n\
             6. Confirm with nbcad://scene. Use returned ids in later calls."
                .to_string()
        }
        "model_hole" => {
            let face = arguments
                .get("face_id")
                .map(|value| value.to_string())
                .unwrap_or_else(|| "a face_id from nbcad://scene".to_string());
            format!(
                "Add a hole on {face}.\n\
                 1. Read nbcad://scene and pick a planar face if face_id was not given.\n\
                 2. cad_set_focus focus=modify.\n\
                 3. Call solid_hole with that face and a standard (ISO metric or Unified).\n\
                 4. Confirm the body in nbcad://scene."
            )
        }
        "attach_ui" => {
            let mode = arguments
                .get("mode")
                .and_then(Value::as_str)
                .unwrap_or("live");
            format!(
                "Co-link this MCP process to a running UI session.\n\
                 1. Read nbcad://sessions (or cad_list_sessions).\n\
                 2. Pick a UUID with has_model=true and a fresh heartbeat.\n\
                 3. cad_attach session_id=<uuid> mode={mode}.\n\
                 4. Live mode takes writer=mcp from ui/none and writebacks model.json.\n\
                 5. If the UI later holds the lock, cad_refresh loads it without stealing.\n\
                 Headless goldens do not need attach."
            )
        }
        "print_3mf" => {
            let target = arguments
                .get("slicer_target")
                .and_then(Value::as_str)
                .unwrap_or("bambu");
            format!(
                "Export a print-ready 3MF (materials/colors are in the file; this is not a sliced project).\n\
                 1. cad_set_focus focus=print.\n\
                 2. solid_export_preflight, then optional set_body_appearance.\n\
                 3. solid_export_3mf with slicer_target={target} (bambu|orca|prusa|cura).\n\
                 4. STL is geometry-only; STEP is CAD interchange."
            )
        }
        "drawing_read" => {
            "Inspect technical drawings stored in the project model.\n\
             1. cad_set_focus focus=drawing.\n\
             2. Read nbcad://drawing or call cad_drawing_document.\n\
             3. Sheets store view intent (direction, scale, placement), not HLR curves.\n\
             4. Prefer cad_drawing_* command tools (create_sheet, auto_layout, add_note, dimensions) over cad_set_drawing_document.\n\
             5. cad_drawing_project_sheet runs native HLR; cad_drawing_export_dxf / cad_drawing_export_svg write paper interchange.\n\
             6. cad_drawing_export_profile_dxf writes a 1:1 sketch-plane profile. cad_set_workspace drawing shows the live UI sheet."
                .to_string()
        }
        "drawing_sheet" => {
            "Create a drawing sheet and place standard views.\n\
             1. cad_set_focus focus=drawing.\n\
             2. cad_drawing_create_sheet (ISO/ANSI, format, projection method).\n\
             3. cad_drawing_auto_layout to place front/top/side/iso from the current 3D scene.\n\
             4. Optional cad_drawing_add_note / dimension tools.\n\
             5. cad_drawing_project_sheet for HLR curves, then cad_drawing_export_dxf or cad_drawing_export_svg."
                .to_string()
        }
        other => return Err(format!("unknown prompt '{other}'")),
    };
    Ok(json!({
        "resultType": "complete",
        "description": prompt_title(name),
        "messages": [{
            "role": "user",
            "content": { "type": "text", "text": text }
        }]
    }))
}

fn resource(uri: &str, name: &str, title: &str, description: &str) -> Value {
    json!({
        "uri": uri,
        "name": name,
        "title": title,
        "description": description,
        "mimeType": RESOURCE_MIME
    })
}

fn prompt_desc(
    name: &str,
    title: &str,
    description: &str,
    arguments: &[(&str, &str, bool)],
) -> Value {
    let args: Vec<Value> = arguments
        .iter()
        .map(|(arg_name, arg_description, required)| {
            json!({
                "name": arg_name,
                "description": arg_description,
                "required": required
            })
        })
        .collect();
    json!({
        "name": name,
        "title": title,
        "description": description,
        "arguments": args
    })
}

fn prompt_title(name: &str) -> &'static str {
    match name {
        "model_box" => "Model a box",
        "model_hole" => "Add a hole",
        "attach_ui" => "Attach to the UI session",
        "print_3mf" => "Export 3MF",
        "drawing_read" => "Inspect drawings",
        "drawing_sheet" => "Create a drawing sheet",
        _ => name,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_uris_parse_and_reject_traversal() {
        assert_eq!(
            parse_resource_uri("nbcad://document").unwrap(),
            ResourceKind::Document
        );
        assert_eq!(
            parse_resource_uri("nbcad://drawing").unwrap(),
            ResourceKind::Drawing
        );
        match parse_resource_uri("nbcad://session/01732db8-694c-886c-87d8-c2c64537d673").unwrap() {
            ResourceKind::Session(id) => assert_eq!(id, "01732db8-694c-886c-87d8-c2c64537d673"),
            other => panic!("unexpected {other:?}"),
        }
        assert!(parse_resource_uri("nbcad://session/../etc/passwd").is_err());
        assert!(parse_resource_uri("nbcad://nope").is_err());
        assert!(parse_resource_uri("file:///etc/passwd").is_err());
    }

    #[test]
    fn catalogs_include_product_surfaces() {
        let resources = list_resources();
        let uris: Vec<_> = resources["resources"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item["uri"].as_str())
            .collect();
        assert!(uris.contains(&"nbcad://document"));
        assert!(uris.contains(&"nbcad://drawing"));
        assert!(uris.contains(&"nbcad://sessions"));
        let prompt_catalog = list_prompts();
        let prompts: Vec<_> = prompt_catalog["prompts"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item["name"].as_str())
            .collect();
        assert!(prompts.contains(&"model_box"));
        assert!(prompts.contains(&"print_3mf"));
        assert!(prompts.contains(&"drawing_read"));
        assert!(prompts.contains(&"drawing_sheet"));
        assert!(
            get_prompt("model_box", &json!({})).unwrap()["messages"][0]["content"]["text"]
                .as_str()
                .unwrap()
                .contains("sketch_begin")
        );
        assert!(get_prompt("unknown", &json!({})).is_err());
    }
}
