//! MCP 2026-07-28 product surfaces: resources and prompts.
//!
//! Tools remain the mutation path. Resources are the read path for the same
//! product state. Prompts are the user-selectable recipes. `subscriptions/listen`
//! is out of this slice.
//!
//! # Main surface (shipped)
//!
//! Readable state (`resources/read`):
//! - `nbcad://document` — browser tree, settings, and feature history
//! - `nbcad://project` — versioned model.json
//! - `nbcad://scene` — tessellated bodies, faces, and edges
//! - `nbcad://drawing` — sheets and view intent (not generated HLR curves)
//! - `nbcad://focus` — disclosure pack plus workspace
//! - `nbcad://workspace` — solid vs drawing, plus focus
//! - `nbcad://sessions` — attachable UI session directories
//! - `nbcad://sketch` — active sketch snapshot
//! - `nbcad://sketches` — finished sketch snapshots
//! - `nbcad://profiles` — closed loops for extrude / revolve / sweep / loft / rib
//! - `nbcad://visibility` — hidden bodies / datums / sketches
//! - `nbcad://appearances` — per-body color / filament
//! - `nbcad://materials` — filament catalog
//! - `nbcad://features` — persisted solid / datum / body-op definitions
//!
//! Template: `nbcad://session/{session_id}` peeks a session `model.json`.
//!
//! Recipes (`prompts/get`): `model_box`, `model_hole`, `model_solid`,
//! `attach_ui`, `print_3mf`, `model_print_tool`, `model_print_kit`,
//! `import_step`, `export_step`, `drawing_read`, `drawing_sheet`,
//! `drawing_export`, `undo_history`, `invoke`.
//!
//! # Remaining (not this slice)
//!
//! - `nbcad://projection` — native HLR is expensive; keep `cad_drawing_project_sheet`
//! - `nbcad://tools` — use `cad_list_all_tools` / `tools/list`
//! - `nbcad://session/{id}/focus` and `…/window` — `cad_attach` already loads `focus.json`
//! - Dedicated construction-plane / body-ops / fillet prompts — tools exist;
//!   `model_solid` + `invoke` cover the loop
//! - `subscriptions/listen` — out of scope
//! - Collaboration comments — not a shipped Drawing / Solid product surface
//! - Jack's annotation-rich UI DXF writer — MCP has its own DXF / SVG export

use serde_json::{json, Value};

pub const RESOURCE_MIME: &str = "application/json";

/// Every `resources/list` URI on the main product surface.
pub const MAIN_RESOURCE_URIS: &[&str] = &[
    "nbcad://document",
    "nbcad://project",
    "nbcad://scene",
    "nbcad://drawing",
    "nbcad://focus",
    "nbcad://workspace",
    "nbcad://sessions",
    "nbcad://sketch",
    "nbcad://sketches",
    "nbcad://profiles",
    "nbcad://visibility",
    "nbcad://appearances",
    "nbcad://materials",
    "nbcad://features",
];

/// Every `prompts/list` recipe on the main product surface.
pub const MAIN_PROMPT_NAMES: &[&str] = &[
    "model_box",
    "model_hole",
    "model_solid",
    "attach_ui",
    "print_3mf",
    "model_print_tool",
    "model_print_kit",
    "import_step",
    "export_step",
    "drawing_read",
    "drawing_sheet",
    "drawing_export",
    "undo_history",
    "invoke",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceKind {
    Document,
    Project,
    Scene,
    Drawing,
    Focus,
    Workspace,
    Sessions,
    Sketch,
    Sketches,
    Profiles,
    Visibility,
    Appearances,
    Materials,
    Features,
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
            resource("nbcad://focus", "focus", "Disclosure focus", "Active focus pack, advertisement mode, and workspace."),
            resource("nbcad://workspace", "workspace", "Product workspace", "Solid (model/sketch) vs drawing workspace, plus current focus."),
            resource("nbcad://sessions", "sessions", "UI sessions", "Attachable session directories under NBCAD_SESSION_DIR."),
            resource("nbcad://sketch", "sketch", "Active sketch", "In-progress sketch snapshot, or null when none is active."),
            resource("nbcad://sketches", "sketches", "Finished sketches", "Retained snapshots of every finished sketch."),
            resource("nbcad://profiles", "profiles", "Closed profiles", "Closed profile loops and path references for solid tools."),
            resource("nbcad://visibility", "visibility", "Browser visibility", "Hidden body ids, datum ids, and sketch names."),
            resource("nbcad://appearances", "appearances", "Body appearances", "Per-body color and filament assignments."),
            resource("nbcad://materials", "materials", "Material catalog", "Built-in filament presets for 3MF appearance."),
            resource("nbcad://features", "features", "Feature definitions", "Persisted extrude, revolve, sweep, loft, rib, fillet, chamfer, hole, datum, and body-op definitions."),
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
                "model_solid",
                "Create a solid from profiles",
                "Revolve, sweep, loft, or rib after reading closed profiles.",
                &[("kind", "revolve, sweep, loft, or rib (default revolve)", false)]
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
                "model_print_tool",
                "Walk through a printable tool",
                "Sketch, build, and export a useful small 3D-printed part (desk cable clip).",
                &[]
            ),
            prompt_desc(
                "model_print_kit",
                "Teach a printed turntable",
                "Synthesis exam: assembled printed turntable with cone/land thrust and a sleeve bushing, modeled for a 0.4 mm Bambu nozzle.",
                &[("nozzle_mm", "Nozzle diameter used as the diametral clearance (default 0.4)", false)]
            ),
            prompt_desc(
                "import_step",
                "Import STEP",
                "Import a STEP/STP exchange file as a persistent history feature.",
                &[("file_name", "Original file name, e.g. part.step", false)]
            ),
            prompt_desc(
                "export_step",
                "Export STEP",
                "Export active bodies as AP242 STEP (CAD interchange, not a slicer project).",
                &[]
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
            prompt_desc(
                "drawing_export",
                "Export a drawing",
                "Project hidden-line views and write DXF, SVG, or a 1:1 profile DXF.",
                &[("format", "dxf, svg, or profile_dxf (default dxf)", false)]
            ),
            prompt_desc(
                "undo_history",
                "Undo or edit history",
                "Application undo/redo, plus timeline rollback, delete, and reorder.",
                &[]
            ),
            prompt_desc(
                "invoke",
                "Invoke any engine method",
                "Mechanical escape hatch: cad_invoke for host methods, cad_drawing_command for drawing ops.",
                &[("method", "host.rs method name when using cad_invoke", false)]
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
        "nbcad://workspace" => Ok(ResourceKind::Workspace),
        "nbcad://sessions" => Ok(ResourceKind::Sessions),
        "nbcad://sketch" => Ok(ResourceKind::Sketch),
        "nbcad://sketches" => Ok(ResourceKind::Sketches),
        "nbcad://profiles" => Ok(ResourceKind::Profiles),
        "nbcad://visibility" => Ok(ResourceKind::Visibility),
        "nbcad://appearances" => Ok(ResourceKind::Appearances),
        "nbcad://materials" => Ok(ResourceKind::Materials),
        "nbcad://features" => Ok(ResourceKind::Features),
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
             4. sketch_add_rectangle then sketch_finish → read nbcad://profiles (or sketch_profiles).\n\
             5. solid_extrude a new_body from profile 0.\n\
             6. Confirm with nbcad://scene and nbcad://features. Use returned ids in later calls."
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
                 4. Confirm the body in nbcad://scene and the definition in nbcad://features."
            )
        }
        "model_solid" => {
            let kind = arguments
                .get("kind")
                .and_then(Value::as_str)
                .unwrap_or("revolve");
            format!(
                "Create a solid from closed profiles (kind={kind}).\n\
                 1. Read nbcad://profiles (or sketch_profiles) after finishing one or more sketches.\n\
                 2. cad_set_focus focus=solid.\n\
                 3. revolve: solid_revolve around a sketch line or manual axis.\n\
                 4. sweep: solid_sweep one profile along a connected line/arc/circle/spline path.\n\
                 5. loft: solid_loft two or more ordered profiles; optional centerline/guide and G0/G1/G2.\n\
                 6. rib: solid_rib from a line/arc/circle/spline with Distance, To Next, Up to Face, or Through All.\n\
                 7. Confirm nbcad://scene and nbcad://features. Edit later with solid_edit_* / nbcad://features ids."
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
                 6. cad_set_workspace solid|drawing (or read nbcad://workspace) to follow the live UI.\n\
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
                 2. Read nbcad://materials and nbcad://appearances.\n\
                 3. solid_export_preflight, then optional set_body_appearance.\n\
                 4. solid_export_3mf with slicer_target={target} (bambu|orca|prusa|cura).\n\
                 5. STL is geometry-only; STEP is CAD interchange (see export_step)."
            )
        }
        "model_print_tool" => {
            "Walk through a useful small 3D-printed tool (desk cable clip).\n\
             1. cad_list_all_tools (or resources/list). cad_agent_guidance is not on this server.\n\
             2. Optional UI: read nbcad://sessions then cad_attach mode=live. cad_set_workspace solid.\n\
             3. cad_new_project only for a fresh document. cad_set_document_name.\n\
             4. cad_set_focus sketch. sketch_begin on origin_plane xy. sketch_set_grid_snap enabled=false.\n\
             5. sketch_add_rectangle_locked with width_mm/height_mm for the plate. sketch_finish. Read nbcad://profiles.\n\
             6. cad_set_focus solid. solid_extrude a new_body (plate 3–4 mm). Confirm nbcad://scene.\n\
             7. sketch_begin on planar_face (plate top), face_origin=face_center. Add locked rectangles for clip walls.\n\
             8. sketch_finish → solid_extrude join (8–12 mm). Do not stop at a single origin-plane extrusion.\n\
             9. cad_set_focus modify. solid_fillet / solid_chamfer / solid_hole using Body/Face/Edge ids from nbcad://scene.\n\
             10. cad_set_focus print. material_catalog → set_body_appearance → solid_export_preflight → solid_export_3mf slicer_target=bambu_studio.\n\
             11. Confirm nbcad://document, nbcad://features, nbcad://appearances. Print flat; keep walls ≥ 2 mm."
                .to_string()
        }
        "model_print_kit" => {
            let nozzle = arguments
                .get("nozzle_mm")
                .and_then(Value::as_f64)
                .unwrap_or(0.4);
            format!(
                "CAD synthesis tutor — build a fully printed turntable as an assembled stack, then grade FDM tolerancing.\n\
                 Spec: scripts/fixtures/print-kit-tutor.spec.json (id fdm-print-turntable). Rerun: npm run test:mcp-print-kit.\n\
                 Exam (headless is enough; cad_attach is optional live UI):\n\
                 1. prompts/get model_print_kit. cad_list_all_tools. cad_new_project. cad_set_document_name Print Kit Tutor.\n\
                 2. Design input: nozzle={nozzle} mm. Every printed-to-printed running/slip fit is +{nozzle} mm diametral in CAD. No FDM press fits. Leave slicer XY hole compensation at 0.\n\
                 3. Build a product, not a cage. The platter is larger than the foot. The keeper is a small collar — not a second lid. Assembly order: base → shaft → platter → keeper → printed bushing → cap. noBS CAD has no mates; place bodies by construction. That gap is real.\n\
                 4. Base: Ø48 × 6 foot. Cut a 45° conical thrust cup (female r5) with a Ø3 relief at the apex. No posts.\n\
                 5. Shaft: revolve on XZ. Smaller male cone (r4.8) plus a Ø13 × 0.8 thrust land with 0.20 float above the base — do not use a same-angle lifted cone (parallel surfaces never touch). Ø8 journal, Ø16 shoulder, double-D 6.0 only in the platter zone, upper journal through the keeper.\n\
                 6. Platter: Ø72 × 6 that sits on the shoulder (do not swallow the shoulder), Ø8.4 bore, double-D 6.4, rim well Ø64 × 1.2 from the top, 3× Ø16 wells on R22 at 120°. A leftover helical C-bucket is not a turntable.\n\
                 7. Keeper Ø28 × 6 at z=23.5 (small collar, not a second lid). Ø8.4 journal. Bushing seat Ø14.4 × 4 from the top so a 2 mm land remains. Printed bushing Ø8.4/Ø14 × 4 on that land. Cap Ø20 × 2.4 with 0.20 float. Functional holes are XY circles. Disable grid snap. Prefer locked circles.\n\
                 8. cad_set_focus print. set_body_appearance. solid_export_preflight. solid_export_3mf slicer_target=bambu_studio.\n\
                 9. Grade: timeline ok, ≥6 coaxial bodies, platter larger than the foot, small keeper, cone/land thrust, double-D drive, even 3-well pattern, mounted platter, 3MF is a PK zip.\n\
                 Later: catalog metal bearings from a standard table at larger sizes. Not this exam."
            )
        }
        "import_step" => {
            let file_name = arguments
                .get("file_name")
                .and_then(Value::as_str)
                .unwrap_or("part.step");
            format!(
                "Import a STEP/STP file as a persistent history feature.\n\
                 1. cad_set_focus focus=solid.\n\
                 2. solid_import_step file_name={file_name} data_base64=<exchange bytes>.\n\
                 3. Confirm bodies in nbcad://scene and the import in nbcad://features.\n\
                 4. Optional set_body_appearance / cad_set_project_visibility after import."
            )
        }
        "export_step" => {
            "Export AP242 STEP for CAD interchange (not a slicer project).\n\
             1. Read nbcad://scene for body ids (omit body_ids to export every active body).\n\
             2. cad_set_focus focus=inspect.\n\
             3. solid_export_step → format=step, encoding=base64, bytes_base64=…\n\
             4. Prefer solid_export_3mf / print_3mf for slicers. STL is geometry-only."
                .to_string()
        }
        "drawing_read" => {
            "Inspect technical drawings stored in the project model.\n\
             1. cad_set_focus focus=drawing. Read nbcad://workspace if the live UI should follow.\n\
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
        "drawing_export" => {
            let format = arguments
                .get("format")
                .and_then(Value::as_str)
                .unwrap_or("dxf");
            format!(
                "Export a technical drawing (format={format}).\n\
                 1. cad_set_focus focus=drawing. Optional cad_set_workspace drawing.\n\
                 2. Read nbcad://drawing. Create/layout a sheet if none exists (drawing_sheet).\n\
                 3. cad_drawing_project_sheet for native HLR curves from a saved view.\n\
                 4. dxf: cad_drawing_export_dxf. svg: cad_drawing_export_svg (paper / print stand-in).\n\
                 5. profile_dxf: cad_drawing_export_profile_dxf for a 1:1 sketch-plane profile.\n\
                 MCP writes its own DXF/SVG; do not wait for the desktop File menu."
            )
        }
        "undo_history" => {
            "Undo, redo, or edit feature history.\n\
             1. Read nbcad://document (feature list + rollback_index) and nbcad://workspace.\n\
             2. cad_undo / cad_redo follow the UI: drawing workspace → drawing history; active sketch → sketch undo; else delete/restore the latest solid feature or step the rollback marker.\n\
             3. Timeline edits: solid_set_rollback, solid_delete_feature, solid_reorder_feature.\n\
             4. Confirm nbcad://document, nbcad://features, and nbcad://scene."
                .to_string()
        }
        "invoke" => {
            let method = arguments
                .get("method")
                .and_then(Value::as_str)
                .unwrap_or("a host.rs method");
            format!(
                "Mechanical full control when a named tool is the wrong shape.\n\
                 1. Prefer the named sketch_ / solid_ / cad_drawing_* tool when you know the feature.\n\
                 2. cad_invoke method={method} arguments={{…}}. Omit arguments for empty host methods.\n\
                 3. solid_prepare_* and project_prepare_* run OCCT replay. solid_commit is rejected.\n\
                 4. drawing_command takes {{op, …fields}} — or call cad_drawing_command with the same shape.\n\
                 5. Read nbcad://document / nbcad://scene / nbcad://drawing after mutating."
            )
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

fn prompt_title(name: &str) -> String {
    match name {
        "model_box" => "Model a box".to_string(),
        "model_hole" => "Add a hole".to_string(),
        "model_solid" => "Create a solid from profiles".to_string(),
        "attach_ui" => "Attach to the UI session".to_string(),
        "print_3mf" => "Export 3MF".to_string(),
        "model_print_tool" => "Walk through a printable tool".to_string(),
        "model_print_kit" => "Teach an FDM-tolerant print kit".to_string(),
        "import_step" => "Import STEP".to_string(),
        "export_step" => "Export STEP".to_string(),
        "drawing_read" => "Inspect drawings".to_string(),
        "drawing_sheet" => "Create a drawing sheet".to_string(),
        "drawing_export" => "Export a drawing".to_string(),
        "undo_history" => "Undo or edit history".to_string(),
        "invoke" => "Invoke any engine method".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog_uris(list: &Value) -> Vec<&str> {
        list["resources"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item["uri"].as_str())
            .collect()
    }

    fn catalog_prompt_names(list: &Value) -> Vec<&str> {
        list["prompts"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|item| item["name"].as_str())
            .collect()
    }

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
        assert_eq!(
            parse_resource_uri("nbcad://workspace").unwrap(),
            ResourceKind::Workspace
        );
        assert_eq!(
            parse_resource_uri("nbcad://profiles").unwrap(),
            ResourceKind::Profiles
        );
        assert_eq!(
            parse_resource_uri("nbcad://features").unwrap(),
            ResourceKind::Features
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
        let resource_catalog = list_resources();
        let uris = catalog_uris(&resource_catalog);
        assert_eq!(uris, MAIN_RESOURCE_URIS);
        for uri in MAIN_RESOURCE_URIS {
            assert!(
                parse_resource_uri(uri).is_ok(),
                "main resource {uri} must parse"
            );
        }
        let prompt_catalog = list_prompts();
        let prompts = catalog_prompt_names(&prompt_catalog);
        assert_eq!(prompts, MAIN_PROMPT_NAMES);
        for name in MAIN_PROMPT_NAMES {
            let prompt = get_prompt(name, &json!({})).unwrap();
            let text = prompt["messages"][0]["content"]["text"].as_str().unwrap();
            assert!(text.len() > 40, "prompt {name} recipe is too short: {text}");
            assert_ne!(
                prompt_title(name),
                *name,
                "prompt {name} needs a human title"
            );
        }
        assert!(
            get_prompt("model_box", &json!({})).unwrap()["messages"][0]["content"]["text"]
                .as_str()
                .unwrap()
                .contains("sketch_begin")
        );
        assert!(
            get_prompt("model_solid", &json!({"kind": "loft"})).unwrap()["messages"][0]["content"]
                ["text"]
                .as_str()
                .unwrap()
                .contains("solid_loft")
        );
        assert!(
            get_prompt("model_print_tool", &json!({})).unwrap()["messages"][0]["content"]["text"]
                .as_str()
                .unwrap()
                .contains("sketch_add_rectangle_locked")
        );
        assert!(
            get_prompt("model_print_kit", &json!({})).unwrap()["messages"][0]["content"]["text"]
                .as_str()
                .unwrap()
                .contains("thrust")
        );
        assert!(
            get_prompt("import_step", &json!({})).unwrap()["messages"][0]["content"]["text"]
                .as_str()
                .unwrap()
                .contains("solid_import_step")
        );
        assert!(
            get_prompt("export_step", &json!({})).unwrap()["messages"][0]["content"]["text"]
                .as_str()
                .unwrap()
                .contains("solid_export_step")
        );
        assert!(
            get_prompt("drawing_export", &json!({"format": "svg"})).unwrap()["messages"][0]
                ["content"]["text"]
                .as_str()
                .unwrap()
                .contains("cad_drawing_export_svg")
        );
        assert!(
            get_prompt("undo_history", &json!({})).unwrap()["messages"][0]["content"]["text"]
                .as_str()
                .unwrap()
                .contains("cad_undo")
        );
        assert!(
            get_prompt("invoke", &json!({"method": "add_line"})).unwrap()["messages"][0]["content"]
                ["text"]
                .as_str()
                .unwrap()
                .contains("cad_invoke")
        );
        assert!(get_prompt("unknown", &json!({})).is_err());
    }
}
