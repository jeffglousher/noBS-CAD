//! Named MCP wrappers for every `assembly_*` host method.
//!
//! Tool names match host.rs. Complex DTOs stay open (`additionalProperties`)
//! so Jack’s joint/occurrence shapes are not re-specified here.

use serde_json::{json, Value};

pub const ASSEMBLY_TOOL_COUNT: usize = 38;

pub enum AssemblyPayload {
    Empty,
    Object,
    Field(&'static str),
}

pub struct AssemblyTool {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub engine_method: &'static str,
    pub payload: AssemblyPayload,
    pub schema: Value,
}

fn empty() -> Value {
    json!({
        "type": "object",
        "properties": {},
        "additionalProperties": false
    })
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

fn open(description: &str, properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "description": description,
        "properties": properties,
        "required": required,
        "additionalProperties": true
    })
}

fn id() -> Value {
    json!({ "type": "integer", "minimum": 1 })
}

fn ids() -> Value {
    json!({
        "type": "array",
        "items": { "type": "integer", "minimum": 1 }
    })
}

fn transform() -> Value {
    json!({
        "type": "object",
        "description": "AssemblyTransformDto: translation [x,y,z] mm and rotation quaternion [x,y,z,w]",
        "additionalProperties": true
    })
}

fn tool(
    name: &'static str,
    title: &'static str,
    description: &'static str,
    payload: AssemblyPayload,
    schema: Value,
) -> AssemblyTool {
    AssemblyTool {
        name,
        title,
        description,
        engine_method: name,
        payload,
        schema,
    }
}

pub fn assembly_tools() -> Vec<AssemblyTool> {
    vec![
        tool(
            "assembly_document",
            "Read assembly document",
            "Return components, occurrences, joints, positions, motion studies, and contact sets. Prefer nbcad://assembly.",
            AssemblyPayload::Empty,
            empty(),
        ),
        tool(
            "assembly_set_document",
            "Replace assembly document",
            "Replace the entire assembly document. Prefer the named create/update tools.",
            AssemblyPayload::Object,
            open("AssemblyDocumentDto", json!({}), &[]),
        ),
        tool(
            "assembly_solution",
            "Solve assembly poses",
            "Forward-kinematics solution: occurrence and instance body poses. Prefer nbcad://assembly_solution.",
            AssemblyPayload::Empty,
            empty(),
        ),
        tool(
            "assembly_create_component",
            "Create component",
            "Define a reusable component from one or more solid body ids.",
            AssemblyPayload::Object,
            object(
                json!({
                    "name": { "type": "string", "minLength": 1 },
                    "body_ids": ids(),
                    "local_coordinate_system": transform(),
                    "absorb_promoted_bodies": { "type": "boolean" }
                }),
                &["name"],
            ),
        ),
        tool(
            "assembly_update_component",
            "Update component",
            "Replace a component definition. Pass { component: ComponentDefinitionDto }.",
            AssemblyPayload::Object,
            open(
                "UpdateComponentRequestDto",
                json!({ "component": { "type": "object" } }),
                &["component"],
            ),
        ),
        tool(
            "assembly_create_occurrence",
            "Create occurrence",
            "Insert an instance of a component in the occurrence tree.",
            AssemblyPayload::Object,
            object(
                json!({
                    "component_id": id(),
                    "name": { "type": "string", "minLength": 1 },
                    "parent_occurrence_id": { "oneOf": [id(), { "type": "null" }] },
                    "local_pose": transform()
                }),
                &["component_id", "name"],
            ),
        ),
        tool(
            "assembly_update_occurrence",
            "Update occurrence",
            "Replace an occurrence. Pass { occurrence: ComponentOccurrenceDto }.",
            AssemblyPayload::Object,
            open(
                "UpdateOccurrenceRequestDto",
                json!({ "occurrence": { "type": "object" } }),
                &["occurrence"],
            ),
        ),
        tool(
            "assembly_duplicate_occurrence",
            "Duplicate occurrence",
            "Clone an occurrence subtree. Optional local_pose places the copy atomically.",
            AssemblyPayload::Object,
            object(
                json!({
                    "occurrence_id": id(),
                    "parent_occurrence_id": { "oneOf": [id(), { "type": "null" }] },
                    "local_pose": transform()
                }),
                &["occurrence_id"],
            ),
        ),
        tool(
            "assembly_set_occurrence_grounded",
            "Ground occurrence",
            "Ground or unground an occurrence in the kinematic solve.",
            AssemblyPayload::Object,
            object(
                json!({
                    "occurrence_id": id(),
                    "grounded": { "type": "boolean" }
                }),
                &["occurrence_id", "grounded"],
            ),
        ),
        tool(
            "assembly_set_occurrence_pose",
            "Set occurrence pose",
            "Set the occurrence local transform (translation + quaternion).",
            AssemblyPayload::Object,
            object(
                json!({
                    "occurrence_id": id(),
                    "local_pose": transform()
                }),
                &["occurrence_id", "local_pose"],
            ),
        ),
        tool(
            "assembly_preview_joint",
            "Preview joint",
            "Solve a new joint without saving. Same payload as assembly_create_joint.",
            AssemblyPayload::Object,
            joint_request_schema(),
        ),
        tool(
            "assembly_create_joint",
            "Create joint",
            "Create a joint: rigid, revolute, slider, cylindrical, planar, ball, pin_slot, screw, or universal. Connectors reference stable faces/edges and frames.",
            AssemblyPayload::Object,
            joint_request_schema(),
        ),
        tool(
            "assembly_update_joint",
            "Update joint",
            "Replace a joint definition. Pass { joint: JointDefinitionDto, grounded_body_id?, grounded_occurrence_id? }.",
            AssemblyPayload::Object,
            open(
                "UpdateJointRequestDto",
                json!({ "joint": { "type": "object" } }),
                &["joint"],
            ),
        ),
        tool(
            "assembly_preview_joint_update",
            "Preview joint update",
            "Preview an edited joint without saving. Same payload as assembly_update_joint.",
            AssemblyPayload::Object,
            open(
                "UpdateJointRequestDto",
                json!({ "joint": { "type": "object" } }),
                &["joint"],
            ),
        ),
        tool(
            "assembly_delete_joint",
            "Delete joint",
            "Remove a joint by id.",
            AssemblyPayload::Field("joint_id"),
            object(json!({ "joint_id": id() }), &["joint_id"]),
        ),
        tool(
            "assembly_set_joint_enabled",
            "Enable or disable joint",
            "Include or exclude a joint from the kinematic solve.",
            AssemblyPayload::Object,
            object(
                json!({
                    "joint_id": id(),
                    "enabled": { "type": "boolean" }
                }),
                &["joint_id", "enabled"],
            ),
        ),
        tool(
            "assembly_set_joint_motion",
            "Set joint motion",
            "Set angle_offset_deg and linear_offset_mm on a joint.",
            AssemblyPayload::Object,
            object(
                json!({
                    "joint_id": id(),
                    "angle_offset_deg": { "type": "number" },
                    "linear_offset_mm": { "type": "number" }
                }),
                &["joint_id", "angle_offset_deg", "linear_offset_mm"],
            ),
        ),
        tool(
            "assembly_preview_joint_motion",
            "Preview joint motion",
            "Preview angle/linear offsets without saving. Same payload as assembly_set_joint_motion.",
            AssemblyPayload::Object,
            object(
                json!({
                    "joint_id": id(),
                    "angle_offset_deg": { "type": "number" },
                    "linear_offset_mm": { "type": "number" }
                }),
                &["joint_id", "angle_offset_deg", "linear_offset_mm"],
            ),
        ),
        tool(
            "assembly_set_joint_coordinates",
            "Set joint coordinates",
            "Set the full multi-DOF JointMotionStateDto for one joint.",
            AssemblyPayload::Object,
            open(
                "SetJointCoordinatesRequestDto",
                json!({ "motion": { "type": "object" } }),
                &["motion"],
            ),
        ),
        tool(
            "assembly_preview_joint_coordinates",
            "Preview joint coordinates",
            "Preview multi-DOF coordinates without saving.",
            AssemblyPayload::Object,
            open(
                "SetJointCoordinatesRequestDto",
                json!({ "motion": { "type": "object" } }),
                &["motion"],
            ),
        ),
        tool(
            "assembly_preview_mechanism_drag",
            "Preview mechanism drag",
            "Inverse-kinematics preview for dragging a body/occurrence toward a target pose.",
            AssemblyPayload::Object,
            open(
                "MechanismDragRequestDto",
                json!({
                    "body_id": id(),
                    "target_pose": { "type": "object" }
                }),
                &["body_id", "target_pose"],
            ),
        ),
        tool(
            "assembly_apply_joint_motions",
            "Apply joint motions",
            "Batch-apply JointMotionStateDto values.",
            AssemblyPayload::Object,
            object(
                json!({
                    "motions": { "type": "array", "items": { "type": "object" } }
                }),
                &["motions"],
            ),
        ),
        tool(
            "assembly_create_position",
            "Create assembly position",
            "Save a named pose (joint motion snapshot).",
            AssemblyPayload::Object,
            object(
                json!({
                    "name": { "type": "string", "minLength": 1 },
                    "motions": { "type": "array", "items": { "type": "object" } }
                }),
                &["name"],
            ),
        ),
        tool(
            "assembly_update_position",
            "Update assembly position",
            "Replace a saved AssemblyPositionDto.",
            AssemblyPayload::Object,
            open("AssemblyPositionDto", json!({ "id": id(), "name": { "type": "string" } }), &[]),
        ),
        tool(
            "assembly_delete_position",
            "Delete assembly position",
            "Remove a saved position by id.",
            AssemblyPayload::Field("position_id"),
            object(json!({ "position_id": id() }), &["position_id"]),
        ),
        tool(
            "assembly_apply_position",
            "Apply assembly position",
            "Apply a saved position to the live joints.",
            AssemblyPayload::Field("position_id"),
            object(json!({ "position_id": id() }), &["position_id"]),
        ),
        tool(
            "assembly_create_motion_study",
            "Create motion study",
            "Create a timeline motion study with a duration in seconds.",
            AssemblyPayload::Object,
            object(
                json!({
                    "name": { "type": "string", "minLength": 1 },
                    "duration_seconds": { "type": "number", "exclusiveMinimum": 0 }
                }),
                &["name", "duration_seconds"],
            ),
        ),
        tool(
            "assembly_update_motion_study",
            "Update motion study",
            "Replace a MotionStudyDto (keyframes, drivers).",
            AssemblyPayload::Object,
            open("MotionStudyDto", json!({ "id": id(), "name": { "type": "string" } }), &[]),
        ),
        tool(
            "assembly_delete_motion_study",
            "Delete motion study",
            "Remove a motion study by id.",
            AssemblyPayload::Field("study_id"),
            object(json!({ "study_id": id() }), &["study_id"]),
        ),
        tool(
            "assembly_sample_motion_study",
            "Sample motion study",
            "Evaluate poses at one time on a motion study.",
            AssemblyPayload::Object,
            object(
                json!({
                    "study_id": id(),
                    "time_seconds": { "type": "number", "minimum": 0 }
                }),
                &["study_id", "time_seconds"],
            ),
        ),
        tool(
            "assembly_export_motion_path_csv",
            "Export motion path CSV",
            "Sample a motion study and return a CSV path for selected occurrences.",
            AssemblyPayload::Object,
            object(
                json!({
                    "study_id": id(),
                    "sample_rate_hz": { "type": "number" },
                    "occurrence_ids": ids()
                }),
                &["study_id"],
            ),
        ),
        tool(
            "assembly_create_contact_set",
            "Create contact set",
            "Monitor clearance between two occurrence/body pairs. Optional stop_motion.",
            AssemblyPayload::Object,
            object(
                json!({
                    "name": { "type": "string", "minLength": 1 },
                    "occurrence_a": id(),
                    "body_a": id(),
                    "occurrence_b": id(),
                    "body_b": id(),
                    "clearance_mm": { "type": "number" },
                    "stop_motion": { "type": "boolean" }
                }),
                &["name", "occurrence_a", "body_a", "occurrence_b", "body_b"],
            ),
        ),
        tool(
            "assembly_update_contact_set",
            "Update contact set",
            "Replace a ContactSetDto.",
            AssemblyPayload::Object,
            open("ContactSetDto", json!({ "id": id(), "name": { "type": "string" } }), &[]),
        ),
        tool(
            "assembly_delete_contact_set",
            "Delete contact set",
            "Remove a contact set by id.",
            AssemblyPayload::Field("contact_set_id"),
            object(json!({ "contact_set_id": id() }), &["contact_set_id"]),
        ),
        tool(
            "assembly_interference_check",
            "Check interference",
            "Approximate pairwise interference / clearance for occurrences.",
            AssemblyPayload::Object,
            object(
                json!({
                    "occurrence_ids": ids(),
                    "clearance_threshold_mm": { "type": "number" }
                }),
                &[],
            ),
        ),
        tool(
            "assembly_evaluate_motion_study",
            "Evaluate motion study",
            "Sample a study and evaluate contact stops at a time.",
            AssemblyPayload::Object,
            object(
                json!({
                    "study_id": id(),
                    "time_seconds": { "type": "number", "minimum": 0 },
                    "previous_time_seconds": { "oneOf": [{ "type": "number" }, { "type": "null" }] },
                    "enforce_contacts": { "type": "boolean" }
                }),
                &["study_id", "time_seconds"],
            ),
        ),
        tool(
            "assembly_swept_collision_check",
            "Swept collision check",
            "Approximate swept-volume collisions along a motion study.",
            AssemblyPayload::Object,
            object(
                json!({
                    "study_id": id(),
                    "sample_rate_hz": { "type": "number" },
                    "clearance_threshold_mm": { "type": "number" },
                    "stop_at_first": { "type": "boolean" }
                }),
                &["study_id"],
            ),
        ),
        tool(
            "assembly_set_grounded_body",
            "Set grounded body",
            "Set the body held fixed while forward kinematics is evaluated. Pass null to clear.",
            AssemblyPayload::Field("body_id"),
            object(
                json!({
                    "body_id": { "oneOf": [id(), { "type": "null" }] }
                }),
                &["body_id"],
            ),
        ),
    ]
}

fn joint_request_schema() -> Value {
    open(
        "CreateJointRequestDto. kind: rigid|revolute|slider|cylindrical|planar|ball|pin_slot|screw|universal",
        json!({
            "name": { "type": "string", "minLength": 1 },
            "kind": {
                "type": "string",
                "enum": [
                    "rigid",
                    "revolute",
                    "slider",
                    "cylindrical",
                    "planar",
                    "ball",
                    "pin_slot",
                    "screw",
                    "universal"
                ]
            },
            "connector_a": { "type": "object" },
            "connector_b": { "type": "object" },
            "flipped": { "type": "boolean" },
            "angle_offset_deg": { "type": "number" },
            "linear_offset_mm": { "type": "number" },
            "grounded_body_id": { "oneOf": [id(), { "type": "null" }] }
        }),
        &["name", "kind", "connector_a", "connector_b"],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_unique_and_complete() {
        let tools = assembly_tools();
        assert_eq!(tools.len(), ASSEMBLY_TOOL_COUNT);
        let mut names = std::collections::HashSet::new();
        for tool in &tools {
            assert!(names.insert(tool.name), "duplicate {}", tool.name);
            assert_eq!(tool.engine_method, tool.name);
            assert!(tool.name.starts_with("assembly_"));
        }
    }
}
