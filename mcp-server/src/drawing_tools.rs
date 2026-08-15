//! MCP drawing command catalog.
//!
//! Each named tool injects `op` and dispatches through engine `drawing_command`.
//! Projection tools are separate: they call OCCT HLR and do not mutate the DTO.

use serde_json::{json, Value};

pub struct DrawingTool {
    pub name: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub op: &'static str,
    pub schema: Value,
}

fn object(properties: Value, required: &[&str]) -> Value {
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

fn open_object(description: &str) -> Value {
    json!({
        "type": "object",
        "description": description,
        "additionalProperties": true
    })
}

fn entity_id() -> Value {
    json!({ "type": "integer", "minimum": 1 })
}

fn position() -> Value {
    json!({
        "type": "array",
        "items": { "type": "number" },
        "minItems": 2,
        "maxItems": 2
    })
}

pub fn drawing_command_tools() -> Vec<DrawingTool> {
    vec![
        DrawingTool {
            name: "cad_drawing_create_sheet",
            title: "Create drawing sheet",
            description: "Add an empty framed sheet (ISO/ANSI, format, projection method, title block). Same command as the Drawing ribbon New Sheet setup.",
            op: "create_sheet",
            schema: object(
                json!({
                    "standard": { "type": "string", "enum": ["iso", "ansi"] },
                    "format": { "type": "string" },
                    "orientation": { "type": "string", "enum": ["landscape", "portrait"] },
                    "projection_method": { "type": "string", "enum": ["first_angle", "third_angle"] },
                    "tolerance_note": open_object("DrawingToleranceNoteDto"),
                    "title": { "type": "string" },
                    "drawing_number": { "type": "string" },
                    "revision": { "type": "string" },
                    "author": { "type": "string" }
                }),
                &[],
            ),
        },
        DrawingTool {
            name: "cad_drawing_set_active_sheet",
            title: "Set active drawing sheet",
            description: "Select which sheet later drawing commands mutate.",
            op: "set_active_sheet",
            schema: object(json!({ "sheet_id": entity_id() }), &["sheet_id"]),
        },
        DrawingTool {
            name: "cad_drawing_delete_sheet",
            title: "Delete drawing sheet",
            description: "Remove a sheet and its views/annotations.",
            op: "delete_sheet",
            schema: object(json!({ "sheet_id": entity_id() }), &["sheet_id"]),
        },
        DrawingTool {
            name: "cad_drawing_update_sheet",
            title: "Update drawing sheet",
            description: "Patch the active sheet (or sheet_id): format, title block, style, table positions.",
            op: "update_sheet",
            schema: object(
                json!({
                    "sheet_id": entity_id(),
                    "patch": open_object("Partial DrawingSheetDto")
                }),
                &["patch"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_auto_layout",
            title: "Auto-layout drawing views",
            description: "On an empty sheet, place front/top/right/isometric using the sheet's first- or third-angle convention.",
            op: "auto_layout",
            schema: object(json!({}), &[]),
        },
        DrawingTool {
            name: "cad_drawing_add_view",
            title: "Add drawing view",
            description: "Place a standard view (front/top/left/right/rear/bottom/isometric/custom) with projected alignment to the group root.",
            op: "add_view",
            schema: object(
                json!({
                    "kind": { "type": "string", "enum": ["front", "rear", "left", "right", "top", "bottom", "isometric", "custom"] },
                    "position": position(),
                    "parent_view_id": entity_id(),
                    "scale": { "type": "number", "exclusiveMinimum": 0 }
                }),
                &["kind"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_update_view",
            title: "Update drawing view",
            description: "Patch a view (position, scale, hidden lines, body filter). Related orthographic views keep alignment/scale.",
            op: "update_view",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "patch": open_object("Partial DrawingViewDto")
                }),
                &["view_id", "patch"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_delete_view",
            title: "Delete drawing view",
            description: "Remove a view and annotations attached to it. Children become free.",
            op: "delete_view",
            schema: object(json!({ "view_id": entity_id() }), &["view_id"]),
        },
        DrawingTool {
            name: "cad_drawing_add_derived_view",
            title: "Add derived drawing view",
            description: "Add a section, detail, auxiliary, broken, or removed-section view from a parent view plus derivation.",
            op: "add_derived_view",
            schema: object(
                json!({
                    "kind": { "type": "string", "enum": ["section", "detail", "auxiliary", "broken", "removed_section"] },
                    "parent_view_id": entity_id(),
                    "position": position(),
                    "derivation": open_object("DrawingViewDerivationDto")
                }),
                &["kind", "parent_view_id", "position", "derivation"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_note",
            title: "Add drawing note",
            description: "Add a free note on the active sheet (paper millimetres from the upper-left corner).",
            op: "add_note",
            schema: object(
                json!({
                    "position": position(),
                    "text": { "type": "string" }
                }),
                &["position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_linear_dimension",
            title: "Add linear dimension",
            description: "Aligned/horizontal/vertical dimension between two topology anchors on a view.",
            op: "add_linear_dimension",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "first": open_object("DrawingTopologyAnchorRefDto"),
                    "second": open_object("DrawingTopologyAnchorRefDto"),
                    "mode": { "type": "string", "enum": ["aligned", "horizontal", "vertical"] },
                    "offset": { "type": "number" }
                }),
                &["view_id", "first", "second"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_line_dimension",
            title: "Add line dimension",
            description: "Length of one edge, or distance/angle between two straight edges.",
            op: "add_line_dimension",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "first": open_object("DrawingLineRefDto"),
                    "second": open_object("DrawingLineRefDto"),
                    "mode": { "type": "string", "enum": ["length", "distance", "angle"] },
                    "position": position()
                }),
                &["view_id", "first", "mode", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_point_line_dimension",
            title: "Add point-to-line dimension",
            description: "Dimension from a topology point to a straight edge.",
            op: "add_point_line_dimension",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "point": open_object("DrawingTopologyAnchorRefDto"),
                    "line": open_object("DrawingLineRefDto"),
                    "position": position()
                }),
                &["view_id", "point", "line", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_radial_dimension",
            title: "Add diameter or radius",
            description: "Radial dimension on a circular edge. Diameter requires a closed circle.",
            op: "add_radial_dimension",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "feature": open_object("DrawingCircularRefDto"),
                    "mode": { "type": "string", "enum": ["diameter", "radius"] }
                }),
                &["view_id", "feature", "mode"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_angular_dimension",
            title: "Add angular dimension",
            description: "Angle dimension from a vertex and two topology anchors.",
            op: "add_angular_dimension",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "vertex": open_object("DrawingTopologyAnchorRefDto"),
                    "first": open_object("DrawingTopologyAnchorRefDto"),
                    "second": open_object("DrawingTopologyAnchorRefDto")
                }),
                &["view_id", "vertex", "first", "second"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_hole_note",
            title: "Add hole note",
            description: "Hole callout on a complete circular edge. Fills quantity/thread from matching Hole features when present.",
            op: "add_hole_note",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "feature": open_object("DrawingCircularRefDto"),
                    "position": position()
                }),
                &["view_id", "feature", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_chamfer_note",
            title: "Add chamfer note",
            description: "Chamfer callout between two topology anchors.",
            op: "add_chamfer_note",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "first": open_object("DrawingTopologyAnchorRefDto"),
                    "second": open_object("DrawingTopologyAnchorRefDto"),
                    "position": position(),
                    "length": { "type": "number" },
                    "angle_deg": { "type": "number" }
                }),
                &["view_id", "first", "second", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_center_mark",
            title: "Add center mark",
            description: "Center mark on a complete circular edge.",
            op: "add_center_mark",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "feature": open_object("DrawingCircularRefDto"),
                    "extension": { "type": "number" }
                }),
                &["view_id", "feature"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_center_line",
            title: "Add centerline",
            description: "Centerline between two distinct circular centers.",
            op: "add_center_line",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "first": open_object("DrawingCircularRefDto"),
                    "second": open_object("DrawingCircularRefDto"),
                    "extension": { "type": "number" }
                }),
                &["view_id", "first", "second"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_center_line_between_edges",
            title: "Add centerline between edges",
            description: "Centerline between two distinct straight edges.",
            op: "add_center_line_between_edges",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "first": open_object("DrawingLineRefDto"),
                    "second": open_object("DrawingLineRefDto"),
                    "extension": { "type": "number" }
                }),
                &["view_id", "first", "second"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_symmetry_axis",
            title: "Add automatic symmetry axis",
            description: "Automatic symmetry axis for a view (X, Y, or both).",
            op: "add_symmetry_axis",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "axis": { "type": "string", "enum": ["x", "y", "both"] },
                    "extension": { "type": "number" }
                }),
                &["view_id"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_bolt_circle",
            title: "Add bolt-circle centerline",
            description: "Bolt circle through at least three circular centers.",
            op: "add_bolt_circle",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "features": { "type": "array", "items": open_object("DrawingCircularRefDto"), "minItems": 3 },
                    "extension": { "type": "number" }
                }),
                &["view_id", "features"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_chain_dimension",
            title: "Add chain/baseline/continued dimension",
            description: "Dimension series across two or more topology anchors.",
            op: "add_chain_dimension",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "anchors": { "type": "array", "items": open_object("DrawingTopologyAnchorRefDto"), "minItems": 2 },
                    "layout": { "type": "string", "enum": ["chain", "baseline", "continued"] },
                    "mode": { "type": "string", "enum": ["aligned", "horizontal", "vertical"] }
                }),
                &["view_id", "anchors"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_ordinate_dimension",
            title: "Add ordinate dimension",
            description: "Ordinate dimension from an origin anchor to a target.",
            op: "add_ordinate_dimension",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "origin": open_object("DrawingTopologyAnchorRefDto"),
                    "target": open_object("DrawingTopologyAnchorRefDto"),
                    "axis": { "type": "string", "enum": ["x", "y", "both"] }
                }),
                &["view_id", "origin", "target"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_arc_length",
            title: "Add arc-length dimension",
            description: "Arc-length dimension on a circular edge between two anchors.",
            op: "add_arc_length_dimension",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "feature": open_object("DrawingCircularRefDto"),
                    "first": open_object("DrawingTopologyAnchorRefDto"),
                    "second": open_object("DrawingTopologyAnchorRefDto")
                }),
                &["view_id", "feature", "first", "second"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_jogged_radius",
            title: "Add jogged radius",
            description: "Jogged radius dimension for a circular feature.",
            op: "add_jogged_radius",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "feature": open_object("DrawingCircularRefDto"),
                    "position": position()
                }),
                &["view_id", "feature", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_datum",
            title: "Add datum feature",
            description: "Datum feature symbol attached to topology.",
            op: "add_datum",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "attachment": open_object("DrawingAttachmentRefDto"),
                    "position": position(),
                    "label": { "type": "string" }
                }),
                &["view_id", "attachment", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_gdt",
            title: "Add GD&T frame",
            description: "Feature-control frame attached to topology.",
            op: "add_gdt",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "attachment": open_object("DrawingAttachmentRefDto"),
                    "position": position(),
                    "characteristic": { "type": "string" },
                    "tolerance": { "type": "number" }
                }),
                &["view_id", "attachment", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_surface_texture",
            title: "Add surface texture",
            description: "Surface-texture symbol attached to topology.",
            op: "add_surface_texture",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "attachment": open_object("DrawingAttachmentRefDto"),
                    "position": position(),
                    "roughness_ra": { "type": "number" }
                }),
                &["view_id", "attachment", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_edge_requirement",
            title: "Add edge requirement",
            description: "Edge requirement symbol on a straight edge.",
            op: "add_edge_requirement",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "attachment": open_object("DrawingLineRefDto"),
                    "position": position(),
                    "upper_deviation": { "type": "number" },
                    "lower_deviation": { "type": "number" }
                }),
                &["view_id", "attachment", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_weld",
            title: "Add weld symbol",
            description: "Weld symbol on a straight edge.",
            op: "add_weld",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "attachment": open_object("DrawingLineRefDto"),
                    "position": position(),
                    "weld_type": { "type": "string" },
                    "side": { "type": "string", "enum": ["arrow", "other", "both"] },
                    "size": { "type": "number" }
                }),
                &["view_id", "attachment", "position"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_balloon",
            title: "Add item balloon",
            description: "BOM balloon attached to topology. bom_item_id must exist on the active sheet.",
            op: "add_balloon",
            schema: object(
                json!({
                    "view_id": entity_id(),
                    "attachment": open_object("DrawingAttachmentRefDto"),
                    "position": position(),
                    "bom_item_id": entity_id()
                }),
                &["view_id", "attachment", "position", "bom_item_id"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_revision_cloud",
            title: "Add revision cloud",
            description: "Revision cloud from three or more paper points.",
            op: "add_revision_cloud",
            schema: object(
                json!({
                    "points": { "type": "array", "items": position(), "minItems": 3 },
                    "revision": { "type": "string" }
                }),
                &["points"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_add_annotation",
            title: "Add drawing annotation",
            description: "Generic tagged annotation (kind + fields). Prefer the named cad_drawing_add_* tools when you know the kind.",
            op: "add_annotation",
            schema: object(
                json!({ "annotation": open_object("DrawingAnnotationDto with kind tag; id is assigned") }),
                &["annotation"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_update_annotation",
            title: "Update drawing annotation",
            description: "Patch fields on an existing annotation by id.",
            op: "update_annotation",
            schema: object(
                json!({
                    "annotation_id": entity_id(),
                    "patch": open_object("Partial annotation fields")
                }),
                &["annotation_id", "patch"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_delete_annotation",
            title: "Delete drawing annotation",
            description: "Remove an annotation by id.",
            op: "delete_annotation",
            schema: object(json!({ "annotation_id": entity_id() }), &["annotation_id"]),
        },
        DrawingTool {
            name: "cad_drawing_save_template",
            title: "Save drawing template",
            description: "Save the active sheet's standards, title defaults, and style as a project-local template.",
            op: "save_template",
            schema: object(json!({ "name": { "type": "string", "minLength": 1 } }), &["name"]),
        },
        DrawingTool {
            name: "cad_drawing_apply_template",
            title: "Apply drawing template",
            description: "Apply a project template to the active sheet without changing drawing number/revision identity.",
            op: "apply_template",
            schema: object(json!({ "template_id": entity_id() }), &["template_id"]),
        },
        DrawingTool {
            name: "cad_drawing_delete_template",
            title: "Delete drawing template",
            description: "Remove a project-local drawing template.",
            op: "delete_template",
            schema: object(json!({ "template_id": entity_id() }), &["template_id"]),
        },
        DrawingTool {
            name: "cad_drawing_add_revision",
            title: "Add drawing revision",
            description: "Add a revision row. status=released marks the sheet released.",
            op: "add_revision",
            schema: object(
                json!({
                    "revision": open_object("DrawingRevisionDraft: revision, description, date, author, status")
                }),
                &["revision"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_update_revision",
            title: "Update drawing revision",
            description: "Patch a draft revision. Released revisions are immutable.",
            op: "update_revision",
            schema: object(
                json!({
                    "revision_id": entity_id(),
                    "patch": open_object("Partial revision fields")
                }),
                &["revision_id", "patch"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_delete_revision",
            title: "Delete drawing revision",
            description: "Delete a draft revision. Released revisions cannot be deleted.",
            op: "delete_revision",
            schema: object(json!({ "revision_id": entity_id() }), &["revision_id"]),
        },
        DrawingTool {
            name: "cad_drawing_add_bom_item",
            title: "Add BOM item",
            description: "Add a bill-of-materials row on the active sheet. Optional body_id links a solid body.",
            op: "add_bom_item",
            schema: object(
                json!({ "item": open_object("DrawingBomItemDraft") }),
                &[],
            ),
        },
        DrawingTool {
            name: "cad_drawing_update_bom_item",
            title: "Update BOM item",
            description: "Patch a BOM row by id.",
            op: "update_bom_item",
            schema: object(
                json!({
                    "item_id": entity_id(),
                    "patch": open_object("Partial BOM fields")
                }),
                &["item_id", "patch"],
            ),
        },
        DrawingTool {
            name: "cad_drawing_delete_bom_item",
            title: "Delete BOM item",
            description: "Remove a BOM row and any balloons pointing at it.",
            op: "delete_bom_item",
            schema: object(json!({ "item_id": entity_id() }), &["item_id"]),
        },
    ]
}

pub const DRAWING_COMMAND_TOOL_COUNT: usize = 45;
pub const DRAWING_PROJECTION_TOOL_COUNT: usize = 2;
pub const DRAWING_HISTORY_TOOL_COUNT: usize = 2;
pub const DRAWING_EXPORT_TOOL_COUNT: usize = 3;
pub const DRAWING_GENERIC_TOOL_COUNT: usize = 1;
pub const DRAWING_PACK_TOOL_COUNT: usize = 2
    + DRAWING_COMMAND_TOOL_COUNT
    + DRAWING_PROJECTION_TOOL_COUNT
    + DRAWING_HISTORY_TOOL_COUNT
    + DRAWING_EXPORT_TOOL_COUNT
    + DRAWING_GENERIC_TOOL_COUNT;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn drawing_command_catalog_is_complete_and_unique() {
        let tools = drawing_command_tools();
        assert_eq!(tools.len(), DRAWING_COMMAND_TOOL_COUNT);
        assert_eq!(DRAWING_PACK_TOOL_COUNT, 55);
        let mut names = HashSet::new();
        let mut ops = HashSet::new();
        for tool in &tools {
            assert!(
                tool.name.starts_with("cad_drawing_"),
                "unexpected drawing tool name {}",
                tool.name
            );
            assert!(!tool.op.is_empty(), "{} is missing op", tool.name);
            assert!(names.insert(tool.name), "duplicate tool {}", tool.name);
            assert!(
                ops.insert(tool.op),
                "duplicate op {} on {}",
                tool.op,
                tool.name
            );
        }
    }
}
