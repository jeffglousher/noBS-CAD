//! Completeness matrix: every real product feature MCP can drive.
//!
//! Disabled ribbon placeholders, collaboration comments, and pointer-only
//! placement gestures are not product features. Pointer tools have coordinate
//! or topology command equivalents.

pub struct HostMethod {
    pub method: &'static str,
    pub tool: &'static str,
}

pub struct RibbonFeature {
    pub action: &'static str,
    pub payload: Option<&'static str>,
    pub tool: &'static str,
}

pub struct FileFeature {
    pub id: &'static str,
    pub tool: &'static str,
}

/// Engine `host::handle` methods that agents call, with the MCP tool that
/// owns them. `solid_commit` and `solid_prepare_*` stay internal to replay.
pub const HOST_METHODS: &[HostMethod] = &[
    HostMethod {
        method: "document",
        tool: "cad_document",
    },
    HostMethod {
        method: "document_set_name",
        tool: "cad_set_document_name",
    },
    HostMethod {
        method: "project_export_model",
        tool: "cad_project_model",
    },
    HostMethod {
        method: "project_prepare_new",
        tool: "cad_new_project",
    },
    HostMethod {
        method: "project_prepare_load",
        tool: "cad_load_project_model",
    },
    HostMethod {
        method: "begin_sketch",
        tool: "sketch_begin",
    },
    HostMethod {
        method: "end_sketch",
        tool: "sketch_finish",
    },
    HostMethod {
        method: "finished_sketches",
        tool: "sketch_finished",
    },
    HostMethod {
        method: "edit_sketch",
        tool: "sketch_edit",
    },
    HostMethod {
        method: "active_sketch",
        tool: "sketch_active",
    },
    HostMethod {
        method: "profile_catalog",
        tool: "sketch_profiles",
    },
    HostMethod {
        method: "solid_scene",
        tool: "solid_scene",
    },
    HostMethod {
        method: "body_appearances",
        tool: "body_appearances",
    },
    HostMethod {
        method: "project_visibility",
        tool: "cad_project_visibility",
    },
    HostMethod {
        method: "project_set_visibility",
        tool: "cad_set_project_visibility",
    },
    HostMethod {
        method: "drawing_document",
        tool: "cad_drawing_document",
    },
    HostMethod {
        method: "drawing_set_document",
        tool: "cad_set_drawing_document",
    },
    HostMethod {
        method: "drawing_command",
        tool: "cad_drawing_create_sheet",
    },
    HostMethod {
        method: "drawing_undo",
        tool: "cad_drawing_undo",
    },
    HostMethod {
        method: "drawing_redo",
        tool: "cad_drawing_redo",
    },
    HostMethod {
        method: "set_body_appearance",
        tool: "set_body_appearance",
    },
    HostMethod {
        method: "extrude_definitions",
        tool: "solid_extrude_definitions",
    },
    HostMethod {
        method: "revolve_definitions",
        tool: "solid_revolve_definitions",
    },
    HostMethod {
        method: "sweep_definitions",
        tool: "solid_sweep_definitions",
    },
    HostMethod {
        method: "loft_definitions",
        tool: "solid_loft_definitions",
    },
    HostMethod {
        method: "rib_definitions",
        tool: "solid_rib_definitions",
    },
    HostMethod {
        method: "fillet_definitions",
        tool: "solid_fillet_definitions",
    },
    HostMethod {
        method: "chamfer_definitions",
        tool: "solid_chamfer_definitions",
    },
    HostMethod {
        method: "hole_definitions",
        tool: "solid_hole_definitions",
    },
    HostMethod {
        method: "datum_plane_definitions",
        tool: "construction_plane_definitions",
    },
    HostMethod {
        method: "body_feature_definitions",
        tool: "solid_body_feature_definitions",
    },
    HostMethod {
        method: "datum_plane_create",
        tool: "construction_plane_offset",
    },
    HostMethod {
        method: "datum_plane_edit",
        tool: "construction_plane_edit_offset",
    },
    HostMethod {
        method: "solid_prepare_body_feature",
        tool: "solid_shell",
    },
    HostMethod {
        method: "solid_prepare_edit_body_feature",
        tool: "solid_edit_shell",
    },
    HostMethod {
        method: "solid_prepare_extrude",
        tool: "solid_extrude",
    },
    HostMethod {
        method: "solid_prepare_edit_extrude",
        tool: "solid_edit_extrude",
    },
    HostMethod {
        method: "solid_prepare_revolve",
        tool: "solid_revolve",
    },
    HostMethod {
        method: "solid_prepare_edit_revolve",
        tool: "solid_edit_revolve",
    },
    HostMethod {
        method: "solid_prepare_sweep",
        tool: "solid_sweep",
    },
    HostMethod {
        method: "solid_prepare_edit_sweep",
        tool: "solid_edit_sweep",
    },
    HostMethod {
        method: "solid_prepare_loft",
        tool: "solid_loft",
    },
    HostMethod {
        method: "solid_prepare_edit_loft",
        tool: "solid_edit_loft",
    },
    HostMethod {
        method: "solid_prepare_rib",
        tool: "solid_rib",
    },
    HostMethod {
        method: "solid_prepare_edit_rib",
        tool: "solid_edit_rib",
    },
    HostMethod {
        method: "solid_prepare_fillet",
        tool: "solid_fillet",
    },
    HostMethod {
        method: "solid_prepare_edit_fillet",
        tool: "solid_edit_fillet",
    },
    HostMethod {
        method: "solid_prepare_chamfer",
        tool: "solid_chamfer",
    },
    HostMethod {
        method: "solid_prepare_edit_chamfer",
        tool: "solid_edit_chamfer",
    },
    HostMethod {
        method: "solid_prepare_hole",
        tool: "solid_hole",
    },
    HostMethod {
        method: "solid_prepare_edit_hole",
        tool: "solid_edit_hole",
    },
    HostMethod {
        method: "solid_prepare_recompute",
        tool: "solid_recompute",
    },
    HostMethod {
        method: "solid_prepare_set_rollback",
        tool: "solid_set_rollback",
    },
    HostMethod {
        method: "solid_prepare_delete_feature",
        tool: "solid_delete_feature",
    },
    HostMethod {
        method: "solid_prepare_reorder_feature",
        tool: "solid_reorder_feature",
    },
    HostMethod {
        method: "preview_segment",
        tool: "sketch_preview_line",
    },
    HostMethod {
        method: "eval_expression",
        tool: "sketch_eval_expression",
    },
    HostMethod {
        method: "add_line",
        tool: "sketch_add_line",
    },
    HostMethod {
        method: "preview_segment_locked",
        tool: "sketch_preview_line_locked",
    },
    HostMethod {
        method: "add_line_locked",
        tool: "sketch_add_line_locked",
    },
    HostMethod {
        method: "add_point",
        tool: "sketch_add_point",
    },
    HostMethod {
        method: "add_line_midpoint",
        tool: "sketch_add_midpoint_line",
    },
    HostMethod {
        method: "add_rectangle",
        tool: "sketch_add_rectangle",
    },
    HostMethod {
        method: "add_rectangle_locked",
        tool: "sketch_add_rectangle_locked",
    },
    HostMethod {
        method: "add_circle",
        tool: "sketch_add_circle",
    },
    HostMethod {
        method: "add_circle_locked",
        tool: "sketch_add_circle_locked",
    },
    HostMethod {
        method: "add_slot",
        tool: "sketch_add_slot",
    },
    HostMethod {
        method: "add_spline",
        tool: "sketch_add_spline",
    },
    HostMethod {
        method: "add_arc_3pt",
        tool: "sketch_add_arc_3pt",
    },
    HostMethod {
        method: "add_arc_center",
        tool: "sketch_add_arc_center",
    },
    HostMethod {
        method: "add_constraint",
        tool: "sketch_add_constraint",
    },
    HostMethod {
        method: "add_constraints",
        tool: "sketch_add_constraints",
    },
    HostMethod {
        method: "add_dimension",
        tool: "sketch_add_dimension",
    },
    HostMethod {
        method: "edit_dimension",
        tool: "sketch_edit_dimension",
    },
    HostMethod {
        method: "move_dimension",
        tool: "sketch_move_dimension",
    },
    HostMethod {
        method: "delete_dimension",
        tool: "sketch_delete_dimension",
    },
    HostMethod {
        method: "set_dimension_style",
        tool: "sketch_set_dimension_style",
    },
    HostMethod {
        method: "fillet_preview",
        tool: "sketch_preview_fillet",
    },
    HostMethod {
        method: "fillet_lines",
        tool: "sketch_fillet",
    },
    HostMethod {
        method: "chamfer_lines",
        tool: "sketch_chamfer",
    },
    HostMethod {
        method: "offset_preview",
        tool: "sketch_preview_offset",
    },
    HostMethod {
        method: "offset_curve",
        tool: "sketch_offset",
    },
    HostMethod {
        method: "trim_preview",
        tool: "sketch_preview_trim",
    },
    HostMethod {
        method: "trim_entity",
        tool: "sketch_trim",
    },
    HostMethod {
        method: "extend_entity",
        tool: "sketch_extend",
    },
    HostMethod {
        method: "break_curve",
        tool: "sketch_break",
    },
    HostMethod {
        method: "mirror_entities",
        tool: "sketch_mirror",
    },
    HostMethod {
        method: "rectangular_pattern",
        tool: "sketch_rectangular_pattern",
    },
    HostMethod {
        method: "circular_pattern",
        tool: "sketch_circular_pattern",
    },
    HostMethod {
        method: "move_copy_entities",
        tool: "sketch_move_copy",
    },
    HostMethod {
        method: "scale_entities",
        tool: "sketch_scale",
    },
    HostMethod {
        method: "polygon_create",
        tool: "sketch_polygon",
    },
    HostMethod {
        method: "toggle_fix",
        tool: "sketch_toggle_fix",
    },
    HostMethod {
        method: "toggle_fix_entities",
        tool: "sketch_toggle_fix",
    },
    HostMethod {
        method: "move_point",
        tool: "sketch_move_point",
    },
    HostMethod {
        method: "delete_entity",
        tool: "sketch_delete_entities",
    },
    HostMethod {
        method: "delete_entities",
        tool: "sketch_delete_entities",
    },
    HostMethod {
        method: "undo",
        tool: "sketch_undo",
    },
    HostMethod {
        method: "redo",
        tool: "sketch_redo",
    },
    HostMethod {
        method: "set_grid_snap",
        tool: "sketch_set_grid_snap",
    },
    HostMethod {
        method: "set_grid_step",
        tool: "sketch_set_grid_step",
    },
    HostMethod {
        method: "assembly_document",
        tool: "assembly_document",
    },
    HostMethod {
        method: "assembly_set_document",
        tool: "assembly_set_document",
    },
    HostMethod {
        method: "assembly_solution",
        tool: "assembly_solution",
    },
    HostMethod {
        method: "assembly_create_component",
        tool: "assembly_create_component",
    },
    HostMethod {
        method: "assembly_update_component",
        tool: "assembly_update_component",
    },
    HostMethod {
        method: "assembly_create_occurrence",
        tool: "assembly_create_occurrence",
    },
    HostMethod {
        method: "assembly_update_occurrence",
        tool: "assembly_update_occurrence",
    },
    HostMethod {
        method: "assembly_duplicate_occurrence",
        tool: "assembly_duplicate_occurrence",
    },
    HostMethod {
        method: "assembly_set_occurrence_grounded",
        tool: "assembly_set_occurrence_grounded",
    },
    HostMethod {
        method: "assembly_set_occurrence_pose",
        tool: "assembly_set_occurrence_pose",
    },
    HostMethod {
        method: "assembly_preview_joint",
        tool: "assembly_preview_joint",
    },
    HostMethod {
        method: "assembly_create_joint",
        tool: "assembly_create_joint",
    },
    HostMethod {
        method: "assembly_update_joint",
        tool: "assembly_update_joint",
    },
    HostMethod {
        method: "assembly_preview_joint_update",
        tool: "assembly_preview_joint_update",
    },
    HostMethod {
        method: "assembly_delete_joint",
        tool: "assembly_delete_joint",
    },
    HostMethod {
        method: "assembly_set_joint_enabled",
        tool: "assembly_set_joint_enabled",
    },
    HostMethod {
        method: "assembly_set_joint_motion",
        tool: "assembly_set_joint_motion",
    },
    HostMethod {
        method: "assembly_preview_joint_motion",
        tool: "assembly_preview_joint_motion",
    },
    HostMethod {
        method: "assembly_set_joint_coordinates",
        tool: "assembly_set_joint_coordinates",
    },
    HostMethod {
        method: "assembly_preview_joint_coordinates",
        tool: "assembly_preview_joint_coordinates",
    },
    HostMethod {
        method: "assembly_preview_mechanism_drag",
        tool: "assembly_preview_mechanism_drag",
    },
    HostMethod {
        method: "assembly_apply_joint_motions",
        tool: "assembly_apply_joint_motions",
    },
    HostMethod {
        method: "assembly_create_position",
        tool: "assembly_create_position",
    },
    HostMethod {
        method: "assembly_update_position",
        tool: "assembly_update_position",
    },
    HostMethod {
        method: "assembly_delete_position",
        tool: "assembly_delete_position",
    },
    HostMethod {
        method: "assembly_apply_position",
        tool: "assembly_apply_position",
    },
    HostMethod {
        method: "assembly_create_motion_study",
        tool: "assembly_create_motion_study",
    },
    HostMethod {
        method: "assembly_update_motion_study",
        tool: "assembly_update_motion_study",
    },
    HostMethod {
        method: "assembly_delete_motion_study",
        tool: "assembly_delete_motion_study",
    },
    HostMethod {
        method: "assembly_sample_motion_study",
        tool: "assembly_sample_motion_study",
    },
    HostMethod {
        method: "assembly_export_motion_path_csv",
        tool: "assembly_export_motion_path_csv",
    },
    HostMethod {
        method: "assembly_create_contact_set",
        tool: "assembly_create_contact_set",
    },
    HostMethod {
        method: "assembly_update_contact_set",
        tool: "assembly_update_contact_set",
    },
    HostMethod {
        method: "assembly_delete_contact_set",
        tool: "assembly_delete_contact_set",
    },
    HostMethod {
        method: "assembly_interference_check",
        tool: "assembly_interference_check",
    },
    HostMethod {
        method: "assembly_evaluate_motion_study",
        tool: "assembly_evaluate_motion_study",
    },
    HostMethod {
        method: "assembly_swept_collision_check",
        tool: "assembly_swept_collision_check",
    },
    HostMethod {
        method: "assembly_set_grounded_body",
        tool: "assembly_set_grounded_body",
    },
];

/// Enabled ribbon actions (and flyout payloads) that ship in the product.
pub const RIBBON_FEATURES: &[RibbonFeature] = &[
    RibbonFeature {
        action: "enterSketch",
        payload: None,
        tool: "sketch_begin",
    },
    RibbonFeature {
        action: "exitSketch",
        payload: None,
        tool: "sketch_finish",
    },
    RibbonFeature {
        action: "extrude",
        payload: None,
        tool: "solid_extrude",
    },
    RibbonFeature {
        action: "revolve",
        payload: None,
        tool: "solid_revolve",
    },
    RibbonFeature {
        action: "sweep",
        payload: None,
        tool: "solid_sweep",
    },
    RibbonFeature {
        action: "loft",
        payload: None,
        tool: "solid_loft",
    },
    RibbonFeature {
        action: "rib",
        payload: None,
        tool: "solid_rib",
    },
    RibbonFeature {
        action: "solidFillet",
        payload: None,
        tool: "solid_fillet",
    },
    RibbonFeature {
        action: "solidChamfer",
        payload: None,
        tool: "solid_chamfer",
    },
    RibbonFeature {
        action: "hole",
        payload: None,
        tool: "solid_hole",
    },
    RibbonFeature {
        action: "constructionPlane",
        payload: Some("offset"),
        tool: "construction_plane_offset",
    },
    RibbonFeature {
        action: "constructionPlane",
        payload: Some("midplane"),
        tool: "construction_plane_midplane",
    },
    RibbonFeature {
        action: "constructionPlane",
        payload: Some("at_angle"),
        tool: "construction_plane_at_angle",
    },
    RibbonFeature {
        action: "bodyFeature",
        payload: Some("external_thread"),
        tool: "solid_external_thread",
    },
    RibbonFeature {
        action: "bodyFeature",
        payload: Some("shell"),
        tool: "solid_shell",
    },
    RibbonFeature {
        action: "bodyFeature",
        payload: Some("mirror"),
        tool: "solid_mirror",
    },
    RibbonFeature {
        action: "bodyFeature",
        payload: Some("rectangular_pattern"),
        tool: "solid_rectangular_pattern",
    },
    RibbonFeature {
        action: "bodyFeature",
        payload: Some("circular_pattern"),
        tool: "solid_circular_pattern",
    },
    RibbonFeature {
        action: "bodyFeature",
        payload: Some("combine"),
        tool: "solid_combine",
    },
    RibbonFeature {
        action: "bodyFeature",
        payload: Some("split_body"),
        tool: "solid_split_body",
    },
    RibbonFeature {
        action: "bodyFeature",
        payload: Some("move_copy"),
        tool: "solid_move_copy",
    },
    RibbonFeature {
        action: "assemblyWorkspace",
        payload: None,
        tool: "cad_set_workspace",
    },
    RibbonFeature {
        action: "joint",
        payload: None,
        tool: "assembly_create_joint",
    },
    RibbonFeature {
        action: "sketchPattern",
        payload: Some("rectangular"),
        tool: "sketch_rectangular_pattern",
    },
    RibbonFeature {
        action: "sketchPattern",
        payload: Some("circular"),
        tool: "sketch_circular_pattern",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("line"),
        tool: "sketch_add_line",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("arc3pt"),
        tool: "sketch_add_arc_3pt",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("arcCenter"),
        tool: "sketch_add_arc_center",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("splineFit"),
        tool: "sketch_add_spline",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("rect2pt"),
        tool: "sketch_add_rectangle",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("rectCenter"),
        tool: "sketch_add_rectangle",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("circleCenter"),
        tool: "sketch_add_circle",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("circle2pt"),
        tool: "sketch_add_circle",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("polygon:inscribed"),
        tool: "sketch_polygon",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("polygon:circumscribed"),
        tool: "sketch_polygon",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("slot:centerToCenter"),
        tool: "sketch_add_slot",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("slot:overall"),
        tool: "sketch_add_slot",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("slot:centerPoint"),
        tool: "sketch_add_slot",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("midpointLine"),
        tool: "sketch_add_midpoint_line",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("point"),
        tool: "sketch_add_point",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("moveCopy"),
        tool: "sketch_move_copy",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("scale"),
        tool: "sketch_scale",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("offset"),
        tool: "sketch_offset",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("trim"),
        tool: "sketch_trim",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("extend"),
        tool: "sketch_extend",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("break"),
        tool: "sketch_break",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("fillet"),
        tool: "sketch_fillet",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("chamfer"),
        tool: "sketch_chamfer",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("mirror"),
        tool: "sketch_mirror",
    },
    RibbonFeature {
        action: "sketchTool",
        payload: Some("dimension"),
        tool: "sketch_add_dimension",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("coincident"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("midpoint"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("collinear"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("hv"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("parallel"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("perpendicular"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("tangent"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("concentric"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("equal"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("symmetry"),
        tool: "sketch_add_constraint",
    },
    RibbonFeature {
        action: "applyConstraint",
        payload: Some("fixUnfix"),
        tool: "sketch_toggle_fix",
    },
    RibbonFeature {
        action: "drawingWorkspace",
        payload: None,
        tool: "cad_set_workspace",
    },
    RibbonFeature {
        action: "modelWorkspace",
        payload: None,
        tool: "cad_set_workspace",
    },
    RibbonFeature {
        action: "drawingNewSheet",
        payload: None,
        tool: "cad_drawing_create_sheet",
    },
    RibbonFeature {
        action: "drawingAutoLayout",
        payload: None,
        tool: "cad_drawing_auto_layout",
    },
    RibbonFeature {
        action: "drawingAddView",
        payload: Some("front"),
        tool: "cad_drawing_add_view",
    },
    RibbonFeature {
        action: "drawingAddView",
        payload: Some("top"),
        tool: "cad_drawing_add_view",
    },
    RibbonFeature {
        action: "drawingAddView",
        payload: Some("left"),
        tool: "cad_drawing_add_view",
    },
    RibbonFeature {
        action: "drawingAddView",
        payload: Some("right"),
        tool: "cad_drawing_add_view",
    },
    RibbonFeature {
        action: "drawingAddView",
        payload: Some("isometric"),
        tool: "cad_drawing_add_view",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("section_view"),
        tool: "cad_drawing_add_derived_view",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("detail_view"),
        tool: "cad_drawing_add_derived_view",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("auxiliary_view"),
        tool: "cad_drawing_add_derived_view",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("broken_view"),
        tool: "cad_drawing_add_derived_view",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("removed_section"),
        tool: "cad_drawing_add_derived_view",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("dimension"),
        tool: "cad_drawing_add_linear_dimension",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("chain_dimension"),
        tool: "cad_drawing_add_chain_dimension",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("baseline_dimension"),
        tool: "cad_drawing_add_chain_dimension",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("continued_dimension"),
        tool: "cad_drawing_add_chain_dimension",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("ordinate_dimension"),
        tool: "cad_drawing_add_ordinate_dimension",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("diameter"),
        tool: "cad_drawing_add_radial_dimension",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("radius"),
        tool: "cad_drawing_add_radial_dimension",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("arc_length"),
        tool: "cad_drawing_add_arc_length",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("jogged_radius"),
        tool: "cad_drawing_add_jogged_radius",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("angle"),
        tool: "cad_drawing_add_angular_dimension",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("center_mark"),
        tool: "cad_drawing_add_center_mark",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("center_line"),
        tool: "cad_drawing_add_center_line",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("symmetry_axis"),
        tool: "cad_drawing_add_symmetry_axis",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("bolt_circle"),
        tool: "cad_drawing_add_bolt_circle",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("hole_note"),
        tool: "cad_drawing_add_hole_note",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("chamfer_note"),
        tool: "cad_drawing_add_chamfer_note",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("note"),
        tool: "cad_drawing_add_note",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("datum"),
        tool: "cad_drawing_add_datum",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("gdt"),
        tool: "cad_drawing_add_gdt",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("surface_texture"),
        tool: "cad_drawing_add_surface_texture",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("edge_requirement"),
        tool: "cad_drawing_add_edge_requirement",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("weld"),
        tool: "cad_drawing_add_weld",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("balloon"),
        tool: "cad_drawing_add_balloon",
    },
    RibbonFeature {
        action: "drawingTool",
        payload: Some("revision_cloud"),
        tool: "cad_drawing_add_revision_cloud",
    },
    RibbonFeature {
        action: "drawingExportDxf",
        payload: None,
        tool: "cad_drawing_export_dxf",
    },
    RibbonFeature {
        action: "drawingExportProfileDxf",
        payload: None,
        tool: "cad_drawing_export_profile_dxf",
    },
    RibbonFeature {
        action: "drawingPrint",
        payload: None,
        tool: "cad_drawing_export_svg",
    },
];

pub const FILE_FEATURES: &[FileFeature] = &[
    FileFeature {
        id: "new",
        tool: "cad_new_project",
    },
    FileFeature {
        id: "open",
        tool: "cad_load_project_model",
    },
    FileFeature {
        id: "save",
        tool: "cad_project_model",
    },
    FileFeature {
        id: "saveAs",
        tool: "cad_project_model",
    },
    FileFeature {
        id: "rename",
        tool: "cad_set_document_name",
    },
    FileFeature {
        id: "importStep",
        tool: "solid_import_step",
    },
    FileFeature {
        id: "exportStep",
        tool: "solid_export_step",
    },
    FileFeature {
        id: "export3mf",
        tool: "solid_export_3mf",
    },
    FileFeature {
        id: "exportStl",
        tool: "solid_export_stl",
    },
    FileFeature {
        id: "exportDrawingDxf",
        tool: "cad_drawing_export_dxf",
    },
    FileFeature {
        id: "exportManufacturingProfileDxf",
        tool: "cad_drawing_export_profile_dxf",
    },
];

pub const BROWSER_FEATURES: &[FileFeature] = &[
    FileFeature {
        id: "hide_show",
        tool: "cad_set_project_visibility",
    },
    FileFeature {
        id: "edit_sketch",
        tool: "sketch_edit",
    },
    FileFeature {
        id: "delete_feature",
        tool: "solid_delete_feature",
    },
    FileFeature {
        id: "appearance",
        tool: "set_body_appearance",
    },
    FileFeature {
        id: "rename_document",
        tool: "cad_set_document_name",
    },
];

pub const TIMELINE_FEATURES: &[FileFeature] = &[
    FileFeature {
        id: "rollback",
        tool: "solid_set_rollback",
    },
    FileFeature {
        id: "reorder",
        tool: "solid_reorder_feature",
    },
    FileFeature {
        id: "delete",
        tool: "solid_delete_feature",
    },
    FileFeature {
        id: "edit_feature",
        tool: "solid_edit_extrude",
    },
];

pub const EDIT_FEATURES: &[FileFeature] = &[
    FileFeature {
        id: "undo",
        tool: "cad_undo",
    },
    FileFeature {
        id: "redo",
        tool: "cad_redo",
    },
];

pub const WORKSPACES: &[&str] = &["solid", "drawing", "assembly"];

pub const INTERNAL_HOST_METHODS: &[&str] = &["solid_commit"];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn host_methods_are_unique_and_mapped() {
        let mut methods = HashSet::new();
        for item in HOST_METHODS {
            assert!(
                methods.insert(item.method),
                "duplicate host method {}",
                item.method
            );
            assert!(!item.tool.is_empty());
        }
        for required in [
            "drawing_undo",
            "drawing_redo",
            "drawing_command",
            "solid_prepare_body_feature",
        ] {
            assert!(
                methods.contains(required),
                "completeness matrix missing host method {required}"
            );
        }
    }

    #[test]
    fn ribbon_and_file_features_cover_output_and_import() {
        let tools: HashSet<_> = RIBBON_FEATURES
            .iter()
            .map(|item| item.tool)
            .chain(FILE_FEATURES.iter().map(|item| item.tool))
            .chain(BROWSER_FEATURES.iter().map(|item| item.tool))
            .chain(TIMELINE_FEATURES.iter().map(|item| item.tool))
            .chain(EDIT_FEATURES.iter().map(|item| item.tool))
            .collect();
        for required in [
            "solid_import_step",
            "cad_drawing_export_dxf",
            "cad_drawing_export_svg",
            "cad_drawing_export_profile_dxf",
            "cad_set_workspace",
            "cad_drawing_add_derived_view",
            "cad_set_document_name",
            "cad_undo",
            "cad_redo",
        ] {
            assert!(tools.contains(required), "missing product tool {required}");
        }
        assert_eq!(WORKSPACES, &["solid", "drawing", "assembly"]);
    }

    #[test]
    fn host_source_methods_are_in_the_matrix() {
        let mapped: HashSet<_> = HOST_METHODS.iter().map(|item| item.method).collect();
        for method in super::host_methods_in_source() {
            if INTERNAL_HOST_METHODS.contains(&method.as_str()) {
                continue;
            }
            assert!(
                mapped.contains(method.as_str()),
                "host method {method} is not in HOST_METHODS"
            );
        }
    }

    #[test]
    fn drawing_command_source_ops_have_named_tools() {
        let ops: HashSet<_> = crate::drawing_tools::drawing_command_tools()
            .into_iter()
            .map(|tool| tool.op.to_string())
            .collect();
        for op in super::drawing_ops_in_source() {
            assert!(
                ops.contains(&op),
                "DrawingCommand {op} has no named MCP tool"
            );
        }
        assert_eq!(ops.len(), crate::drawing_tools::DRAWING_COMMAND_TOOL_COUNT);
    }
}

fn host_methods_in_source() -> Vec<String> {
    include_str!("../../crates/sketch/src/host.rs")
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix('"')?;
            let end = rest.find('"')?;
            let name = &rest[..end];
            if rest[end + 1..].trim_start().starts_with("=>") {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn drawing_ops_in_source() -> Vec<String> {
    let src = include_str!("../../crates/sketch/src/drawing_ops.rs");
    let start = src
        .find("pub enum DrawingCommand")
        .expect("DrawingCommand enum");
    let body = &src[start..];
    let end = body
        .find("pub struct DrawingRevisionDraft")
        .expect("enum terminator");
    body[..end]
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let name = trimmed
                .strip_suffix(" {},")
                .or_else(|| trimmed.strip_suffix(" {"))?;
            if name
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase())
                && !name.contains(' ')
            {
                Some(pascal_to_snake(name))
            } else {
                None
            }
        })
        .collect()
}

fn pascal_to_snake(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::new();
    for (index, ch) in chars.iter().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                let prev_lower = chars[index - 1].is_ascii_lowercase();
                let next_lower = chars
                    .get(index + 1)
                    .is_some_and(|next| next.is_ascii_lowercase());
                if prev_lower || next_lower {
                    out.push('_');
                }
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(*ch);
        }
    }
    out
}
