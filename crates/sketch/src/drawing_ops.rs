//! Authoritative drawing-document commands.
//!
//! The UI still mutates sheets in TypeScript for pointer-driven placement.
//! Headless MCP (and any host that calls `drawing_command`) uses this module
//! so agents get first-class sheet/view/annotation ops instead of inventing
//! a `DrawingDocumentDto`. Generated HLR curves stay out of the document.

use nbcad_core::BodyId;
use nbcad_solid::{HoleDefinitionDto, HoleExtent, HoleStyle, HoleThreadHand, SolidSceneDto};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::drawing::{
    drawing_sheet_size, DrawingAnnotationDto, DrawingAttachmentRefDto, DrawingBomItemDto,
    DrawingChainDimensionLayout, DrawingCircularRefDto, DrawingDimensionPresentationDto,
    DrawingDocumentDto, DrawingGdtCharacteristic, DrawingHoleStyle, DrawingLineDimensionMode,
    DrawingLineRefDto, DrawingLinearDimensionMode, DrawingMaterialCondition, DrawingOrdinateAxis,
    DrawingProjectionMethod, DrawingRadialDimensionMode, DrawingReleaseDto, DrawingReleaseStatus,
    DrawingRevisionDto, DrawingSheetDto, DrawingSheetFormat, DrawingSheetOrientation,
    DrawingSheetStyleDto, DrawingStandard, DrawingSurfaceLay, DrawingTemplateDto,
    DrawingTitleBlockDto, DrawingToleranceNoteDto, DrawingTolerancePreset,
    DrawingTopologyAnchorRefDto, DrawingViewAlignment, DrawingViewDerivationDto, DrawingViewDto,
    DrawingViewKind, DrawingWeldContour, DrawingWeldSide, DrawingWeldType,
};

const STANDARD_SCALES: [f64; 10] = [10.0, 5.0, 2.0, 1.0, 0.5, 0.2, 0.1, 0.05, 0.02, 0.01];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum DrawingCommand {
    CreateSheet {
        #[serde(default)]
        standard: DrawingStandard,
        #[serde(default)]
        format: Option<DrawingSheetFormat>,
        #[serde(default)]
        orientation: Option<DrawingSheetOrientation>,
        #[serde(default)]
        projection_method: Option<DrawingProjectionMethod>,
        #[serde(default)]
        tolerance_note: Option<DrawingToleranceNoteDto>,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        drawing_number: Option<String>,
        #[serde(default)]
        revision: Option<String>,
        #[serde(default)]
        author: Option<String>,
    },
    SetActiveSheet {
        sheet_id: u64,
    },
    DeleteSheet {
        sheet_id: u64,
    },
    UpdateSheet {
        #[serde(default)]
        sheet_id: Option<u64>,
        #[serde(default)]
        patch: Value,
    },
    AutoLayout,
    AddView {
        kind: DrawingViewKind,
        #[serde(default)]
        position: Option<[f64; 2]>,
        #[serde(default)]
        parent_view_id: Option<u64>,
        #[serde(default)]
        scale: Option<f64>,
    },
    UpdateView {
        view_id: u64,
        #[serde(default)]
        patch: Value,
    },
    DeleteView {
        view_id: u64,
    },
    AddDerivedView {
        kind: DrawingViewKind,
        parent_view_id: u64,
        position: [f64; 2],
        derivation: DrawingViewDerivationDto,
    },
    AddNote {
        position: [f64; 2],
        #[serde(default)]
        text: Option<String>,
    },
    AddLinearDimension {
        view_id: u64,
        first: DrawingTopologyAnchorRefDto,
        second: DrawingTopologyAnchorRefDto,
        #[serde(default)]
        mode: Option<DrawingLinearDimensionMode>,
        #[serde(default)]
        offset: Option<f64>,
    },
    AddLineDimension {
        view_id: u64,
        first: DrawingLineRefDto,
        #[serde(default)]
        second: Option<DrawingLineRefDto>,
        mode: DrawingLineDimensionMode,
        position: [f64; 2],
    },
    AddPointLineDimension {
        view_id: u64,
        point: DrawingTopologyAnchorRefDto,
        line: DrawingLineRefDto,
        position: [f64; 2],
    },
    AddRadialDimension {
        view_id: u64,
        feature: DrawingCircularRefDto,
        mode: DrawingRadialDimensionMode,
    },
    AddAngularDimension {
        view_id: u64,
        vertex: DrawingTopologyAnchorRefDto,
        first: DrawingTopologyAnchorRefDto,
        second: DrawingTopologyAnchorRefDto,
    },
    AddHoleNote {
        view_id: u64,
        feature: DrawingCircularRefDto,
        position: [f64; 2],
    },
    AddChamferNote {
        view_id: u64,
        first: DrawingTopologyAnchorRefDto,
        second: DrawingTopologyAnchorRefDto,
        position: [f64; 2],
        #[serde(default)]
        length: Option<f64>,
        #[serde(default)]
        angle_deg: Option<f64>,
    },
    AddCenterMark {
        view_id: u64,
        feature: DrawingCircularRefDto,
        #[serde(default)]
        extension: Option<f64>,
    },
    AddCenterLine {
        view_id: u64,
        first: DrawingCircularRefDto,
        second: DrawingCircularRefDto,
        #[serde(default)]
        extension: Option<f64>,
    },
    AddCenterLineBetweenEdges {
        view_id: u64,
        first: DrawingLineRefDto,
        second: DrawingLineRefDto,
        #[serde(default)]
        extension: Option<f64>,
    },
    AddSymmetryAxis {
        view_id: u64,
        #[serde(default)]
        axis: Option<DrawingOrdinateAxis>,
        #[serde(default)]
        extension: Option<f64>,
    },
    AddBoltCircle {
        view_id: u64,
        features: Vec<DrawingCircularRefDto>,
        #[serde(default)]
        extension: Option<f64>,
    },
    AddChainDimension {
        view_id: u64,
        anchors: Vec<DrawingTopologyAnchorRefDto>,
        #[serde(default)]
        layout: Option<DrawingChainDimensionLayout>,
        #[serde(default)]
        mode: Option<DrawingLinearDimensionMode>,
    },
    AddOrdinateDimension {
        view_id: u64,
        origin: DrawingTopologyAnchorRefDto,
        target: DrawingTopologyAnchorRefDto,
        #[serde(default)]
        axis: Option<DrawingOrdinateAxis>,
    },
    AddArcLengthDimension {
        view_id: u64,
        feature: DrawingCircularRefDto,
        first: DrawingTopologyAnchorRefDto,
        second: DrawingTopologyAnchorRefDto,
    },
    AddJoggedRadius {
        view_id: u64,
        feature: DrawingCircularRefDto,
        position: [f64; 2],
    },
    AddDatum {
        view_id: u64,
        attachment: DrawingAttachmentRefDto,
        position: [f64; 2],
        #[serde(default)]
        label: Option<String>,
    },
    AddGdt {
        view_id: u64,
        attachment: DrawingAttachmentRefDto,
        position: [f64; 2],
        #[serde(default)]
        characteristic: Option<DrawingGdtCharacteristic>,
        #[serde(default)]
        tolerance: Option<f64>,
    },
    AddSurfaceTexture {
        view_id: u64,
        attachment: DrawingAttachmentRefDto,
        position: [f64; 2],
        #[serde(default)]
        roughness_ra: Option<f64>,
    },
    AddEdgeRequirement {
        view_id: u64,
        attachment: DrawingLineRefDto,
        position: [f64; 2],
        #[serde(default)]
        upper_deviation: Option<f64>,
        #[serde(default)]
        lower_deviation: Option<f64>,
    },
    AddWeld {
        view_id: u64,
        attachment: DrawingLineRefDto,
        position: [f64; 2],
        #[serde(default)]
        weld_type: Option<DrawingWeldType>,
        #[serde(default)]
        side: Option<DrawingWeldSide>,
        #[serde(default)]
        size: Option<f64>,
    },
    AddBalloon {
        view_id: u64,
        attachment: DrawingAttachmentRefDto,
        position: [f64; 2],
        bom_item_id: u64,
    },
    AddRevisionCloud {
        points: Vec<[f64; 2]>,
        #[serde(default)]
        revision: Option<String>,
    },
    AddAnnotation {
        annotation: DrawingAnnotationDto,
    },
    UpdateAnnotation {
        annotation_id: u64,
        patch: Value,
    },
    DeleteAnnotation {
        annotation_id: u64,
    },
    SaveTemplate {
        name: String,
    },
    ApplyTemplate {
        template_id: u64,
    },
    DeleteTemplate {
        template_id: u64,
    },
    AddRevision {
        revision: DrawingRevisionDraft,
    },
    UpdateRevision {
        revision_id: u64,
        patch: Value,
    },
    DeleteRevision {
        revision_id: u64,
    },
    AddBomItem {
        #[serde(default)]
        item: DrawingBomItemDraft,
    },
    UpdateBomItem {
        item_id: u64,
        patch: Value,
    },
    DeleteBomItem {
        item_id: u64,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DrawingRevisionDraft {
    pub revision: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub checked_by: String,
    #[serde(default)]
    pub approved_by: String,
    #[serde(default)]
    pub change_order: String,
    #[serde(default)]
    pub status: DrawingReleaseStatus,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DrawingBomItemDraft {
    #[serde(default)]
    pub item_number: Option<String>,
    #[serde(default)]
    pub body_id: Option<BodyId>,
    #[serde(default)]
    pub part_number: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub quantity: Option<f64>,
    #[serde(default)]
    pub material: Option<String>,
    #[serde(default)]
    pub finish: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingViewProjectionIntent {
    pub view_id: u64,
    pub body_ids: Vec<BodyId>,
    pub direction: [f64; 3],
    pub up: [f64; 3],
    pub include_hidden: bool,
    pub include_tangent_edges: bool,
    pub deflection: f64,
    #[serde(default)]
    pub section_plane: Option<DrawingSectionPlaneIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingSectionPlaneIntent {
    pub point: [f64; 3],
    pub normal: [f64; 3],
    #[serde(default)]
    pub depth: Option<f64>,
}

pub fn apply_drawing_command(
    drawing: &DrawingDocumentDto,
    scene: &SolidSceneDto,
    holes: &[HoleDefinitionDto],
    document_name: &str,
    command: DrawingCommand,
) -> Result<DrawingDocumentDto, String> {
    let before = drawing.clone();
    let preserve_release = matches!(
        command,
        DrawingCommand::AddRevision {
            revision: DrawingRevisionDraft {
                status: DrawingReleaseStatus::Released,
                ..
            }
        } | DrawingCommand::UpdateRevision { .. }
    );
    let mut next = match command {
        DrawingCommand::CreateSheet {
            standard,
            format,
            orientation,
            projection_method,
            tolerance_note,
            title,
            drawing_number,
            revision,
            author,
        } => create_sheet(
            drawing,
            standard,
            format,
            orientation,
            projection_method,
            tolerance_note,
            title.unwrap_or_else(|| document_name.to_string()),
            drawing_number.unwrap_or_default(),
            revision.unwrap_or_else(|| "A".to_string()),
            author.unwrap_or_default(),
        ),
        DrawingCommand::SetActiveSheet { sheet_id } => set_active_sheet(drawing, sheet_id)?,
        DrawingCommand::DeleteSheet { sheet_id } => delete_sheet(drawing, sheet_id),
        DrawingCommand::UpdateSheet { sheet_id, patch } => update_sheet(drawing, sheet_id, patch)?,
        DrawingCommand::AutoLayout => auto_layout(drawing, scene)?,
        DrawingCommand::AddView {
            kind,
            position,
            parent_view_id,
            scale,
        } => add_view(drawing, scene, kind, position, parent_view_id, scale)?,
        DrawingCommand::UpdateView { view_id, patch } => update_view(drawing, view_id, patch)?,
        DrawingCommand::DeleteView { view_id } => delete_view(drawing, view_id),
        DrawingCommand::AddDerivedView {
            kind,
            parent_view_id,
            position,
            derivation,
        } => add_derived_view(drawing, kind, parent_view_id, position, derivation)?,
        DrawingCommand::AddNote { position, text } => add_note(drawing, position, text)?,
        DrawingCommand::AddLinearDimension {
            view_id,
            first,
            second,
            mode,
            offset,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::LinearDimension {
                id: 0,
                view_id,
                first,
                second,
                mode: mode.unwrap_or(DrawingLinearDimensionMode::Aligned),
                offset: offset.unwrap_or(12.0),
                prefix: String::new(),
                suffix: String::new(),
                precision: 2,
                presentation: DrawingDimensionPresentationDto::default(),
            },
        )?,
        DrawingCommand::AddLineDimension {
            view_id,
            first,
            second,
            mode,
            position,
        } => {
            if mode == DrawingLineDimensionMode::Length && second.is_some() {
                return Err("A line-length dimension accepts one straight edge.".to_string());
            }
            if mode != DrawingLineDimensionMode::Length && second.is_none() {
                return Err(
                    "Line distance and angle dimensions require two straight edges.".to_string(),
                );
            }
            add_view_annotation(
                drawing,
                view_id,
                DrawingAnnotationDto::LineDimension {
                    id: 0,
                    view_id,
                    first,
                    second,
                    mode,
                    position,
                    prefix: String::new(),
                    suffix: String::new(),
                    precision: if mode == DrawingLineDimensionMode::Angle {
                        1
                    } else {
                        2
                    },
                    presentation: DrawingDimensionPresentationDto::default(),
                },
            )?
        }
        DrawingCommand::AddPointLineDimension {
            view_id,
            point,
            line,
            position,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::PointLineDimension {
                id: 0,
                view_id,
                point,
                line,
                position,
                prefix: String::new(),
                suffix: String::new(),
                precision: 2,
                presentation: DrawingDimensionPresentationDto::default(),
            },
        )?,
        DrawingCommand::AddRadialDimension {
            view_id,
            feature,
            mode,
        } => {
            if mode == DrawingRadialDimensionMode::Diameter && !feature.closed {
                return Err(
                    "Diameter dimensions require a complete circle. Use Radius for an arc."
                        .to_string(),
                );
            }
            add_view_annotation(
                drawing,
                view_id,
                DrawingAnnotationDto::RadialDimension {
                    id: 0,
                    view_id,
                    feature,
                    mode,
                    leader_angle_deg: -35.0,
                    offset: 14.0,
                    prefix: String::new(),
                    suffix: String::new(),
                    precision: 2,
                    presentation: DrawingDimensionPresentationDto::default(),
                },
            )?
        }
        DrawingCommand::AddAngularDimension {
            view_id,
            vertex,
            first,
            second,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::AngularDimension {
                id: 0,
                view_id,
                vertex,
                first,
                second,
                radius: 12.0,
                prefix: String::new(),
                suffix: String::new(),
                precision: 1,
                presentation: DrawingDimensionPresentationDto::default(),
            },
        )?,
        DrawingCommand::AddHoleNote {
            view_id,
            feature,
            position,
        } => add_hole_note(drawing, holes, view_id, feature, position)?,
        DrawingCommand::AddChamferNote {
            view_id,
            first,
            second,
            position,
            length,
            angle_deg,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::ChamferNote {
                id: 0,
                view_id,
                length: length
                    .unwrap_or_else(|| distance3(first.fallback_point, second.fallback_point)),
                first,
                second,
                position,
                angle_deg: angle_deg.unwrap_or(45.0),
                prefix: String::new(),
            },
        )?,
        DrawingCommand::AddCenterMark {
            view_id,
            feature,
            extension,
        } => {
            if !feature.closed {
                return Err("Center marks require a complete circular edge.".to_string());
            }
            add_view_annotation(
                drawing,
                view_id,
                DrawingAnnotationDto::CenterMark {
                    id: 0,
                    view_id,
                    feature,
                    extension: extension.unwrap_or(2.5),
                },
            )?
        }
        DrawingCommand::AddCenterLine {
            view_id,
            first,
            second,
            extension,
        } => {
            if !first.closed || !second.closed {
                return Err("Centerlines require two complete circular edges.".to_string());
            }
            if (first.body_id == second.body_id && first.edge_id == second.edge_id)
                || distance3(first.fallback_center, second.fallback_center) < 1.0e-7
            {
                return Err("Select two distinct circular centers for a centerline.".to_string());
            }
            add_view_annotation(
                drawing,
                view_id,
                DrawingAnnotationDto::CenterLine {
                    id: 0,
                    view_id,
                    first,
                    second,
                    extension: extension.unwrap_or(2.5),
                },
            )?
        }
        DrawingCommand::AddCenterLineBetweenEdges {
            view_id,
            first,
            second,
            extension,
        } => {
            if first.body_id == second.body_id && first.edge_id == second.edge_id {
                return Err("Select two distinct parallel edges for a centerline.".to_string());
            }
            add_view_annotation(
                drawing,
                view_id,
                DrawingAnnotationDto::CenterLineBetweenEdges {
                    id: 0,
                    view_id,
                    first,
                    second,
                    extension: extension.unwrap_or(2.5),
                },
            )?
        }
        DrawingCommand::AddSymmetryAxis {
            view_id,
            axis,
            extension,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::AutomaticSymmetryAxis {
                id: 0,
                view_id,
                axis: axis.unwrap_or(DrawingOrdinateAxis::Both),
                extension: extension.unwrap_or(2.5),
            },
        )?,
        DrawingCommand::AddBoltCircle {
            view_id,
            features,
            extension,
        } => {
            if features.len() < 3 {
                return Err("A bolt circle needs at least three circular centers.".to_string());
            }
            add_view_annotation(
                drawing,
                view_id,
                DrawingAnnotationDto::BoltCircleCenterLine {
                    id: 0,
                    view_id,
                    features,
                    extension: extension.unwrap_or(2.5),
                },
            )?
        }
        DrawingCommand::AddChainDimension {
            view_id,
            anchors,
            layout,
            mode,
        } => {
            if anchors.len() < 2 {
                return Err("Select at least two points for a dimension series.".to_string());
            }
            add_view_annotation(
                drawing,
                view_id,
                DrawingAnnotationDto::ChainDimension {
                    id: 0,
                    view_id,
                    anchors,
                    mode: mode.unwrap_or(DrawingLinearDimensionMode::Aligned),
                    layout: layout.unwrap_or(DrawingChainDimensionLayout::Chain),
                    offset: 12.0,
                    spacing: 7.0,
                    prefix: String::new(),
                    suffix: String::new(),
                    precision: 2,
                    presentation: DrawingDimensionPresentationDto::default(),
                },
            )?
        }
        DrawingCommand::AddOrdinateDimension {
            view_id,
            origin,
            target,
            axis,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::OrdinateDimension {
                id: 0,
                view_id,
                origin,
                target,
                axis: axis.unwrap_or(DrawingOrdinateAxis::Both),
                offset: 10.0,
                precision: 2,
                presentation: DrawingDimensionPresentationDto::default(),
            },
        )?,
        DrawingCommand::AddArcLengthDimension {
            view_id,
            feature,
            first,
            second,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::ArcLengthDimension {
                id: 0,
                view_id,
                feature,
                first,
                second,
                offset: 7.0,
                precision: 2,
                presentation: DrawingDimensionPresentationDto::default(),
            },
        )?,
        DrawingCommand::AddJoggedRadius {
            view_id,
            feature,
            position,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::JoggedRadiusDimension {
                id: 0,
                view_id,
                feature,
                jog: [position[0] - 10.0, position[1]],
                position,
                precision: 2,
                presentation: DrawingDimensionPresentationDto::default(),
            },
        )?,
        DrawingCommand::AddDatum {
            view_id,
            attachment,
            position,
            label,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::DatumFeature {
                id: 0,
                view_id,
                attachment,
                label: label.unwrap_or_else(|| "A".to_string()),
                position,
                target_index: None,
            },
        )?,
        DrawingCommand::AddGdt {
            view_id,
            attachment,
            position,
            characteristic,
            tolerance,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::GdtFrame {
                id: 0,
                view_id,
                attachment,
                position,
                characteristic: characteristic.unwrap_or(DrawingGdtCharacteristic::Position),
                tolerance: tolerance.unwrap_or(0.1),
                diameter_zone: true,
                material_condition: DrawingMaterialCondition::None,
                datums: Vec::new(),
                projected_zone: None,
                free_state: false,
            },
        )?,
        DrawingCommand::AddSurfaceTexture {
            view_id,
            attachment,
            position,
            roughness_ra,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::SurfaceTexture {
                id: 0,
                view_id,
                attachment,
                position,
                roughness_ra: roughness_ra.unwrap_or(3.2),
                process: String::new(),
                lay: DrawingSurfaceLay::None,
                machining_allowance: None,
            },
        )?,
        DrawingCommand::AddEdgeRequirement {
            view_id,
            attachment,
            position,
            upper_deviation,
            lower_deviation,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::EdgeRequirement {
                id: 0,
                view_id,
                attachment,
                position,
                upper_deviation: upper_deviation.unwrap_or(0.0),
                lower_deviation: lower_deviation.unwrap_or(-0.2),
                note: String::new(),
            },
        )?,
        DrawingCommand::AddWeld {
            view_id,
            attachment,
            position,
            weld_type,
            side,
            size,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::WeldSymbol {
                id: 0,
                view_id,
                attachment,
                position,
                weld_type: weld_type.unwrap_or(DrawingWeldType::Fillet),
                side: side.unwrap_or(DrawingWeldSide::Arrow),
                size: size.unwrap_or(3.0),
                length: None,
                pitch: None,
                contour: DrawingWeldContour::None,
                finish: String::new(),
                all_around: false,
                field_weld: false,
                tail: String::new(),
            },
        )?,
        DrawingCommand::AddBalloon {
            view_id,
            attachment,
            position,
            bom_item_id,
        } => add_view_annotation(
            drawing,
            view_id,
            DrawingAnnotationDto::ItemBalloon {
                id: 0,
                view_id,
                attachment,
                position,
                bom_item_id,
            },
        )?,
        DrawingCommand::AddRevisionCloud { points, revision } => {
            add_revision_cloud(drawing, points, revision)?
        }
        DrawingCommand::AddAnnotation { annotation } => {
            let view_id = annotation_view_id(&annotation);
            if let Some(view_id) = view_id {
                add_view_annotation(drawing, view_id, annotation)?
            } else {
                push_sheet_annotation(drawing, annotation)?
            }
        }
        DrawingCommand::UpdateAnnotation {
            annotation_id,
            patch,
        } => update_annotation(drawing, annotation_id, patch)?,
        DrawingCommand::DeleteAnnotation { annotation_id } => {
            delete_annotation(drawing, annotation_id)
        }
        DrawingCommand::SaveTemplate { name } => save_template(drawing, name)?,
        DrawingCommand::ApplyTemplate { template_id } => apply_template(drawing, template_id)?,
        DrawingCommand::DeleteTemplate { template_id } => delete_template(drawing, template_id),
        DrawingCommand::AddRevision { revision } => add_revision(drawing, revision)?,
        DrawingCommand::UpdateRevision { revision_id, patch } => {
            update_revision(drawing, revision_id, patch)?
        }
        DrawingCommand::DeleteRevision { revision_id } => delete_revision(drawing, revision_id)?,
        DrawingCommand::AddBomItem { item } => add_bom_item(drawing, scene, item)?,
        DrawingCommand::UpdateBomItem { item_id, patch } => {
            update_bom_item(drawing, item_id, patch)?
        }
        DrawingCommand::DeleteBomItem { item_id } => delete_bom_item(drawing, item_id),
    };
    if !preserve_release {
        return_released_sheets_to_draft(&before, &mut next);
    }
    next.validate()?;
    Ok(next)
}

pub fn projection_intents_for_sheet(
    drawing: &DrawingDocumentDto,
    scene: &SolidSceneDto,
    sheet_id: Option<u64>,
) -> Result<Vec<DrawingViewProjectionIntent>, String> {
    let sheet = match sheet_id {
        Some(id) => drawing
            .sheets
            .iter()
            .find(|sheet| sheet.id == id)
            .ok_or_else(|| format!("drawing sheet {id} does not exist"))?,
        None => active_sheet(drawing).ok_or_else(|| "Create a drawing sheet first.".to_string())?,
    };
    sheet
        .views
        .iter()
        .map(|view| {
            projection_intent_for_view(view, &sheet.views, scene, std::collections::HashSet::new())
        })
        .collect()
}

pub fn projection_intent_for_view(
    view: &DrawingViewDto,
    views: &[DrawingViewDto],
    scene: &SolidSceneDto,
    visited: std::collections::HashSet<u64>,
) -> Result<DrawingViewProjectionIntent, String> {
    let basis = current_view_basis(view, views, scene, visited);
    let section_plane = match &view.derivation {
        Some(DrawingViewDerivationDto::Section { first, depth, .. }) => {
            Some(DrawingSectionPlaneIntent {
                point: resolve_anchor_point(first, scene).unwrap_or(first.fallback_point),
                normal: basis.0,
                depth: *depth,
            })
        }
        Some(DrawingViewDerivationDto::RemovedSection { first, .. }) => {
            Some(DrawingSectionPlaneIntent {
                point: resolve_anchor_point(first, scene).unwrap_or(first.fallback_point),
                normal: basis.0,
                depth: None,
            })
        }
        _ => None,
    };
    Ok(DrawingViewProjectionIntent {
        view_id: view.id,
        body_ids: view.body_ids.clone(),
        direction: basis.0,
        up: basis.1,
        include_hidden: view.show_hidden_lines,
        include_tangent_edges: view.show_tangent_edges,
        deflection: (0.08 / view.scale).max(0.01),
        section_plane,
    })
}

fn create_sheet(
    drawing: &DrawingDocumentDto,
    standard: DrawingStandard,
    format: Option<DrawingSheetFormat>,
    orientation: Option<DrawingSheetOrientation>,
    projection_method: Option<DrawingProjectionMethod>,
    tolerance_note: Option<DrawingToleranceNoteDto>,
    title: String,
    drawing_number: String,
    revision: String,
    author: String,
) -> DrawingDocumentDto {
    let mut next = drawing.clone();
    let format = format.unwrap_or_else(|| DrawingSheetFormat::default_for_standard(standard));
    let projection_method = projection_method.unwrap_or(match standard {
        DrawingStandard::Ansi => DrawingProjectionMethod::ThirdAngle,
        DrawingStandard::Iso => DrawingProjectionMethod::FirstAngle,
    });
    let tolerance_note = tolerance_note.unwrap_or(DrawingToleranceNoteDto {
        preset: match standard {
            DrawingStandard::Ansi => DrawingTolerancePreset::AnsiDecimal,
            DrawingStandard::Iso => DrawingTolerancePreset::Iso2768Medium,
        },
        custom: String::new(),
    });
    let sheet = DrawingSheetDto {
        id: next.next_sheet_id,
        name: format!("Sheet {}", next.sheets.len() + 1),
        format,
        orientation: orientation.unwrap_or(DrawingSheetOrientation::Landscape),
        standard,
        projection_method,
        tolerance_note,
        title_block: DrawingTitleBlockDto {
            title,
            drawing_number,
            revision,
            author,
            ..DrawingTitleBlockDto::default()
        },
        views: Vec::new(),
        annotations: Vec::new(),
        style: DrawingSheetStyleDto::default(),
        template_name: "noBS CAD Default".to_string(),
        revisions: Vec::new(),
        bom: Vec::new(),
        release: DrawingReleaseDto::default(),
        revision_table_position: None,
        bom_table_position: None,
    };
    next.active_sheet_id = Some(sheet.id);
    next.next_sheet_id += 1;
    next.sheets.push(sheet);
    next
}

fn set_active_sheet(
    drawing: &DrawingDocumentDto,
    sheet_id: u64,
) -> Result<DrawingDocumentDto, String> {
    if !drawing.sheets.iter().any(|sheet| sheet.id == sheet_id) {
        return Err(format!("drawing sheet {sheet_id} does not exist"));
    }
    let mut next = drawing.clone();
    next.active_sheet_id = Some(sheet_id);
    Ok(next)
}

fn delete_sheet(drawing: &DrawingDocumentDto, sheet_id: u64) -> DrawingDocumentDto {
    let mut next = drawing.clone();
    next.sheets.retain(|sheet| sheet.id != sheet_id);
    if next.active_sheet_id == Some(sheet_id) {
        next.active_sheet_id = next.sheets.first().map(|sheet| sheet.id);
    }
    next
}

fn update_sheet(
    drawing: &DrawingDocumentDto,
    sheet_id: Option<u64>,
    patch: Value,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let id = sheet_id
        .or(next.active_sheet_id)
        .ok_or_else(|| "Create a drawing sheet first.".to_string())?;
    let sheet = next
        .sheets
        .iter_mut()
        .find(|sheet| sheet.id == id)
        .ok_or_else(|| format!("drawing sheet {id} does not exist"))?;
    merge_json(sheet, patch)?;
    Ok(next)
}

fn auto_layout(
    drawing: &DrawingDocumentDto,
    scene: &SolidSceneDto,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    {
        let preview =
            active_sheet(&next).ok_or_else(|| "Create a drawing sheet first.".to_string())?;
        if !preview.views.is_empty() {
            return Err(
                "Automatic layout is available on an empty sheet. Delete existing views or place additional views manually."
                    .to_string(),
            );
        }
    }
    let front_id = next.next_view_id;
    next.next_view_id += 1;
    let top_id = next.next_view_id;
    next.next_view_id += 1;
    let right_id = next.next_view_id;
    next.next_view_id += 1;
    let iso_id = next.next_view_id;
    next.next_view_id += 1;
    let sheet = active_sheet_mut(&mut next)?;
    let [width, height] = drawing_sheet_size(sheet.format, sheet.orientation);
    let scale = suggested_view_scale(scene, width, height);
    let front_position = [width * 0.39, height * 0.47];
    let vertical_offset = (height * 0.28).min(70.0);
    let horizontal_offset = (width * 0.24).min(90.0);
    let top_above = sheet.projection_method == DrawingProjectionMethod::ThirdAngle;
    let right_on_right = sheet.projection_method == DrawingProjectionMethod::ThirdAngle;
    sheet.views.extend([
        make_view(
            front_id,
            DrawingViewKind::Front,
            front_position,
            scale,
            None,
            DrawingViewAlignment::Free,
        ),
        make_view(
            top_id,
            DrawingViewKind::Top,
            [
                front_position[0],
                front_position[1]
                    + if top_above {
                        -vertical_offset
                    } else {
                        vertical_offset
                    },
            ],
            scale,
            Some(front_id),
            DrawingViewAlignment::Vertical,
        ),
        make_view(
            right_id,
            DrawingViewKind::Right,
            [
                front_position[0]
                    + if right_on_right {
                        horizontal_offset
                    } else {
                        -horizontal_offset
                    },
                front_position[1],
            ],
            scale,
            Some(front_id),
            DrawingViewAlignment::Horizontal,
        ),
        make_view(
            iso_id,
            DrawingViewKind::Isometric,
            [width * 0.74, height * 0.31],
            scale,
            Some(front_id),
            DrawingViewAlignment::Free,
        ),
    ]);
    Ok(next)
}

fn add_view(
    drawing: &DrawingDocumentDto,
    scene: &SolidSceneDto,
    kind: DrawingViewKind,
    position: Option<[f64; 2]>,
    parent_view_id: Option<u64>,
    scale: Option<f64>,
) -> Result<DrawingDocumentDto, String> {
    if matches!(
        kind,
        DrawingViewKind::Section
            | DrawingViewKind::Detail
            | DrawingViewKind::Auxiliary
            | DrawingViewKind::Broken
            | DrawingViewKind::RemovedSection
    ) {
        return Err(
            "Derived views require cad_drawing_add_derived_view with a derivation.".to_string(),
        );
    }
    let mut next = drawing.clone();
    let id = next.next_view_id;
    next.next_view_id += 1;
    let sheet = active_sheet_mut(&mut next)?;
    let position = position.unwrap_or_else(|| default_view_position(sheet, kind, parent_view_id));
    let view = placement_draft(sheet, scene, kind, position, parent_view_id, scale, id);
    if let Some(requested) = scale {
        if let Some(root_id) = view_group_root_id(sheet, parent_view_id.unwrap_or(view.id)) {
            let member_ids: Vec<u64> = sheet
                .views
                .iter()
                .filter(|member| view_group_root_id(sheet, member.id) == Some(root_id))
                .map(|member| member.id)
                .collect();
            for member in &mut sheet.views {
                if member_ids.contains(&member.id) {
                    member.scale = requested;
                }
            }
        }
    }
    sheet.views.push(view);
    Ok(next)
}

fn update_view(
    drawing: &DrawingDocumentDto,
    view_id: u64,
    patch: Value,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let sheet = next
        .sheets
        .iter_mut()
        .find(|sheet| sheet.views.iter().any(|view| view.id == view_id))
        .ok_or_else(|| format!("drawing view {view_id} does not exist"))?;
    let old_position = sheet
        .views
        .iter()
        .find(|view| view.id == view_id)
        .map(|view| view.position)
        .unwrap_or([0.0, 0.0]);
    {
        let view = sheet
            .views
            .iter_mut()
            .find(|view| view.id == view_id)
            .expect("view exists");
        merge_json(view, patch.clone())?;
    }
    let scale_update = patch.get("scale").and_then(Value::as_f64);
    if let Some(scale) = scale_update {
        if let Some(root_id) = view_group_root_id(sheet, view_id) {
            let member_ids: Vec<u64> = sheet
                .views
                .iter()
                .filter(|member| view_group_root_id(sheet, member.id) == Some(root_id))
                .map(|member| member.id)
                .collect();
            for member in &mut sheet.views {
                if member_ids.contains(&member.id) {
                    member.scale = scale;
                }
            }
        }
    }
    let (parent_id, alignment, new_position) = {
        let view = sheet.views.iter().find(|view| view.id == view_id).unwrap();
        (view.parent_view_id, view.alignment, view.position)
    };
    if let Some(parent_id) = parent_id {
        if patch.get("position").is_some() {
            if let Some(parent) = sheet.views.iter().find(|view| view.id == parent_id) {
                let parent_position = parent.position;
                if let Some(view) = sheet.views.iter_mut().find(|view| view.id == view_id) {
                    if alignment == DrawingViewAlignment::Horizontal {
                        view.position[1] = parent_position[1];
                    }
                    if alignment == DrawingViewAlignment::Vertical {
                        view.position[0] = parent_position[0];
                    }
                }
            }
        }
    }
    if patch.get("position").is_some() {
        let delta = [
            new_position[0] - old_position[0],
            new_position[1] - old_position[1],
        ];
        for child in sheet
            .views
            .iter_mut()
            .filter(|view| view.parent_view_id == Some(view_id))
        {
            if child.alignment == DrawingViewAlignment::Horizontal {
                child.position[1] += delta[1];
            }
            if child.alignment == DrawingViewAlignment::Vertical {
                child.position[0] += delta[0];
            }
        }
    }
    Ok(next)
}

fn delete_view(drawing: &DrawingDocumentDto, view_id: u64) -> DrawingDocumentDto {
    let mut next = drawing.clone();
    for sheet in &mut next.sheets {
        sheet.views.retain(|view| view.id != view_id);
        for child in &mut sheet.views {
            if child.parent_view_id == Some(view_id) {
                child.parent_view_id = None;
                child.alignment = DrawingViewAlignment::Free;
            }
        }
        sheet
            .annotations
            .retain(|annotation| annotation_view_id(annotation).map_or(true, |id| id != view_id));
    }
    next
}

fn add_derived_view(
    drawing: &DrawingDocumentDto,
    kind: DrawingViewKind,
    parent_view_id: u64,
    position: [f64; 2],
    derivation: DrawingViewDerivationDto,
) -> Result<DrawingDocumentDto, String> {
    if !matches!(
        kind,
        DrawingViewKind::Section
            | DrawingViewKind::Detail
            | DrawingViewKind::Auxiliary
            | DrawingViewKind::Broken
            | DrawingViewKind::RemovedSection
    ) {
        return Err("cad_drawing_add_derived_view requires a derived view kind.".to_string());
    }
    let mut next = drawing.clone();
    let id = next.next_view_id;
    next.next_view_id += 1;
    let sheet = active_sheet_mut(&mut next)?;
    let parent = sheet
        .views
        .iter()
        .find(|view| view.id == parent_view_id)
        .cloned()
        .ok_or_else(|| "Select an existing parent view first.".to_string())?;
    let (direction, up) = derived_view_basis(&parent, &derivation);
    sheet.views.push(DrawingViewDto {
        id,
        name: view_label(kind),
        kind,
        direction,
        up,
        position,
        scale: parent.scale,
        body_ids: parent.body_ids.clone(),
        show_hidden_lines: parent.show_hidden_lines,
        show_tangent_edges: parent.show_tangent_edges,
        parent_view_id: Some(parent.id),
        alignment: DrawingViewAlignment::Free,
        derivation: Some(derivation),
    });
    Ok(next)
}

fn add_note(
    drawing: &DrawingDocumentDto,
    position: [f64; 2],
    text: Option<String>,
) -> Result<DrawingDocumentDto, String> {
    push_sheet_annotation(
        drawing,
        DrawingAnnotationDto::Note {
            id: 0,
            text: text.unwrap_or_else(|| "NOTE".to_string()),
            position,
        },
    )
}

fn add_revision_cloud(
    drawing: &DrawingDocumentDto,
    points: Vec<[f64; 2]>,
    revision: Option<String>,
) -> Result<DrawingDocumentDto, String> {
    if points.len() < 3 {
        return Err("A revision cloud needs at least three points.".to_string());
    }
    push_sheet_annotation(
        drawing,
        DrawingAnnotationDto::RevisionCloud {
            id: 0,
            revision: revision.unwrap_or_else(|| "A".to_string()),
            points,
        },
    )
}

fn add_view_annotation(
    drawing: &DrawingDocumentDto,
    view_id: u64,
    mut annotation: DrawingAnnotationDto,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let id = next.next_annotation_id;
    next.next_annotation_id += 1;
    set_annotation_id(&mut annotation, id);
    let sheet = active_sheet_mut(&mut next)?;
    if !sheet.views.iter().any(|view| view.id == view_id) {
        return Err("The projected view for this annotation no longer exists.".to_string());
    }
    sheet.annotations.push(annotation);
    Ok(next)
}

fn push_sheet_annotation(
    drawing: &DrawingDocumentDto,
    mut annotation: DrawingAnnotationDto,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let id = next.next_annotation_id;
    next.next_annotation_id += 1;
    set_annotation_id(&mut annotation, id);
    active_sheet_mut(&mut next)?.annotations.push(annotation);
    Ok(next)
}

fn update_annotation(
    drawing: &DrawingDocumentDto,
    annotation_id: u64,
    patch: Value,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let annotation = next
        .sheets
        .iter_mut()
        .flat_map(|sheet| sheet.annotations.iter_mut())
        .find(|annotation| annotation.id() == annotation_id)
        .ok_or_else(|| format!("drawing annotation {annotation_id} does not exist"))?;
    merge_json(annotation, patch)?;
    if let DrawingAnnotationDto::CenterMark { extension, .. }
    | DrawingAnnotationDto::CenterLine { extension, .. }
    | DrawingAnnotationDto::CenterLineBetweenEdges { extension, .. }
    | DrawingAnnotationDto::AutomaticSymmetryAxis { extension, .. }
    | DrawingAnnotationDto::BoltCircleCenterLine { extension, .. } = annotation
    {
        *extension = extension.max(0.0);
    }
    Ok(next)
}

fn delete_annotation(drawing: &DrawingDocumentDto, annotation_id: u64) -> DrawingDocumentDto {
    let mut next = drawing.clone();
    for sheet in &mut next.sheets {
        sheet
            .annotations
            .retain(|annotation| annotation.id() != annotation_id);
    }
    next
}

fn save_template(drawing: &DrawingDocumentDto, name: String) -> Result<DrawingDocumentDto, String> {
    let normalized = name.trim().to_string();
    if normalized.is_empty() {
        return Err("Enter a template name first.".to_string());
    }
    let mut next = drawing.clone();
    let sheet = active_sheet(&next)
        .cloned()
        .ok_or_else(|| "Create a drawing sheet first.".to_string())?;
    let values = DrawingTemplateDto {
        id: 0,
        name: normalized.clone(),
        standard: sheet.standard,
        projection_method: sheet.projection_method,
        tolerance_note: sheet.tolerance_note.clone(),
        title_defaults: sheet.title_block.clone(),
        style: sheet.style.clone(),
    };
    if let Some(existing) = next
        .templates
        .iter_mut()
        .find(|template| template.name.eq_ignore_ascii_case(&normalized))
    {
        let id = existing.id;
        *existing = DrawingTemplateDto { id, ..values };
    } else {
        next.templates.push(DrawingTemplateDto {
            id: next.next_template_id,
            ..values
        });
        next.next_template_id += 1;
    }
    if let Some(sheet) = active_sheet_mut(&mut next).ok() {
        sheet.template_name = normalized;
    }
    Ok(next)
}

fn apply_template(
    drawing: &DrawingDocumentDto,
    template_id: u64,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let template = next
        .templates
        .iter()
        .find(|template| template.id == template_id)
        .cloned()
        .ok_or_else(|| "The selected drawing template no longer exists.".to_string())?;
    let sheet = active_sheet_mut(&mut next)?;
    let identity = (
        sheet.title_block.title.clone(),
        sheet.title_block.drawing_number.clone(),
        sheet.title_block.revision.clone(),
    );
    sheet.standard = template.standard;
    sheet.projection_method = template.projection_method;
    sheet.tolerance_note = template.tolerance_note.clone();
    sheet.style = template.style.clone();
    sheet.template_name = template.name.clone();
    sheet.title_block = template.title_defaults.clone();
    sheet.title_block.title = identity.0;
    sheet.title_block.drawing_number = identity.1;
    sheet.title_block.revision = identity.2;
    Ok(next)
}

fn delete_template(drawing: &DrawingDocumentDto, template_id: u64) -> DrawingDocumentDto {
    let mut next = drawing.clone();
    next.templates.retain(|template| template.id != template_id);
    next
}

fn add_revision(
    drawing: &DrawingDocumentDto,
    draft: DrawingRevisionDraft,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let id = next.next_revision_id;
    next.next_revision_id += 1;
    let released = draft.status == DrawingReleaseStatus::Released;
    let sheet = active_sheet_mut(&mut next)?;
    sheet.revisions.push(DrawingRevisionDto {
        id,
        revision: draft.revision.clone(),
        description: draft.description,
        date: draft.date.clone(),
        author: draft.author,
        checked_by: draft.checked_by,
        approved_by: draft.approved_by,
        change_order: draft.change_order,
        status: draft.status,
    });
    sheet.title_block.revision = draft.revision.clone();
    if released {
        sheet.release = DrawingReleaseDto {
            status: DrawingReleaseStatus::Released,
            released_revision: draft.revision,
            released_at: draft.date,
        };
    }
    Ok(next)
}

fn update_revision(
    drawing: &DrawingDocumentDto,
    revision_id: u64,
    patch: Value,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let sheet = active_sheet_mut(&mut next)?;
    let revision = sheet
        .revisions
        .iter_mut()
        .find(|revision| revision.id == revision_id)
        .ok_or_else(|| format!("drawing revision {revision_id} does not exist"))?;
    if revision.status == DrawingReleaseStatus::Released {
        return Err(format!(
            "Revision {} is released and immutable. Add the next revision to make changes.",
            revision.revision
        ));
    }
    merge_json(revision, patch.clone())?;
    sheet.title_block.revision = revision.revision.clone();
    if patch.get("status").and_then(Value::as_str) == Some("released") {
        sheet.release = DrawingReleaseDto {
            status: DrawingReleaseStatus::Released,
            released_revision: revision.revision.clone(),
            released_at: revision.date.clone(),
        };
    }
    Ok(next)
}

fn delete_revision(
    drawing: &DrawingDocumentDto,
    revision_id: u64,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let sheet = active_sheet_mut(&mut next)?;
    if let Some(revision) = sheet
        .revisions
        .iter()
        .find(|revision| revision.id == revision_id)
    {
        if revision.status == DrawingReleaseStatus::Released {
            return Err(format!(
                "Released revision {} cannot be deleted.",
                revision.revision
            ));
        }
    }
    sheet
        .revisions
        .retain(|revision| revision.id != revision_id);
    Ok(next)
}

fn add_bom_item(
    drawing: &DrawingDocumentDto,
    scene: &SolidSceneDto,
    item: DrawingBomItemDraft,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let id = next.next_bom_item_id;
    next.next_bom_item_id += 1;
    let sheet = active_sheet_mut(&mut next)?;
    let used: Vec<_> = sheet.bom.iter().filter_map(|entry| entry.body_id).collect();
    let body = item
        .body_id
        .and_then(|body_id| scene.bodies.iter().find(|body| body.id == body_id))
        .or_else(|| scene.bodies.iter().find(|body| !used.contains(&body.id)));
    let item_number = item
        .item_number
        .unwrap_or_else(|| (sheet.bom.len() + 1).to_string());
    let description = item.description.unwrap_or_else(|| {
        body.map(|body| body.name.clone())
            .unwrap_or_else(|| format!("Item {}", sheet.bom.len() + 1))
    });
    sheet.bom.push(DrawingBomItemDto {
        id,
        item_number,
        body_id: item.body_id.or_else(|| body.map(|body| body.id)),
        part_number: item.part_number.unwrap_or_default(),
        description,
        quantity: item.quantity.unwrap_or(1.0),
        material: item.material.unwrap_or_default(),
        finish: item.finish.unwrap_or_default(),
    });
    Ok(next)
}

fn update_bom_item(
    drawing: &DrawingDocumentDto,
    item_id: u64,
    patch: Value,
) -> Result<DrawingDocumentDto, String> {
    let mut next = drawing.clone();
    let sheet = active_sheet_mut(&mut next)?;
    let item = sheet
        .bom
        .iter_mut()
        .find(|item| item.id == item_id)
        .ok_or_else(|| format!("drawing BOM item {item_id} does not exist"))?;
    merge_json(item, patch)?;
    Ok(next)
}

fn delete_bom_item(drawing: &DrawingDocumentDto, item_id: u64) -> DrawingDocumentDto {
    let mut next = drawing.clone();
    if let Ok(sheet) = active_sheet_mut(&mut next) {
        sheet.bom.retain(|item| item.id != item_id);
        sheet.annotations.retain(|annotation| match annotation {
            DrawingAnnotationDto::ItemBalloon { bom_item_id, .. } => *bom_item_id != item_id,
            _ => true,
        });
    }
    next
}

fn add_hole_note(
    drawing: &DrawingDocumentDto,
    holes: &[HoleDefinitionDto],
    view_id: u64,
    feature: DrawingCircularRefDto,
    position: [f64; 2],
) -> Result<DrawingDocumentDto, String> {
    if !feature.closed {
        return Err("Hole notes require a complete circular edge.".to_string());
    }
    let definition = best_hole_definition(holes, &feature);
    let positions = definition
        .map(|definition| {
            if definition.positions.is_empty() {
                vec![definition.position]
            } else {
                definition
                    .positions
                    .iter()
                    .map(|entry| entry.position)
                    .collect()
            }
        })
        .unwrap_or_default();
    let quantity = positions.len().max(1) as u32;
    let depth = definition.and_then(|definition| match definition.extent {
        HoleExtent::Distance { depth } => Some(depth),
        HoleExtent::ThroughAll => None,
    });
    let thread = definition
        .and_then(|definition| definition.thread.as_ref())
        .map(|thread| {
            let mut text = thread.designation.clone();
            if !thread.class.is_empty() {
                text.push_str(" - ");
                text.push_str(&thread.class);
            }
            if thread.hand == HoleThreadHand::Left {
                text.push_str(" LH");
            }
            text
        })
        .unwrap_or_default();
    let hole_style = definition
        .map(|definition| match definition.style {
            HoleStyle::Simple => DrawingHoleStyle::Simple,
            HoleStyle::Counterbore => DrawingHoleStyle::Counterbore,
            HoleStyle::Countersink => DrawingHoleStyle::Countersink,
        })
        .unwrap_or(DrawingHoleStyle::Simple);
    add_view_annotation(
        drawing,
        view_id,
        DrawingAnnotationDto::HoleNote {
            id: 0,
            view_id,
            feature: feature.clone(),
            position,
            quantity,
            diameter: definition
                .map(|definition| definition.diameter)
                .unwrap_or(feature.fallback_radius * 2.0),
            depth,
            thread,
            note: if matches!(definition.map(|d| &d.extent), Some(HoleExtent::ThroughAll)) {
                "THRU".to_string()
            } else {
                String::new()
            },
            source_feature_id: definition.map(|definition| definition.feature_id.0),
            feature_name: definition
                .map(|definition| definition.name.clone())
                .unwrap_or_default(),
            hole_style,
            counterbore_diameter: definition.and_then(|definition| {
                (definition.style == HoleStyle::Counterbore)
                    .then_some(definition.counterbore_diameter)
            }),
            counterbore_depth: definition.and_then(|definition| {
                (definition.style == HoleStyle::Counterbore).then_some(definition.counterbore_depth)
            }),
            countersink_diameter: definition.and_then(|definition| {
                (definition.style == HoleStyle::Countersink)
                    .then_some(definition.countersink_diameter)
            }),
            countersink_angle_deg: definition.and_then(|definition| {
                (definition.style == HoleStyle::Countersink)
                    .then_some(definition.countersink_angle_deg)
            }),
            thread_depth: definition
                .and_then(|definition| definition.thread.as_ref().and_then(|thread| thread.depth)),
            pattern_note: if quantity > 1 {
                format!("{quantity} HOLES")
            } else {
                String::new()
            },
        },
    )
}

fn best_hole_definition<'a>(
    holes: &'a [HoleDefinitionDto],
    feature: &DrawingCircularRefDto,
) -> Option<&'a HoleDefinitionDto> {
    let mut best: Option<(&HoleDefinitionDto, f64)> = None;
    for definition in holes {
        if definition.body_id != feature.body_id {
            continue;
        }
        let Some(basis) = definition.face_basis else {
            continue;
        };
        let radius_error = (definition.diameter / 2.0 - feature.fallback_radius).abs();
        let radius_tolerance = (definition.diameter * 0.015).max(0.03);
        if radius_error > radius_tolerance {
            continue;
        }
        if let Some(normal) = normalize3(feature.fallback_normal) {
            let alignment = (normal[0] * basis.normal[0]
                + normal[1] * basis.normal[1]
                + normal[2] * basis.normal[2])
                .abs();
            if alignment < 0.985 {
                continue;
            }
        }
        let delta = subtract3(feature.fallback_center, basis.origin);
        let projected = [
            delta[0] * basis.u[0] + delta[1] * basis.u[1] + delta[2] * basis.u[2],
            delta[0] * basis.v[0] + delta[1] * basis.v[1] + delta[2] * basis.v[2],
        ];
        let positions = if definition.positions.is_empty() {
            vec![definition.position]
        } else {
            definition
                .positions
                .iter()
                .map(|entry| entry.position)
                .collect()
        };
        let center_error = positions
            .iter()
            .map(|position| {
                ((projected[0] - position.x).powi(2) + (projected[1] - position.y).powi(2)).sqrt()
            })
            .fold(f64::INFINITY, f64::min);
        let center_tolerance = (definition.diameter * 0.025).max(0.08);
        if center_error > center_tolerance {
            continue;
        }
        let score = center_error + radius_error * 4.0;
        let replace = match best {
            None => true,
            Some((current, current_score)) => {
                score < current_score
                    || ((score - current_score).abs() < f64::EPSILON
                        && definition.feature_id.0 < current.feature_id.0)
            }
        };
        if replace {
            best = Some((definition, score));
        }
    }
    best.map(|(definition, _)| definition)
}

fn make_view(
    id: u64,
    kind: DrawingViewKind,
    position: [f64; 2],
    scale: f64,
    parent_view_id: Option<u64>,
    alignment: DrawingViewAlignment,
) -> DrawingViewDto {
    let (direction, up) = standard_view_basis(kind);
    DrawingViewDto {
        id,
        name: view_label(kind),
        kind,
        direction,
        up,
        position,
        scale,
        body_ids: Vec::new(),
        show_hidden_lines: false,
        show_tangent_edges: false,
        parent_view_id,
        alignment,
        derivation: None,
    }
}

fn placement_draft(
    sheet: &DrawingSheetDto,
    scene: &SolidSceneDto,
    kind: DrawingViewKind,
    position: [f64; 2],
    parent_view_id: Option<u64>,
    scale: Option<f64>,
    id: u64,
) -> DrawingViewDto {
    let root = placement_root(sheet, parent_view_id);
    let (parent_id, alignment) = related_alignment(kind, root);
    let aligned = match (alignment, root) {
        (DrawingViewAlignment::Vertical, Some(root)) => [root.position[0], position[1]],
        (DrawingViewAlignment::Horizontal, Some(root)) => [position[0], root.position[1]],
        _ => position,
    };
    let [width, height] = drawing_sheet_size(sheet.format, sheet.orientation);
    let scale = scale
        .or_else(|| root.map(|root| root.scale))
        .unwrap_or_else(|| suggested_view_scale(scene, width, height));
    let mut view = make_view(id, kind, aligned, scale, parent_id, alignment);
    if let Some(root) = root {
        view.body_ids = root.body_ids.clone();
        view.show_hidden_lines = root.show_hidden_lines;
        view.show_tangent_edges = root.show_tangent_edges;
    }
    view
}

fn default_view_position(
    sheet: &DrawingSheetDto,
    kind: DrawingViewKind,
    parent_view_id: Option<u64>,
) -> [f64; 2] {
    let [width, height] = drawing_sheet_size(sheet.format, sheet.orientation);
    let Some(root) = placement_root(sheet, parent_view_id) else {
        return [width * 0.45, height * 0.43];
    };
    let vertical_offset = (height * 0.28).min(70.0);
    let horizontal_offset = (width * 0.24).min(90.0);
    let top_above = sheet.projection_method == DrawingProjectionMethod::ThirdAngle;
    let right_on_right = sheet.projection_method == DrawingProjectionMethod::ThirdAngle;
    let mut position = if matches!(kind, DrawingViewKind::Top | DrawingViewKind::Bottom) {
        let top_direction = if top_above { -1.0 } else { 1.0 };
        let direction = if kind == DrawingViewKind::Top {
            top_direction
        } else {
            -top_direction
        };
        [
            root.position[0],
            root.position[1] + direction * vertical_offset,
        ]
    } else if matches!(kind, DrawingViewKind::Left | DrawingViewKind::Right) {
        let right_direction = if right_on_right { 1.0 } else { -1.0 };
        let direction = if kind == DrawingViewKind::Right {
            right_direction
        } else {
            -right_direction
        };
        [
            root.position[0] + direction * horizontal_offset,
            root.position[1],
        ]
    } else {
        [
            root.position[0] + horizontal_offset,
            root.position[1] - vertical_offset * 0.7,
        ]
    };
    position[0] = position[0].clamp(10.0, width - 10.0);
    position[1] = position[1].clamp(10.0, height - 10.0);
    position
}

fn related_alignment(
    kind: DrawingViewKind,
    parent: Option<&DrawingViewDto>,
) -> (Option<u64>, DrawingViewAlignment) {
    let Some(parent) = parent else {
        return (None, DrawingViewAlignment::Free);
    };
    if matches!(kind, DrawingViewKind::Top | DrawingViewKind::Bottom) {
        (Some(parent.id), DrawingViewAlignment::Vertical)
    } else if matches!(kind, DrawingViewKind::Left | DrawingViewKind::Right) {
        (Some(parent.id), DrawingViewAlignment::Horizontal)
    } else {
        (Some(parent.id), DrawingViewAlignment::Free)
    }
}

fn view_group_root<'a>(sheet: &'a DrawingSheetDto, view_id: u64) -> Option<&'a DrawingViewDto> {
    let mut current = sheet.views.iter().find(|view| view.id == view_id)?;
    let mut visited = std::collections::HashSet::new();
    while let Some(parent_id) = current.parent_view_id {
        if !visited.insert(current.id) {
            break;
        }
        match sheet.views.iter().find(|view| view.id == parent_id) {
            Some(parent) => current = parent,
            None => break,
        }
    }
    Some(current)
}

fn view_group_root_id(sheet: &DrawingSheetDto, view_id: u64) -> Option<u64> {
    view_group_root(sheet, view_id).map(|view| view.id)
}

fn placement_root(
    sheet: &DrawingSheetDto,
    requested_parent_id: Option<u64>,
) -> Option<&DrawingViewDto> {
    if let Some(parent_id) = requested_parent_id {
        if let Some(root) = view_group_root(sheet, parent_id) {
            return Some(root);
        }
    }
    sheet
        .views
        .first()
        .and_then(|first| view_group_root(sheet, first.id))
}

fn standard_view_basis(kind: DrawingViewKind) -> ([f64; 3], [f64; 3]) {
    match kind {
        DrawingViewKind::Front => ([0.0, -1.0, 0.0], [0.0, 0.0, 1.0]),
        DrawingViewKind::Rear => ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
        DrawingViewKind::Left => ([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        DrawingViewKind::Right => ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        DrawingViewKind::Top => ([0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
        DrawingViewKind::Bottom => ([0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
        DrawingViewKind::Isometric | DrawingViewKind::Custom => ([1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
        DrawingViewKind::Section
        | DrawingViewKind::Detail
        | DrawingViewKind::Auxiliary
        | DrawingViewKind::Broken
        | DrawingViewKind::RemovedSection => ([1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
    }
}

fn view_label(kind: DrawingViewKind) -> String {
    match kind {
        DrawingViewKind::Front => "Front",
        DrawingViewKind::Rear => "Rear",
        DrawingViewKind::Left => "Left",
        DrawingViewKind::Right => "Right",
        DrawingViewKind::Top => "Top",
        DrawingViewKind::Bottom => "Bottom",
        DrawingViewKind::Isometric => "Isometric",
        DrawingViewKind::Custom => "Custom",
        DrawingViewKind::Section => "Section",
        DrawingViewKind::Detail => "Detail",
        DrawingViewKind::Auxiliary => "Auxiliary",
        DrawingViewKind::Broken => "Broken",
        DrawingViewKind::RemovedSection => "Removed_section",
    }
    .to_string()
}

fn suggested_view_scale(scene: &SolidSceneDto, sheet_width: f64, sheet_height: f64) -> f64 {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for body in &scene.bodies {
        for chunk in body.mesh.positions.chunks(3) {
            if chunk.len() < 3 {
                continue;
            }
            for axis in 0..3 {
                let value = chunk[axis] as f64;
                min[axis] = min[axis].min(value);
                max[axis] = max[axis].max(value);
            }
        }
    }
    let largest = (0..3).map(|axis| max[axis] - min[axis]).fold(0.0, f64::max);
    if !largest.is_finite() || largest <= 0.0 {
        return 1.0;
    }
    let target = (sheet_width * 0.2).min(sheet_height * 0.23) / largest;
    STANDARD_SCALES
        .iter()
        .copied()
        .find(|candidate| *candidate <= target)
        .unwrap_or(0.01)
}

fn derived_view_basis(
    parent: &DrawingViewDto,
    derivation: &DrawingViewDerivationDto,
) -> ([f64; 3], [f64; 3]) {
    let (first, second, flipped) = match derivation {
        DrawingViewDerivationDto::Auxiliary {
            reference, flipped, ..
        } => (reference.fallback_end, reference.fallback_start, *flipped),
        DrawingViewDerivationDto::Section { first, second, .. }
        | DrawingViewDerivationDto::RemovedSection { first, second, .. } => {
            (second.fallback_point, first.fallback_point, false)
        }
        _ => return (parent.direction, parent.up),
    };
    let edge = match normalize3(subtract3(first, second)) {
        Some(edge) => edge,
        None => return (parent.direction, parent.up),
    };
    let parent_direction = match normalize3(parent.direction) {
        Some(direction) => direction,
        None => return (parent.direction, parent.up),
    };
    let mut direction = match normalize3(cross3(edge, parent_direction)) {
        Some(direction) => direction,
        None => return (parent.direction, parent.up),
    };
    if matches!(derivation, DrawingViewDerivationDto::Auxiliary { .. }) && flipped {
        direction = scale3(direction, -1.0);
    }
    let up = normalize3(cross3(direction, edge))
        .unwrap_or_else(|| normalize3(parent.up).unwrap_or([0.0, 0.0, 1.0]));
    (direction, up)
}

fn current_view_basis(
    view: &DrawingViewDto,
    views: &[DrawingViewDto],
    scene: &SolidSceneDto,
    mut visited: std::collections::HashSet<u64>,
) -> ([f64; 3], [f64; 3]) {
    if !visited.insert(view.id) {
        return (view.direction, view.up);
    }
    let Some(derivation) = &view.derivation else {
        return (view.direction, view.up);
    };
    let Some(parent) = views.iter().find(|candidate| match derivation {
        DrawingViewDerivationDto::Section { parent_view_id, .. }
        | DrawingViewDerivationDto::Detail { parent_view_id, .. }
        | DrawingViewDerivationDto::Auxiliary { parent_view_id, .. }
        | DrawingViewDerivationDto::Broken { parent_view_id, .. }
        | DrawingViewDerivationDto::RemovedSection { parent_view_id, .. } => {
            candidate.id == *parent_view_id
        }
    }) else {
        return (view.direction, view.up);
    };
    let parent_basis = current_view_basis(parent, views, scene, visited);
    if matches!(
        derivation,
        DrawingViewDerivationDto::Detail { .. } | DrawingViewDerivationDto::Broken { .. }
    ) {
        return parent_basis;
    }
    let line = match derivation {
        DrawingViewDerivationDto::Auxiliary { reference, .. } => resolve_model_line(
            reference.body_id,
            reference.edge_id,
            &reference.edge_key,
            scene,
        )
        .unwrap_or((reference.fallback_start, reference.fallback_end)),
        DrawingViewDerivationDto::Section { first, second, .. }
        | DrawingViewDerivationDto::RemovedSection { first, second, .. } => (
            resolve_anchor_point(first, scene).unwrap_or(first.fallback_point),
            resolve_anchor_point(second, scene).unwrap_or(second.fallback_point),
        ),
        _ => return parent_basis,
    };
    let Some(edge) = normalize3(subtract3(line.1, line.0)) else {
        return (view.direction, view.up);
    };
    let Some(parent_direction) = normalize3(parent_basis.0) else {
        return (view.direction, view.up);
    };
    let Some(mut direction) = normalize3(cross3(edge, parent_direction)) else {
        return (view.direction, view.up);
    };
    if let DrawingViewDerivationDto::Auxiliary { flipped, .. } = derivation {
        if *flipped {
            direction = scale3(direction, -1.0);
        }
    } else if dot3(direction, view.direction) < 0.0 {
        direction = scale3(direction, -1.0);
    }
    let mut up = normalize3(cross3(direction, edge)).unwrap_or(view.up);
    if !matches!(derivation, DrawingViewDerivationDto::Auxiliary { .. }) && dot3(up, view.up) < 0.0
    {
        up = scale3(up, -1.0);
    }
    (direction, up)
}

fn resolve_anchor_point(
    reference: &DrawingTopologyAnchorRefDto,
    scene: &SolidSceneDto,
) -> Option<[f64; 3]> {
    let line = resolve_model_line(
        reference.body_id,
        reference.edge_id,
        &reference.edge_key,
        scene,
    )?;
    if !reference.circle_center {
        return Some(
            if matches!(
                reference.endpoint,
                crate::drawing::DrawingEdgeEndpoint::Start
            ) {
                line.0
            } else {
                line.1
            },
        );
    }
    let body = scene
        .bodies
        .iter()
        .find(|body| body.id == reference.body_id)?;
    let edge = body
        .edges
        .iter()
        .find(|edge| edge.id == reference.edge_id)
        .or_else(|| {
            body.edges
                .iter()
                .find(|edge| edge.key == reference.edge_key)
        })?;
    fit_circle_center(
        &edge
            .points
            .iter()
            .map(|point| [point.x, point.y, point.z])
            .collect::<Vec<_>>(),
    )
}

fn resolve_model_line(
    body_id: BodyId,
    edge_id: nbcad_core::EdgeId,
    edge_key: &str,
    scene: &SolidSceneDto,
) -> Option<([f64; 3], [f64; 3])> {
    let body = scene.bodies.iter().find(|body| body.id == body_id)?;
    let edge = body
        .edges
        .iter()
        .find(|edge| edge.id == edge_id)
        .or_else(|| body.edges.iter().find(|edge| edge.key == edge_key))?;
    let first = edge.points.first()?;
    let last = edge.points.last()?;
    Some(([first.x, first.y, first.z], [last.x, last.y, last.z]))
}

fn fit_circle_center(points: &[[f64; 3]]) -> Option<[f64; 3]> {
    if points.len() < 3 {
        return None;
    }
    let a = points[0];
    let b = points[points.len() / 3];
    let c = points[(points.len() * 2) / 3];
    let ab = subtract3(b, a);
    let ac = subtract3(c, a);
    let normal = cross3(ab, ac);
    let normal_squared = dot3(normal, normal);
    if normal_squared < 1.0e-16 {
        return None;
    }
    let first_term = scale3(cross3(ac, normal), dot3(ab, ab));
    let second_term = scale3(cross3(normal, ab), dot3(ac, ac));
    Some(add3(
        a,
        scale3(add3(first_term, second_term), 1.0 / (2.0 * normal_squared)),
    ))
}

fn active_sheet(drawing: &DrawingDocumentDto) -> Option<&DrawingSheetDto> {
    drawing
        .sheets
        .iter()
        .find(|sheet| Some(sheet.id) == drawing.active_sheet_id)
}

fn active_sheet_mut(drawing: &mut DrawingDocumentDto) -> Result<&mut DrawingSheetDto, String> {
    let id = drawing
        .active_sheet_id
        .ok_or_else(|| "Create a drawing sheet first.".to_string())?;
    drawing
        .sheets
        .iter_mut()
        .find(|sheet| sheet.id == id)
        .ok_or_else(|| "Create a drawing sheet first.".to_string())
}

fn return_released_sheets_to_draft(before: &DrawingDocumentDto, next: &mut DrawingDocumentDto) {
    for prior in &before.sheets {
        if prior.release.status != DrawingReleaseStatus::Released {
            continue;
        }
        let Some(current) = next.sheets.iter_mut().find(|sheet| sheet.id == prior.id) else {
            continue;
        };
        if current.release.status != DrawingReleaseStatus::Released {
            continue;
        }
        let mut prior_cmp = prior.clone();
        let mut current_cmp = current.clone();
        prior_cmp.release = DrawingReleaseDto::default();
        current_cmp.release = DrawingReleaseDto::default();
        if prior_cmp != current_cmp {
            current.release.status = DrawingReleaseStatus::Draft;
        }
    }
}

fn merge_json<T: Serialize + for<'de> Deserialize<'de>>(
    target: &mut T,
    patch: Value,
) -> Result<(), String> {
    let mut current = serde_json::to_value(&*target).map_err(|error| error.to_string())?;
    merge_value(&mut current, patch);
    *target = serde_json::from_value(current).map_err(|error| error.to_string())?;
    Ok(())
}

fn merge_value(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(target), Value::Object(patch)) => {
            for (key, value) in patch {
                if value.is_null() {
                    target.remove(&key);
                } else {
                    merge_value(target.entry(key).or_insert(Value::Null), value);
                }
            }
        }
        (target, patch) => *target = patch,
    }
}

fn annotation_view_id(annotation: &DrawingAnnotationDto) -> Option<u64> {
    match annotation {
        DrawingAnnotationDto::Note { .. } | DrawingAnnotationDto::RevisionCloud { .. } => None,
        DrawingAnnotationDto::LinearDimension { view_id, .. }
        | DrawingAnnotationDto::LineDimension { view_id, .. }
        | DrawingAnnotationDto::PointLineDimension { view_id, .. }
        | DrawingAnnotationDto::RadialDimension { view_id, .. }
        | DrawingAnnotationDto::AngularDimension { view_id, .. }
        | DrawingAnnotationDto::HoleNote { view_id, .. }
        | DrawingAnnotationDto::ChamferNote { view_id, .. }
        | DrawingAnnotationDto::CenterMark { view_id, .. }
        | DrawingAnnotationDto::CenterLine { view_id, .. }
        | DrawingAnnotationDto::CenterLineBetweenEdges { view_id, .. }
        | DrawingAnnotationDto::AutomaticSymmetryAxis { view_id, .. }
        | DrawingAnnotationDto::BoltCircleCenterLine { view_id, .. }
        | DrawingAnnotationDto::ChainDimension { view_id, .. }
        | DrawingAnnotationDto::OrdinateDimension { view_id, .. }
        | DrawingAnnotationDto::ArcLengthDimension { view_id, .. }
        | DrawingAnnotationDto::JoggedRadiusDimension { view_id, .. }
        | DrawingAnnotationDto::DatumFeature { view_id, .. }
        | DrawingAnnotationDto::GdtFrame { view_id, .. }
        | DrawingAnnotationDto::SurfaceTexture { view_id, .. }
        | DrawingAnnotationDto::EdgeRequirement { view_id, .. }
        | DrawingAnnotationDto::WeldSymbol { view_id, .. }
        | DrawingAnnotationDto::ItemBalloon { view_id, .. } => Some(*view_id),
    }
}

fn set_annotation_id(annotation: &mut DrawingAnnotationDto, id: u64) {
    match annotation {
        DrawingAnnotationDto::LinearDimension { id: slot, .. }
        | DrawingAnnotationDto::LineDimension { id: slot, .. }
        | DrawingAnnotationDto::PointLineDimension { id: slot, .. }
        | DrawingAnnotationDto::Note { id: slot, .. }
        | DrawingAnnotationDto::RadialDimension { id: slot, .. }
        | DrawingAnnotationDto::AngularDimension { id: slot, .. }
        | DrawingAnnotationDto::HoleNote { id: slot, .. }
        | DrawingAnnotationDto::ChamferNote { id: slot, .. }
        | DrawingAnnotationDto::CenterMark { id: slot, .. }
        | DrawingAnnotationDto::CenterLine { id: slot, .. }
        | DrawingAnnotationDto::CenterLineBetweenEdges { id: slot, .. }
        | DrawingAnnotationDto::AutomaticSymmetryAxis { id: slot, .. }
        | DrawingAnnotationDto::BoltCircleCenterLine { id: slot, .. }
        | DrawingAnnotationDto::ChainDimension { id: slot, .. }
        | DrawingAnnotationDto::OrdinateDimension { id: slot, .. }
        | DrawingAnnotationDto::ArcLengthDimension { id: slot, .. }
        | DrawingAnnotationDto::JoggedRadiusDimension { id: slot, .. }
        | DrawingAnnotationDto::DatumFeature { id: slot, .. }
        | DrawingAnnotationDto::GdtFrame { id: slot, .. }
        | DrawingAnnotationDto::SurfaceTexture { id: slot, .. }
        | DrawingAnnotationDto::EdgeRequirement { id: slot, .. }
        | DrawingAnnotationDto::WeldSymbol { id: slot, .. }
        | DrawingAnnotationDto::ItemBalloon { id: slot, .. }
        | DrawingAnnotationDto::RevisionCloud { id: slot, .. } => *slot = id,
    }
}

fn distance3(left: [f64; 3], right: [f64; 3]) -> f64 {
    subtract3(left, right)
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt()
}

fn subtract3(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn add3(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn cross3(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn dot3(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn scale3(vector: [f64; 3], scalar: f64) -> [f64; 3] {
    [vector[0] * scalar, vector[1] * scalar, vector[2] * scalar]
}

fn normalize3(vector: [f64; 3]) -> Option<[f64; 3]> {
    let length = distance3(vector, [0.0, 0.0, 0.0]);
    (length >= 1.0e-9).then(|| scale3(vector, 1.0 / length))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nbcad_solid::SolidSceneDto;

    fn empty_scene() -> SolidSceneDto {
        SolidSceneDto::default()
    }

    #[test]
    fn create_sheet_auto_layout_and_note() {
        let drawing = DrawingDocumentDto::default();
        let drawing = apply_drawing_command(
            &drawing,
            &empty_scene(),
            &[],
            "Bracket",
            DrawingCommand::CreateSheet {
                standard: DrawingStandard::Iso,
                format: None,
                orientation: None,
                projection_method: None,
                tolerance_note: None,
                title: None,
                drawing_number: None,
                revision: None,
                author: None,
            },
        )
        .unwrap();
        assert_eq!(drawing.sheets.len(), 1);
        assert_eq!(drawing.sheets[0].title_block.title, "Bracket");
        let drawing = apply_drawing_command(
            &drawing,
            &empty_scene(),
            &[],
            "Bracket",
            DrawingCommand::AutoLayout,
        )
        .unwrap();
        assert_eq!(drawing.sheets[0].views.len(), 4);
        let kinds: Vec<_> = drawing.sheets[0]
            .views
            .iter()
            .map(|view| view.kind)
            .collect();
        assert!(kinds.contains(&DrawingViewKind::Front));
        assert!(kinds.contains(&DrawingViewKind::Top));
        assert!(kinds.contains(&DrawingViewKind::Right));
        assert!(kinds.contains(&DrawingViewKind::Isometric));
        let drawing = apply_drawing_command(
            &drawing,
            &empty_scene(),
            &[],
            "Bracket",
            DrawingCommand::AddNote {
                position: [20.0, 20.0],
                text: Some("MCP".to_string()),
            },
        )
        .unwrap();
        assert_eq!(drawing.sheets[0].annotations.len(), 1);
        match &drawing.sheets[0].annotations[0] {
            DrawingAnnotationDto::Note { text, .. } => assert_eq!(text, "MCP"),
            other => panic!("expected note, got {other:?}"),
        }
    }

    #[test]
    fn third_angle_auto_layout_places_top_above_front() {
        let drawing = apply_drawing_command(
            &DrawingDocumentDto::default(),
            &empty_scene(),
            &[],
            "Part",
            DrawingCommand::CreateSheet {
                standard: DrawingStandard::Ansi,
                format: None,
                orientation: None,
                projection_method: None,
                tolerance_note: None,
                title: None,
                drawing_number: None,
                revision: None,
                author: None,
            },
        )
        .unwrap();
        assert_eq!(
            drawing.sheets[0].projection_method,
            DrawingProjectionMethod::ThirdAngle
        );
        let drawing = apply_drawing_command(
            &drawing,
            &empty_scene(),
            &[],
            "Part",
            DrawingCommand::AutoLayout,
        )
        .unwrap();
        let front = drawing.sheets[0]
            .views
            .iter()
            .find(|view| view.kind == DrawingViewKind::Front)
            .unwrap();
        let top = drawing.sheets[0]
            .views
            .iter()
            .find(|view| view.kind == DrawingViewKind::Top)
            .unwrap();
        assert!(top.position[1] < front.position[1]);
    }

    #[test]
    fn auto_layout_rejects_occupied_sheet() {
        let drawing = apply_drawing_command(
            &DrawingDocumentDto::default(),
            &empty_scene(),
            &[],
            "Part",
            DrawingCommand::CreateSheet {
                standard: DrawingStandard::Iso,
                format: None,
                orientation: None,
                projection_method: None,
                tolerance_note: None,
                title: None,
                drawing_number: None,
                revision: None,
                author: None,
            },
        )
        .unwrap();
        let drawing = apply_drawing_command(
            &drawing,
            &empty_scene(),
            &[],
            "Part",
            DrawingCommand::AutoLayout,
        )
        .unwrap();
        let error = apply_drawing_command(
            &drawing,
            &empty_scene(),
            &[],
            "Part",
            DrawingCommand::AutoLayout,
        )
        .unwrap_err();
        assert!(error.contains("empty sheet"));
    }

    fn sample_anchor(edge: u64, point: [f64; 3]) -> DrawingTopologyAnchorRefDto {
        DrawingTopologyAnchorRefDto {
            body_id: BodyId(1),
            edge_id: nbcad_core::EdgeId(edge),
            edge_key: format!("e{edge}"),
            endpoint: crate::drawing::DrawingEdgeEndpoint::Start,
            fallback_point: point,
            circle_center: false,
        }
    }

    #[test]
    fn host_drawing_command_roundtrip() {
        let mut manager = crate::SketchManager::new();
        let created = crate::host::handle(
            &mut manager,
            "drawing_command",
            r#"{"op":"create_sheet","title":"HostSheet"}"#,
        );
        assert!(created.contains("\"ok\":true"), "{created}");
        let laid_out =
            crate::host::handle(&mut manager, "drawing_command", r#"{"op":"auto_layout"}"#);
        assert!(laid_out.contains("\"ok\":true"), "{laid_out}");
        let noted = crate::host::handle(
            &mut manager,
            "drawing_command",
            r#"{"op":"add_note","position":[12,18],"text":"VIA HOST"}"#,
        );
        assert!(noted.contains("VIA HOST"), "{noted}");
        let drawing = manager.drawing_document();
        assert_eq!(drawing.sheets.len(), 1);
        assert_eq!(drawing.sheets[0].views.len(), 4);
    }

    #[test]
    fn linear_dimension_and_delete_annotation() {
        let drawing = apply_drawing_command(
            &DrawingDocumentDto::default(),
            &empty_scene(),
            &[],
            "Part",
            DrawingCommand::CreateSheet {
                standard: DrawingStandard::Iso,
                format: None,
                orientation: None,
                projection_method: None,
                tolerance_note: None,
                title: None,
                drawing_number: None,
                revision: None,
                author: None,
            },
        )
        .unwrap();
        let drawing = apply_drawing_command(
            &drawing,
            &empty_scene(),
            &[],
            "Part",
            DrawingCommand::AutoLayout,
        )
        .unwrap();
        let view_id = drawing.sheets[0].views[0].id;
        let drawing = apply_drawing_command(
            &drawing,
            &empty_scene(),
            &[],
            "Part",
            DrawingCommand::AddLinearDimension {
                view_id,
                first: sample_anchor(1, [0.0, 0.0, 0.0]),
                second: sample_anchor(2, [10.0, 0.0, 0.0]),
                mode: None,
                offset: None,
            },
        )
        .unwrap();
        let annotation_id = drawing.sheets[0].annotations[0].id();
        let drawing = apply_drawing_command(
            &drawing,
            &empty_scene(),
            &[],
            "Part",
            DrawingCommand::DeleteAnnotation { annotation_id },
        )
        .unwrap();
        assert!(drawing.sheets[0].annotations.is_empty());
    }
}
