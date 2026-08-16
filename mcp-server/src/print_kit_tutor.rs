//! CAD synthesis exam: a fully printed even spinner, built assembled.
//!
//! Spec: `scripts/fixtures/print-kit-tutor.spec.json`.
//! Agents follow the same numbers via `prompts/get model_print_kit`.

use serde::Deserialize;
use serde_json::{json, Value};

pub const SPEC_JSON: &str = include_str!("../../scripts/fixtures/print-kit-tutor.spec.json");

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Spec {
    pub id: String,
    pub title: String,
    pub document_name: String,
    pub slicer_target: String,
    pub nozzle_mm: f64,
    pub clearance_mm: f64,
    pub journal_d: f64,
    pub bush_id: f64,
    pub bush_od: f64,
    pub bush_h: f64,
    pub bush_seat: f64,
    pub post_d: f64,
    pub post_hole: f64,
    pub post_circle_r: f64,
    pub post_count: usize,
    pub post_h: f64,
    pub base_d: f64,
    pub base_h: f64,
    pub cone_h: f64,
    pub cone_half_deg: f64,
    pub cone_relief_d: f64,
    pub cone_tip_lift: f64,
    pub shaft_lower_h: f64,
    pub shaft_shoulder_d: f64,
    pub shaft_shoulder_h: f64,
    pub shaft_upper_h: f64,
    pub hub_od: f64,
    pub hub_h: f64,
    pub rotor_d: f64,
    pub overlap: f64,
    pub blade_wall: f64,
    pub blade_h: f64,
    pub blade_twist_deg: f64,
    pub loft_stations: usize,
    pub blade_count: usize,
    pub top_plate_d: f64,
    pub top_plate_h: f64,
    pub cap_d: f64,
    pub cap_h: f64,
    pub min_bodies: usize,
    pub min_rotor_faces: usize,
}

impl Spec {
    fn cone_r(&self) -> f64 {
        self.cone_h * self.cone_half_deg.to_radians().tan()
    }
    fn cone_apex_z(&self) -> f64 {
        self.base_h - self.cone_h
    }
    fn shaft_tip_z(&self) -> f64 {
        self.cone_apex_z() + self.cone_tip_lift
    }
    fn shoulder_z(&self) -> f64 {
        self.base_h + self.shaft_lower_h
    }
    fn shoulder_top(&self) -> f64 {
        self.shoulder_z() + self.shaft_shoulder_h
    }
    fn post_top(&self) -> f64 {
        self.base_h + self.post_h
    }
    fn top_plate_top(&self) -> f64 {
        self.post_top() + self.top_plate_h
    }
    fn shaft_top(&self) -> f64 {
        self.shoulder_top() + self.shaft_upper_h
    }
    fn post_xy(&self, index: usize) -> [f64; 2] {
        let angle = (360.0 / self.post_count.max(1) as f64) * index as f64;
        let radians = angle.to_radians();
        [
            self.post_circle_r * radians.cos(),
            self.post_circle_r * radians.sin(),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct LessonResult {
    pub id: String,
    pub pass: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Report {
    pub ok: bool,
    pub spec_id: String,
    pub lessons: Vec<LessonResult>,
    pub body_count: usize,
    pub byte_length: usize,
}

impl Report {
    fn fail(spec_id: &str, id: &str, detail: impl Into<String>) -> Self {
        Self {
            ok: false,
            spec_id: spec_id.to_string(),
            lessons: vec![LessonResult {
                id: id.to_string(),
                pass: false,
                detail: detail.into(),
            }],
            body_count: 0,
            byte_length: 0,
        }
    }
}

pub fn load_spec() -> Result<Spec, String> {
    serde_json::from_str(SPEC_JSON).map_err(|error| format!("print-kit spec: {error}"))
}

pub fn run(call: &mut impl FnMut(&str, Value) -> Result<Value, String>) -> Result<Report, String> {
    let spec = load_spec()?;
    if (spec.bush_id - spec.journal_d - spec.clearance_mm).abs() > 1e-9
        || (spec.post_hole - spec.post_d - spec.clearance_mm).abs() > 1e-9
        || (spec.bush_seat - spec.bush_od - spec.clearance_mm).abs() > 1e-9
    {
        return Ok(Report::fail(
            &spec.id,
            "clearance",
            "spec clearances are not exactly +nozzle",
        ));
    }

    call("cad_new_project", json!({}))?;
    call(
        "cad_set_document_name",
        json!({ "name": spec.document_name }),
    )?;

    let base_id = build_base(call, &spec)?;
    let shaft_id = build_shaft(call, &spec)?;
    let rotor_id = build_rotor(call, &spec, &[base_id, shaft_id])?;
    let plate_id = build_top_plate(call, &spec)?;
    let bush_id = build_bushing(call, &spec)?;
    let cap_id = build_cap(call, &spec)?;

    call(
        "cad_set_focus",
        json!({ "focus": "print", "explicit": true }),
    )?;
    for (id, preset) in [
        (base_id, "bambu.pla.basic.black"),
        (shaft_id, "bambu.pla.basic.jade_white"),
        (rotor_id, "bambu.pla.basic.green"),
        (plate_id, "bambu.pla.basic.black"),
        (bush_id, "bambu.pla.matte.dark_gray"),
        (cap_id, "bambu.pla.basic.red"),
    ] {
        call(
            "set_body_appearance",
            json!({ "body_id": id, "preset_id": preset }),
        )?;
    }
    let preflight = call("solid_export_preflight", json!({}))?;
    let exported = call(
        "solid_export_3mf",
        json!({ "slicer_target": spec.slicer_target, "include_appearance": true }),
    )?;
    let scene = call("solid_scene", json!({}))?;
    let document = call("cad_document", json!({}))?;
    Ok(grade(
        &spec, &scene, &document, &preflight, &exported, rotor_id,
    ))
}

fn build_base(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    begin_xy(call)?;
    add_circle(call, [0.0, 0.0], spec.base_d)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.base_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "base plate",
    )?;
    let base_id = first_body_id(&update)?;

    begin_xz(call)?;
    add_poly(
        call,
        &[
            [0.0, spec.cone_apex_z()],
            [spec.cone_r(), spec.base_h],
            [0.0, spec.base_h],
            [0.0, spec.cone_apex_z()],
        ],
    )?;
    let cone_sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_revolve",
            json!({
                "sketch_name": cone_sketch,
                "profile_indices": [0],
                "axis_origin": { "x": 0.0, "y": 0.0 },
                "axis_direction": { "x": 0.0, "y": 1.0 },
                "axis_line_entity_id": null,
                "angle_deg": 360.0,
                "flip": false,
                "operation": "cut",
                "target_body_ids": [base_id]
            }),
        )?,
        "thrust cup",
    )?;

    begin_xy(call)?;
    add_circle(call, [0.0, 0.0], spec.cone_relief_d)?;
    let relief = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": relief,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.base_h + 1.0 },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [base_id]
            }),
        )?,
        "cone relief",
    )?;

    let top = offset_xy(call, spec.base_h)?;
    begin_datum(call, top)?;
    add_circle(call, spec.post_xy(0), spec.post_d)?;
    let post_sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": post_sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.post_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "post",
    )?;
    let post_id = newest_body_id(&update, &[base_id])?;
    let patterned = require_clean(
        call(
            "solid_circular_pattern",
            json!({
                "body_ids": [post_id],
                "axis_origin": { "x": 0.0, "y": 0.0, "z": 0.0 },
                "axis_direction": { "x": 0.0, "y": 0.0, "z": 1.0 },
                "count": spec.post_count,
                "total_angle_deg": 360.0
            }),
        )?,
        "even posts",
    )?;
    let post_ids: Vec<u64> = patterned["scene"]["bodies"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|body| body["id"].as_u64())
        .filter(|id| *id != base_id)
        .collect();
    require_clean(
        call(
            "solid_combine",
            json!({
                "target_body_id": base_id,
                "tool_body_ids": post_ids,
                "operation": "join",
                "keep_tools": false
            }),
        )?,
        "join posts",
    )?;
    Ok(base_id)
}

fn build_shaft(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    begin_xz(call)?;
    add_poly(call, &shaft_profile(spec))?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_revolve",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "axis_origin": { "x": 0.0, "y": 0.0 },
                "axis_direction": { "x": 0.0, "y": 1.0 },
                "axis_line_entity_id": null,
                "angle_deg": 360.0,
                "flip": false,
                "operation": "new_body",
                "target_body_ids": []
            }),
        )?,
        "shaft",
    )?;
    newest_body_id(&update, &[])
}

fn build_rotor(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<u64, String> {
    let shoulder = offset_xy(call, spec.shoulder_z())?;
    begin_datum(call, shoulder.clone())?;
    add_circle(call, [0.0, 0.0], spec.hub_od)?;
    add_circle(call, [0.0, 0.0], spec.bush_id)?;
    let hub_sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": hub_sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.hub_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "rotor hub",
    )?;
    let hub_id = newest_body_id(&update, known)?;

    let stations = spec.loft_stations.max(2);
    let mut section_names = Vec::new();
    for i in 0..stations {
        let z = spec.shoulder_z() + spec.blade_h * (i as f64) / ((stations - 1) as f64);
        let ang = spec.blade_twist_deg * (i as f64) / ((stations - 1) as f64);
        let station = offset_xy(call, z)?;
        begin_datum(call, station)?;
        add_c(call, spec, ang, [0.0, 0.0])?;
        section_names.push(finish_sketch(call)?);
    }
    let update = require_clean(
        call(
            "solid_loft",
            json!({
                "sections": section_names.iter().map(|name| json!({
                    "sketch_name": name,
                    "profile_index": 0
                })).collect::<Vec<_>>(),
                "ruled": false,
                "operation": "new_body",
                "target_body_ids": [],
                "continuity": "g1",
                "centerline": null,
                "guide_rail": null
            }),
        )?,
        "rotor bucket",
    )?;
    let mut skip = known.to_vec();
    skip.push(hub_id);
    let blade_id = newest_body_id(&update, &skip)?;
    let patterned = require_clean(
        call(
            "solid_circular_pattern",
            json!({
                "body_ids": [blade_id],
                "axis_origin": { "x": 0.0, "y": 0.0, "z": 0.0 },
                "axis_direction": { "x": 0.0, "y": 0.0, "z": 1.0 },
                "count": spec.blade_count,
                "total_angle_deg": 360.0
            }),
        )?,
        "even buckets",
    )?;
    skip.push(blade_id);
    let blade_ids: Vec<u64> = patterned["scene"]["bodies"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|body| body["id"].as_u64())
        .filter(|id| !skip.contains(id) || *id == blade_id)
        .filter(|id| !known.contains(id) && *id != hub_id)
        .collect();
    let tools = if blade_ids.is_empty() {
        vec![blade_id]
    } else {
        blade_ids
    };
    require_clean(
        call(
            "solid_combine",
            json!({
                "target_body_id": hub_id,
                "tool_body_ids": tools,
                "operation": "join",
                "keep_tools": false
            }),
        )?,
        "join rotor",
    )?;
    Ok(hub_id)
}

fn build_top_plate(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    let deck = offset_xy(call, spec.post_top())?;
    begin_datum(call, deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.top_plate_d)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.top_plate_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "top plate",
    )?;
    let plate_id = newest_body_id(&update, &[])?;

    let top = offset_xy(call, spec.top_plate_top())?;
    begin_datum(call, top.clone())?;
    add_circle(call, [0.0, 0.0], spec.journal_d + spec.clearance_mm)?;
    for i in 0..spec.post_count {
        add_circle(call, spec.post_xy(i), spec.post_hole)?;
    }
    let holes = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": holes,
                "profile_indices": (0..=spec.post_count).collect::<Vec<_>>(),
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.top_plate_h + 1.0 },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [plate_id]
            }),
        )?,
        "plate through holes",
    )?;
    begin_datum(call, top)?;
    add_circle(call, [0.0, 0.0], spec.bush_seat)?;
    let seat = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": seat,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.bush_h },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [plate_id]
            }),
        )?,
        "bushing seat",
    )?;
    Ok(plate_id)
}

fn build_bushing(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    let deck = offset_xy(call, spec.post_top())?;
    begin_datum(call, deck)?;
    add_circle(call, [0.0, 0.0], spec.bush_od)?;
    add_circle(call, [0.0, 0.0], spec.bush_id)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.bush_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "printed bushing",
    )?;
    newest_body_id(&update, &[])
}

fn build_cap(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    let top = offset_xy(call, spec.top_plate_top())?;
    begin_datum(call, top)?;
    add_circle(call, [0.0, 0.0], spec.cap_d)?;
    add_circle(call, [0.0, 0.0], spec.bush_id)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.cap_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "cap",
    )?;
    newest_body_id(&update, &[])
}

fn grade(
    spec: &Spec,
    scene: &Value,
    document: &Value,
    preflight: &Value,
    exported: &Value,
    rotor_id: u64,
) -> Report {
    let bodies = scene["bodies"].as_array().cloned().unwrap_or_default();
    let features = document["features"].as_array().cloned().unwrap_or_default();
    let rotor = bodies
        .iter()
        .find(|body| body["id"].as_u64() == Some(rotor_id));
    let bytes = exported["bytes_base64"]
        .as_str()
        .and_then(|b64| {
            use base64::{engine::general_purpose::STANDARD, Engine as _};
            STANDARD.decode(b64).ok()
        })
        .unwrap_or_default();

    let mut lessons = Vec::new();
    push_lesson(
        &mut lessons,
        "clearance",
        (spec.bush_id - spec.journal_d - spec.clearance_mm).abs() < 1e-9
            && (spec.post_hole - spec.post_d - spec.clearance_mm).abs() < 1e-9
            && (spec.bush_seat - spec.bush_od - spec.clearance_mm).abs() < 1e-9,
        format!(
            "journal {:.1} in bush {:.1}; post {:.1} in hole {:.1}; bush {:.1} in seat {:.1}",
            spec.journal_d,
            spec.bush_id,
            spec.post_d,
            spec.post_hole,
            spec.bush_od,
            spec.bush_seat
        ),
    );
    push_lesson(
        &mut lessons,
        "no_press",
        spec.clearance_mm >= spec.nozzle_mm
            && spec.bush_id > spec.journal_d
            && spec.post_hole > spec.post_d,
        "printed interfaces are +0.40 slip; cone and cap retain the stack".to_string(),
    );

    let stacked = bodies.len() >= spec.min_bodies
        && bodies.iter().any(|body| {
            bbox(body).is_some_and(|box3| box3[1][2] > spec.post_top() - 1.0)
        })
        && bodies.iter().all(|body| near_axis(body, 40.0));
    push_lesson(
        &mut lessons,
        "assemble",
        stacked,
        format!(
            "{} bodies on one axis in assembly order {:?}",
            bodies.len(),
            ["base", "shaft", "rotor", "top_plate", "bushing", "cap"]
        ),
    );
    push_lesson(
        &mut lessons,
        "thrust",
        (spec.cone_half_deg - 45.0).abs() < 1e-9 && spec.cone_h >= 3.0 && spec.cone_tip_lift > 0.0,
        format!(
            "printed {}° cone-in-cup, tip lift {:.1} mm",
            spec.cone_half_deg, spec.cone_tip_lift
        ),
    );
    push_lesson(
        &mut lessons,
        "even",
        spec.post_count == 3 && spec.blade_count == 2,
        "3 posts at 120° and 2 buckets at 180°".to_string(),
    );
    push_lesson(
        &mut lessons,
        "printed_bearings",
        spec.bush_od < 20.0 && spec.bush_h <= 5.0 && spec.cone_h >= 3.0,
        "printed sleeve + printed conical thrust; no metal 608 required".to_string(),
    );

    let rotor_faces = rotor
        .and_then(|body| body["faces"].as_array().map(|faces| faces.len()))
        .unwrap_or(0);
    let rotor_h = rotor
        .and_then(bbox)
        .map(|box3| box3[1][2] - box3[0][2])
        .unwrap_or(0.0);
    push_lesson(
        &mut lessons,
        "not_2d",
        rotor_faces >= spec.min_rotor_faces && rotor_h > spec.hub_h - 1.0,
        format!("mounted rotor faces={rotor_faces} height={rotor_h:.1}"),
    );

    let timeline_ok = features.iter().all(|feature| {
        feature["status"]["state"]
            .as_str()
            .is_some_and(|state| state == "ok")
            || feature["status"].as_str() == Some("ok")
    });
    let preflight_ok = preflight["ok"] == true
        || preflight["timeline_errors"]
            .as_array()
            .is_some_and(Vec::is_empty);
    let zip_ok = bytes.len() > 32 && bytes.starts_with(b"PK");
    push_lesson(
        &mut lessons,
        "export",
        timeline_ok && preflight_ok && zip_ok,
        format!("3MF {} bytes, timeline clean", bytes.len()),
    );

    Report {
        ok: lessons.iter().all(|lesson| lesson.pass),
        spec_id: spec.id.clone(),
        lessons,
        body_count: bodies.len(),
        byte_length: bytes.len(),
    }
}

fn push_lesson(lessons: &mut Vec<LessonResult>, id: &str, pass: bool, detail: String) {
    lessons.push(LessonResult {
        id: id.to_string(),
        pass,
        detail,
    });
}

fn shaft_profile(spec: &Spec) -> Vec<[f64; 2]> {
    let journal = spec.journal_d / 2.0;
    let shoulder = spec.shaft_shoulder_d / 2.0;
    let tip = spec.shaft_tip_z();
    let cone_top = tip + spec.cone_h;
    let shoulder_z = spec.shoulder_z();
    let shoulder_top = spec.shoulder_top();
    let top = spec.shaft_top();
    vec![
        [0.0, tip],
        [spec.cone_r(), cone_top],
        [journal, cone_top],
        [journal, shoulder_z],
        [shoulder, shoulder_z],
        [shoulder, shoulder_top],
        [journal, shoulder_top],
        [journal, top],
        [0.0, top],
        [0.0, tip],
    ]
}

fn add_c(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    deg: f64,
    origin: [f64; 2],
) -> Result<(), String> {
    let e = spec.rotor_d * spec.overlap;
    let radius = (spec.rotor_d + e) / 4.0;
    let inner = radius - spec.blade_wall;
    let center = radius - e / 2.0;
    let a = deg.to_radians();
    let cx = center * a.cos();
    let cy = center * a.sin();
    let len = (cx * cx + cy * cy).sqrt().max(1e-9);
    let ux = cx / len;
    let uy = cy / len;
    let vx = -uy;
    let vy = ux;
    let p = |x: f64, y: f64| json!({ "x": x + origin[0], "y": y + origin[1] });
    let s = p(cx + radius * vx, cy + radius * vy);
    let f = p(cx + radius * ux, cy + radius * uy);
    let ept = p(cx - radius * vx, cy - radius * vy);
    let si = p(cx + inner * vx, cy + inner * vy);
    let fi = p(cx + inner * ux, cy + inner * uy);
    let ei = p(cx - inner * vx, cy - inner * vy);
    call(
        "sketch_add_arc_3pt",
        json!({ "p1": s, "p2": f, "p3": ept, "ctrl_held": false }),
    )?;
    call(
        "sketch_add_line",
        json!({ "from": ept, "to_raw": ei, "ctrl_held": false }),
    )?;
    call(
        "sketch_add_arc_3pt",
        json!({ "p1": ei, "p2": fi, "p3": si, "ctrl_held": false }),
    )?;
    call(
        "sketch_add_line",
        json!({ "from": si, "to_raw": s, "ctrl_held": false }),
    )?;
    Ok(())
}

fn begin_xy(call: &mut impl FnMut(&str, Value) -> Result<Value, String>) -> Result<(), String> {
    call(
        "cad_set_focus",
        json!({ "focus": "sketch", "explicit": true }),
    )?;
    call(
        "sketch_begin",
        json!({ "plane": { "type": "origin_plane", "plane": "xy" } }),
    )?;
    call("sketch_set_grid_snap", json!({ "enabled": false }))?;
    Ok(())
}

fn offset_xy(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    z: f64,
) -> Result<Value, String> {
    call(
        "cad_set_focus",
        json!({ "focus": "datums", "explicit": true }),
    )?;
    let created = call(
        "construction_plane_offset",
        json!({
            "reference": { "type": "origin_plane", "plane": "xy" },
            "distance": z
        }),
    )?;
    created["planes"]
        .as_array()
        .and_then(|planes| {
            planes
                .iter()
                .rev()
                .find(|plane| {
                    plane["basis"]["origin"]
                        .as_array()
                        .and_then(|origin| origin.get(2))
                        .and_then(Value::as_f64)
                        .is_some_and(|oz| (oz - z).abs() < 0.25)
                })
                .or_else(|| planes.last())
        })
        .map(|plane| plane["datum_id"].clone())
        .filter(|id| !id.is_null())
        .ok_or_else(|| format!("no datum at z={z}"))
}

fn begin_datum(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    datum_id: Value,
) -> Result<(), String> {
    call(
        "cad_set_focus",
        json!({ "focus": "sketch", "explicit": true }),
    )?;
    call(
        "sketch_begin",
        json!({ "plane": { "type": "datum_plane", "datum_id": datum_id } }),
    )?;
    call("sketch_set_grid_snap", json!({ "enabled": false }))?;
    Ok(())
}

fn begin_xz(call: &mut impl FnMut(&str, Value) -> Result<Value, String>) -> Result<(), String> {
    call(
        "cad_set_focus",
        json!({ "focus": "sketch", "explicit": true }),
    )?;
    call(
        "sketch_begin",
        json!({ "plane": { "type": "origin_plane", "plane": "xz" } }),
    )?;
    call("sketch_set_grid_snap", json!({ "enabled": false }))?;
    Ok(())
}

fn add_poly(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    points: &[[f64; 2]],
) -> Result<(), String> {
    for pair in points.windows(2) {
        let dx = pair[1][0] - pair[0][0];
        let dy = pair[1][1] - pair[0][1];
        if dx * dx + dy * dy < 1e-8 {
            continue;
        }
        call(
            "sketch_add_line",
            json!({
                "from": { "x": pair[0][0], "y": pair[0][1] },
                "to_raw": { "x": pair[1][0], "y": pair[1][1] },
                "ctrl_held": false
            }),
        )?;
    }
    Ok(())
}

fn add_circle(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    center: [f64; 2],
    diameter: f64,
) -> Result<(), String> {
    call(
        "sketch_add_circle_locked",
        json!({
            "mode": "center_diameter",
            "anchor": { "x": center[0], "y": center[1] },
            "edge_hint": { "x": center[0] + diameter / 2.0, "y": center[1] },
            "diameter_mm": diameter,
            "ctrl_held": false
        }),
    )?;
    Ok(())
}

fn finish_sketch(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
) -> Result<String, String> {
    call("sketch_finish", json!({}))?;
    let document = call("cad_document", json!({}))?;
    last_sketch(&document).ok_or_else(|| "no sketch after finish".to_string())
}

fn last_sketch(document: &Value) -> Option<String> {
    document["features"]
        .as_array()?
        .iter()
        .rev()
        .find(|feature| feature["kind"] == "sketch")
        .and_then(|feature| feature["name"].as_str().map(str::to_string))
}

fn require_clean(update: Value, label: &str) -> Result<Value, String> {
    let errors = update["scene"]["errors"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !errors.is_empty() {
        return Err(format!("{label}: {errors:?}"));
    }
    Ok(update)
}

fn first_body_id(update: &Value) -> Result<u64, String> {
    update["scene"]["bodies"]
        .as_array()
        .and_then(|bodies| bodies.first())
        .and_then(|body| body["id"].as_u64())
        .ok_or_else(|| "no body in update".to_string())
}

fn newest_body_id(update: &Value, known: &[u64]) -> Result<u64, String> {
    update["scene"]["bodies"]
        .as_array()
        .and_then(|bodies| {
            bodies
                .iter()
                .filter_map(|body| body["id"].as_u64())
                .filter(|id| !known.contains(id))
                .max()
        })
        .ok_or_else(|| "no new body".to_string())
}

fn near_axis(body: &Value, radius: f64) -> bool {
    bbox(body).is_some_and(|box3| {
        let cx = (box3[0][0] + box3[1][0]) * 0.5;
        let cy = (box3[0][1] + box3[1][1]) * 0.5;
        (cx * cx + cy * cy).sqrt() < radius
    })
}

fn bbox(body: &Value) -> Option<[[f64; 3]; 2]> {
    let mut pts = Vec::new();
    if let Some(faces) = body["faces"].as_array() {
        for face in faces {
            if let Some(origin) = xyz(&face["plane"]["origin"]) {
                pts.push(origin);
            }
        }
    }
    if let Some(edges) = body["edges"].as_array() {
        for edge in edges {
            if let Some(points) = edge["points"].as_array() {
                for point in points {
                    if let Some(xyz) = xyz(point) {
                        pts.push(xyz);
                    }
                }
            }
        }
    }
    pts.retain(|p| p.iter().all(|n| n.is_finite() && n.abs() < 1e5));
    if pts.is_empty() {
        return None;
    }
    let mut min = pts[0];
    let mut max = pts[0];
    for p in pts {
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    Some([min, max])
}

fn xyz(value: &Value) -> Option<[f64; 3]> {
    if let Some(arr) = value.as_array() {
        return Some([
            arr.first()?.as_f64()?,
            arr.get(1)?.as_f64()?,
            arr.get(2)?.as_f64()?,
        ]);
    }
    Some([
        value.get("x")?.as_f64()?,
        value.get("y")?.as_f64()?,
        value.get("z")?.as_f64()?,
    ])
}

#[cfg(test)]
mod spec_tests {
    use super::*;

    #[test]
    fn print_kit_spec_encodes_0_4_nozzle_stack() {
        let spec = load_spec().unwrap();
        assert_eq!(spec.id, "fdm-print-spinner");
        assert_eq!(spec.nozzle_mm, 0.4);
        assert_eq!(spec.clearance_mm, 0.4);
        assert!((spec.bush_id - spec.journal_d - spec.clearance_mm).abs() < 1e-12);
        assert!((spec.post_hole - spec.post_d - spec.clearance_mm).abs() < 1e-12);
        assert!((spec.bush_seat - spec.bush_od - spec.clearance_mm).abs() < 1e-12);
        assert_eq!(spec.cone_half_deg, 45.0);
        assert_eq!(spec.post_count, 3);
        assert_eq!(spec.blade_count, 2);
        assert!(spec.min_bodies >= 6);
        assert!(spec.bush_od < 20.0);
    }
}
