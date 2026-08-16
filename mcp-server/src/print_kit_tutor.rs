//! CAD synthesis exam: a printed turntable, built assembled.
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
    pub base_d: f64,
    pub base_h: f64,
    pub plate_z: f64,
    pub rotor_to_plate_gap: f64,
    pub cone_h: f64,
    pub cone_half_deg: f64,
    pub cone_relief_d: f64,
    pub male_cone_r: f64,
    pub tip_r: f64,
    pub thrust_land_od: f64,
    pub thrust_land_h: f64,
    pub thrust_float: f64,
    pub cap_float: f64,
    pub shaft_lower_h: f64,
    pub shaft_shoulder_d: f64,
    pub shaft_shoulder_h: f64,
    pub shaft_upper_h: f64,
    pub platter_d: f64,
    pub platter_h: f64,
    pub rim_d: f64,
    pub rim_depth: f64,
    pub pocket_d: f64,
    pub pocket_count: usize,
    pub pocket_circle_r: f64,
    pub drive_across: f64,
    pub drive_across_hub: f64,
    pub keeper_d: f64,
    pub keeper_h: f64,
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
    fn shoulder_z(&self) -> f64 {
        self.base_h + self.shaft_lower_h
    }
    fn shoulder_top(&self) -> f64 {
        self.shoulder_z() + self.shaft_shoulder_h
    }
    fn plate_top(&self) -> f64 {
        self.plate_z + self.keeper_h
    }
    fn bushing_z(&self) -> f64 {
        self.plate_top() - self.bush_h
    }
    fn cap_z(&self) -> f64 {
        self.plate_top() + self.cap_float
    }
    fn shaft_top(&self) -> f64 {
        self.shoulder_top() + self.shaft_upper_h
    }
    fn pocket_xy(&self, index: usize) -> [f64; 2] {
        let angle = (360.0 / self.pocket_count.max(1) as f64) * index as f64;
        let radians = angle.to_radians();
        [
            self.pocket_circle_r * radians.cos(),
            self.pocket_circle_r * radians.sin(),
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
        || (spec.bush_seat - spec.bush_od - spec.clearance_mm).abs() > 1e-9
        || (spec.drive_across_hub - spec.drive_across - spec.clearance_mm).abs() > 1e-9
    {
        return Ok(Report::fail(
            &spec.id,
            "clearance",
            "spec clearances are not exactly +nozzle",
        ));
    }

    let mut step = 0u32;
    let mut call = |name: &str, arguments: Value| {
        step += 1;
        call(name, arguments).map_err(|error| format!("step {step} {name}: {error}"))
    };

    call("cad_new_project", json!({}))?;
    call(
        "cad_set_document_name",
        json!({ "name": spec.document_name }),
    )?;

    let base_id = build_base(&mut call, &spec)?;
    let shaft_id = build_shaft(&mut call, &spec)?;
    let platter_id = build_platter(&mut call, &spec, &[base_id, shaft_id])?;
    let keeper_id = build_keeper(&mut call, &spec)?;
    let bush_id = build_bushing(&mut call, &spec)?;
    let cap_id = build_cap(&mut call, &spec)?;

    call(
        "cad_set_focus",
        json!({ "focus": "print", "explicit": true }),
    )?;
    for (id, preset) in [
        (base_id, "bambu.pla.basic.black"),
        (shaft_id, "bambu.pla.basic.jade_white"),
        (platter_id, "bambu.pla.basic.green"),
        (keeper_id, "bambu.pla.basic.black"),
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
        &spec, &scene, &document, &preflight, &exported, platter_id,
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
    let shaft_id = newest_body_id(&update, &[])?;
    cut_flats(
        call,
        shaft_id,
        spec.shoulder_top(),
        spec.platter_h + 0.2,
        spec.drive_across / 2.0,
        8.0,
    )?;
    Ok(shaft_id)
}

fn build_platter(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<u64, String> {
    let hub_deck = offset_xy(call, spec.shoulder_top())?;
    begin_datum(call, hub_deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.platter_d)?;
    add_circle(call, [0.0, 0.0], spec.bush_id)?;
    let hub_sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": hub_sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.platter_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "platter",
    )?;
    let platter_id = newest_body_id(&update, known)?;

    let rim_z = spec.shoulder_top() + spec.platter_h;
    let rim_deck = offset_xy(call, rim_z)?;
    begin_datum(call, rim_deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.rim_d)?;
    let rim_sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": rim_sketch,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.rim_depth },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [platter_id]
            }),
        )?,
        "platter rim well",
    )?;

    begin_datum(call, rim_deck)?;
    for i in 0..spec.pocket_count {
        add_circle(call, spec.pocket_xy(i), spec.pocket_d)?;
    }
    let pockets = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": pockets,
                "profile_indices": (0..spec.pocket_count).collect::<Vec<_>>(),
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.platter_h + 1.0 },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [platter_id]
            }),
        )?,
        "even wells",
    )?;
    cut_flats(
        call,
        platter_id,
        spec.shoulder_top(),
        spec.platter_h + 0.2,
        spec.drive_across_hub / 2.0,
        spec.bush_id / 2.0 + 1.0,
    )?;
    Ok(platter_id)
}

fn build_keeper(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    let deck = offset_xy(call, spec.plate_z)?;
    begin_datum(call, deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.keeper_d)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.keeper_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "keeper",
    )?;
    let keeper_id = newest_body_id(&update, &[])?;

    let top = offset_xy(call, spec.plate_top())?;
    begin_datum(call, top.clone())?;
    add_circle(call, [0.0, 0.0], spec.journal_d + spec.clearance_mm)?;
    let holes = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": holes,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.keeper_h + 1.0 },
                "taper_angle_deg": 0.0,
                "flip": true,
                "target_body_ids": [keeper_id]
            }),
        )?,
        "keeper journal",
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
                "target_body_ids": [keeper_id]
            }),
        )?,
        "bushing seat",
    )?;
    Ok(keeper_id)
}

fn build_bushing(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    let deck = offset_xy(call, spec.bushing_z())?;
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
    let top = offset_xy(call, spec.cap_z())?;
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
    platter_id: u64,
) -> Report {
    let bodies = scene["bodies"].as_array().cloned().unwrap_or_default();
    let features = document["features"].as_array().cloned().unwrap_or_default();
    let platter = bodies
        .iter()
        .find(|body| body["id"].as_u64() == Some(platter_id));
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
            && (spec.bush_seat - spec.bush_od - spec.clearance_mm).abs() < 1e-9
            && (spec.drive_across_hub - spec.drive_across - spec.clearance_mm).abs() < 1e-9,
        format!(
            "journal {:.1} in bush {:.1}; bush {:.1} in seat {:.1}",
            spec.journal_d, spec.bush_id, spec.bush_od, spec.bush_seat
        ),
    );
    push_lesson(
        &mut lessons,
        "no_press",
        spec.clearance_mm >= spec.nozzle_mm && spec.bush_id > spec.journal_d,
        "printed interfaces are +0.40 slip; cone and cap retain the stack".to_string(),
    );

    let proper_stack = spec.keeper_h > spec.bush_h
        && spec.platter_d > spec.base_d + 8.0
        && spec.keeper_d < spec.platter_d * 0.5
        && spec.thrust_float > 0.0
        && spec.cap_float > 0.0;
    let stacked = bodies.len() >= spec.min_bodies
        && bodies
            .iter()
            .any(|body| bbox(body).is_some_and(|box3| box3[1][2] > spec.plate_top() - 1.0))
        && bodies.iter().all(|body| near_axis(body, 50.0))
        && proper_stack;
    push_lesson(
        &mut lessons,
        "assemble",
        stacked,
        format!(
            "{} coaxial bodies; platter Ø{:.0} on foot Ø{:.0}; keeper Ø{:.0}",
            bodies.len(),
            spec.platter_d,
            spec.base_d,
            spec.keeper_d
        ),
    );
    push_lesson(
        &mut lessons,
        "thrust",
        (spec.cone_half_deg - 45.0).abs() < 1e-9
            && spec.cone_h >= 3.0
            && spec.male_cone_r + 0.15 < spec.cone_r()
            && spec.thrust_land_od > spec.journal_d
            && spec.thrust_float >= 0.2,
        format!(
            "45° cup r{:.1} / male r{:.1}; Ø{:.0} land float {:.1}",
            spec.cone_r(),
            spec.male_cone_r,
            spec.thrust_land_od,
            spec.thrust_float
        ),
    );
    push_lesson(
        &mut lessons,
        "even",
        spec.pocket_count == 3 && spec.drive_across_hub > spec.drive_across,
        "3 wells at 120°, double-D drive".to_string(),
    );
    push_lesson(
        &mut lessons,
        "printed_bearings",
        spec.bush_od < 20.0 && spec.bush_h < spec.keeper_h && spec.cone_h >= 3.0,
        "printed sleeve on a 2 mm land + printed cone/land thrust; no metal 608".to_string(),
    );

    let platter_faces = platter
        .and_then(|body| body["faces"].as_array().map(|faces| faces.len()))
        .unwrap_or(0);
    let platter_span = platter
        .and_then(bbox)
        .map(|box3| box3[1][2] - box3[0][2])
        .unwrap_or(0.0);
    push_lesson(
        &mut lessons,
        "not_2d",
        platter_faces >= spec.min_rotor_faces && platter_span > spec.platter_h - 1.0,
        format!("mounted platter faces={platter_faces} height={platter_span:.1}"),
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
    let land = spec.thrust_land_od / 2.0;
    let tip_z = spec.cone_apex_z() + 0.5;
    let mouth_z = spec.base_h;
    let land_z = spec.base_h + spec.thrust_float;
    let land_top = land_z + spec.thrust_land_h;
    let shoulder_z = spec.shoulder_z();
    let shoulder_top = spec.shoulder_top();
    let top = spec.shaft_top();
    vec![
        [0.0, tip_z],
        [spec.tip_r, tip_z],
        [spec.male_cone_r, mouth_z],
        [spec.male_cone_r, land_z],
        [land, land_z],
        [land, land_top],
        [journal, land_top],
        [journal, shoulder_z],
        [shoulder, shoulder_z],
        [shoulder, shoulder_top],
        [journal, shoulder_top],
        [journal, top],
        [0.0, top],
        [0.0, tip_z],
    ]
}

fn cut_flats(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    body_id: u64,
    z: f64,
    depth: f64,
    half_across: f64,
    outer: f64,
) -> Result<(), String> {
    let datum = offset_xy(call, z)?;
    begin_datum(call, datum)?;
    call(
        "sketch_add_rectangle",
        json!({
            "mode": "two_point",
            "p1": { "x": half_across, "y": -outer },
            "p2": { "x": outer, "y": outer },
            "ctrl_held": false
        }),
    )?;
    call(
        "sketch_add_rectangle",
        json!({
            "mode": "two_point",
            "p1": { "x": -outer, "y": -outer },
            "p2": { "x": -half_across, "y": outer },
            "ctrl_held": false
        }),
    )?;
    let sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0, 1],
                "operation": "cut",
                "extent": { "type": "distance", "distance": depth },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [body_id]
            }),
        )?,
        "double-D flats",
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
        assert_eq!(spec.id, "fdm-print-turntable");
        assert_eq!(spec.nozzle_mm, 0.4);
        assert_eq!(spec.clearance_mm, 0.4);
        assert!((spec.bush_id - spec.journal_d - spec.clearance_mm).abs() < 1e-12);
        assert!((spec.bush_seat - spec.bush_od - spec.clearance_mm).abs() < 1e-12);
        assert!((spec.drive_across_hub - spec.drive_across - spec.clearance_mm).abs() < 1e-12);
        assert_eq!(spec.cone_half_deg, 45.0);
        assert!(spec.male_cone_r + 0.15 < spec.cone_r());
        assert!(spec.keeper_h > spec.bush_h);
        assert!(spec.platter_d > spec.base_d + 8.0);
        assert!(spec.keeper_d < spec.platter_d * 0.5);
        assert!(
            (spec.plate_z
                - (spec.base_h
                    + spec.shaft_lower_h
                    + spec.shaft_shoulder_h
                    + spec.platter_h
                    + spec.rotor_to_plate_gap))
                .abs()
                < 1e-9
        );
        assert!(spec.shaft_top() > spec.plate_top() + spec.cap_float + spec.cap_h);
        assert_eq!(spec.pocket_count, 3);
        assert!(spec.min_bodies >= 6);
        assert!(spec.bush_od < 20.0);
    }
}
