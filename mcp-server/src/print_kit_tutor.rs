//! CAD synthesis exam: a printed VAWT *assembly*.
//!
//! Spec: `scripts/fixtures/print-kit-tutor.spec.json`.
//! Linear numbers are the X2D-max design; `scale` shrinks the source.

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
    pub scale: f64,
    pub max_scale: f64,
    pub printer: Printer,
    pub fit_running_mm: f64,
    pub fit_slip_mm: f64,
    pub fit_friction_mm: f64,
    pub fit_pip_mm: f64,
    pub bed_relief_mm: f64,
    pub filament: Filament,
    pub materials: PrintMaterials,
    pub wing_count: usize,
    pub wing_h: f64,
    pub wing_radius: f64,
    pub wing_chord_root: f64,
    pub wing_chord_tip: f64,
    pub wing_thick: f64,
    pub wing_offset_deg: f64,
    pub helix_deg: f64,
    pub helix_stations: usize,
    pub airfoil: String,
    pub airfoil_t_c: f64,
    #[serde(default = "default_airfoil_le_index")]
    pub airfoil_le_index: f64,
    #[serde(default = "default_airfoil_xt_c")]
    pub airfoil_xt_c: f64,
    pub airfoil_te_min_mm: f64,
    pub airfoil_stations: usize,
    pub hub_h: f64,
    pub roller_count: usize,
    pub roller_d: f64,
    pub roller_h: f64,
    pub roller_len: f64,
    pub roller_min_d: f64,
    pub inner_race_d: f64,
    pub axle_flange_d: f64,
    pub axle_flange_h: f64,
    pub axle_square: f64,
    pub axle_square_h: f64,
    pub base_h: f64,
    pub rib_w: f64,
    pub pad_d: f64,
    pub post_count: usize,
    pub post_circle_r: f64,
    pub base_boss_d: f64,
    pub cage_h: f64,
    pub retainer_h: f64,
    pub thrust_float: f64,
    pub min_bodies: usize,
    pub min_rotor_faces: usize,
    pub min_print_plates: usize,
    pub print_plates: Vec<String>,
    pub retired_print_plates: Vec<String>,
}

fn default_airfoil_le_index() -> f64 {
    4.5
}
fn default_airfoil_xt_c() -> f64 {
    0.35
}

#[derive(Debug, Clone, Deserialize)]
pub struct Printer {
    pub name: String,
    pub bed_mm: [f64; 3],
    pub margin_mm: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Filament {
    pub name: String,
    pub density_g_cm3: f64,
    pub price_usd_per_kg: f64,
    pub print_volume_factor: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrintMaterials {
    pub orange: String,
    pub glow: String,
    #[serde(default = "default_petg")]
    pub petg: String,
}

fn default_petg() -> String {
    "bambu.petg.hf.black".to_string()
}

impl Spec {
    fn mm(&self, value: f64) -> f64 {
        value * self.scale
    }
    fn mm_min(&self, value: f64, min: f64) -> f64 {
        (value * self.scale).max(min)
    }
    fn wall(&self) -> f64 {
        self.mm_min(4.0, self.nozzle_mm * 4.0)
    }
    fn te_min(&self) -> f64 {
        self.airfoil_te_min_mm.max(self.nozzle_mm * 2.0)
    }
    fn roller_d(&self) -> f64 {
        self.mm_min(self.roller_d, self.roller_min_d)
    }
    fn pack_h(&self) -> f64 {
        // Pack height is the roller diameter: lying cylinders, axis radial.
        self.roller_d()
    }
    fn roller_len(&self) -> f64 {
        self.mm_min(self.roller_len, 8.0)
    }
    fn pocket_len(&self) -> f64 {
        self.roller_len() + self.fit_running_mm
    }
    fn inner_race_d(&self) -> f64 {
        self.mm_min(self.inner_race_d, 12.0)
    }
    fn plate_bore(&self) -> f64 {
        self.inner_race_d() + self.fit_running_mm
    }
    fn hub_od(&self) -> f64 {
        self.hub_deck_od()
    }
    fn hub_h(&self) -> f64 {
        self.hub_deck_h()
    }
    fn hub_deck_h(&self) -> f64 {
        self.mm_min(10.0, 3.2).max(self.bed_relief_h())
    }
    fn hub_deck_od(&self) -> f64 {
        (self.wing_radius() + self.chord_root() * 0.18) * 2.0
    }
    fn blade_loft_z(&self) -> f64 {
        self.hub_z() + self.hub_deck_h()
    }
    fn blade_root_z(&self) -> f64 {
        self.hub_z() + self.hub_deck_h()
    }
    fn hub_square(&self) -> f64 {
        self.mm(self.axle_square) + self.fit_friction_mm
    }
    fn axle_square(&self) -> f64 {
        self.mm(self.axle_square)
    }
    fn axle_square_h(&self) -> f64 {
        self.mm_min(self.axle_square_h, self.hub_h() + 2.0)
    }
    fn axle_flange_d(&self) -> f64 {
        // Race covers the rollers and the cage rim. Stay inside the
        // plate — cage_od+4 was the orange halo under a smaller deck.
        let race = self.pcd() + self.roller_len() + 2.0;
        race.max(self.cage_od()).min(self.hub_deck_od() - 2.0)
    }
    fn axle_flange_h(&self) -> f64 {
        self.mm_min(self.axle_flange_h, 2.4)
    }
    fn wing_h(&self) -> f64 {
        self.mm(self.wing_h)
    }
    fn wing_radius(&self) -> f64 {
        self.mm(self.wing_radius)
    }
    fn chord_root(&self) -> f64 {
        self.mm(self.wing_chord_root)
    }
    fn chord_tip(&self) -> f64 {
        self.mm(self.wing_chord_tip)
    }
    fn wing_thick(&self) -> f64 {
        self.chord_root() * self.airfoil_t_c
    }
    fn base_h(&self) -> f64 {
        self.mm_min(self.base_h, 3.2)
    }
    fn rib_w(&self) -> f64 {
        self.mm_min(self.rib_w, 5.0)
    }
    fn pad_d(&self) -> f64 {
        self.mm_min(self.pad_d, 10.0)
    }
    fn post_circle_r(&self) -> f64 {
        self.mm(self.post_circle_r)
    }
    fn base_boss_d(&self) -> f64 {
        // Small Y-frame hub. The race is a ring under the rollers, not a cookie.
        (self.journal_d() + 2.0 * self.wall())
            .max(16.0)
            .min(self.axle_flange_d() - 8.0)
    }
    fn race_id(&self) -> f64 {
        (self.pcd() - self.roller_len() - 2.0 * self.keeper())
            .max(self.base_boss_d() + 2.0 * self.wall())
    }
    fn keeper(&self) -> f64 {
        self.wall()
    }
    fn root_scale(&self) -> f64 {
        1.28
    }
    fn root_blend_h(&self) -> f64 {
        self.mm_min(16.0, 6.0)
    }
    fn tip_taper_h(&self) -> f64 {
        self.mm_min(12.0, 4.0)
    }
    fn tip_scale(&self) -> f64 {
        0.72
    }
    fn tip_chord(&self) -> f64 {
        self.chord_tip() * self.tip_scale()
    }
    fn top_load(&self) -> f64 {
        self.nozzle_mm * 2.0
    }
    fn top_load_pocket(&self) -> f64 {
        self.roller_d() + self.fit_running_mm + self.top_load()
    }
    fn window_floor(&self) -> f64 {
        self.race_z() + 0.05
    }
    fn window_w(&self) -> f64 {
        self.roller_d() + 4.0 * self.nozzle_mm
    }
    fn mouth_w(&self) -> f64 {
        self.roller_d() + 8.0 * self.nozzle_mm
    }
    fn funnel_h(&self) -> f64 {
        (self.fence_h() * 0.45).clamp(1.2, 2.4)
    }
    fn crown_drop(&self) -> f64 {
        self.nozzle_mm.clamp(0.30, 0.40)
    }
    fn roller_end_d(&self) -> f64 {
        self.roller_d() - 2.0 * self.crown_drop()
    }
    fn land_len(&self) -> f64 {
        (self.roller_len() * 0.22).max(self.nozzle_mm * 6.0)
    }
    fn roller_bore_d(&self) -> f64 {
        // Stay >2.5 mm away from mid Ø/2 so the revolute matcher
        // cannot pick the bore instead of the land (radiusErr = 2.5).
        self.nozzle_mm * 6.0
    }
    fn clip_arm_t(&self) -> f64 {
        self.wall()
    }
    fn clip_mouth(&self) -> f64 {
        (self.groove_d() - 2.4).max(self.wall() * 5.0)
    }
    fn clip_tab_w(&self) -> f64 {
        self.wall()
    }
    fn clip_tab_l(&self) -> f64 {
        self.wall() * 2.0
    }
    fn witness_d(&self) -> f64 {
        self.nozzle_mm * 6.0
    }
    fn fence_h(&self) -> f64 {
        let raw = (self.pack_h() * 0.62).max(self.wall() * 2.0);
        raw.min(self.pack_h() - 1.2)
    }
    fn shoulder_h(&self) -> f64 {
        0.0
    }
    fn shoulder_d(&self) -> f64 {
        // No fat flange above the plate — that blocked dropping the rotor on.
        self.journal_d()
    }
    fn bead_h(&self) -> f64 {
        self.tip_h()
    }
    fn bead_d(&self) -> f64 {
        self.pass_d()
    }
    fn groove_depth(&self) -> f64 {
        self.nozzle_mm * 2.0
    }
    fn groove_d(&self) -> f64 {
        self.journal_d() - 2.0 * self.groove_depth()
    }
    fn groove_h(&self) -> f64 {
        self.retainer_h() + self.thrust_float
    }
    fn tip_h(&self) -> f64 {
        self.wall()
    }
    fn pass_d(&self) -> f64 {
        // The plate must pass every diameter from the tip down. Nothing fatter.
        self.journal_d()
    }
    fn lock_flat_x(&self) -> f64 {
        self.inner_race_d() * 0.22
    }
    fn snap_gap(&self) -> f64 {
        self.clip_mouth()
    }
    fn journal_d(&self) -> f64 {
        self.inner_race_d()
    }
    fn retainer_d_hole(&self) -> f64 {
        self.groove_d() + self.fit_slip_mm
    }
    fn retainer_flat_x(&self) -> f64 {
        self.lock_flat_x() + self.fit_slip_mm * 0.5
    }
    fn journal_h(&self) -> f64 {
        self.groove_z() + self.groove_h() + self.tip_h() - self.race_z()
    }
    fn cage_rim(&self) -> f64 {
        self.wall() * 2.0
    }
    fn cage_od(&self) -> f64 {
        self.pcd() + self.roller_len() + 2.0 * self.cage_rim()
    }
    fn cage_id(&self) -> f64 {
        // Fence sits on the race ring only. Do not fill the Y-frame
        // with a second cookie. Still looser than the plate bore.
        self.race_id().max(self.plate_bore() + 2.0 * self.wall())
    }
    fn cage_h(&self) -> f64 {
        self.fence_h()
    }
    fn cage_pocket(&self) -> f64 {
        self.roller_d() + self.fit_running_mm
    }
    fn bed_relief_h(&self) -> f64 {
        self.bed_relief_mm
    }
    fn bed_relief_d(&self) -> f64 {
        self.bed_relief_mm
    }
    fn retainer_od(&self) -> f64 {
        self.retainer_d_hole() + 2.0 * self.clip_arm_t()
    }
    fn retainer_square(&self) -> f64 {
        self.axle_square() + self.fit_slip_mm
    }
    fn retainer_h(&self) -> f64 {
        self.mm_min(self.retainer_h, 2.0)
    }
    fn pcd(&self) -> f64 {
        // Midline under the blade roots so the couple does not
        // cantilever across a 5 mm plate (the inboard-pack cracker).
        let under_root = self.wing_radius() * 2.0;
        let max_fit = self.hub_deck_od() - self.roller_len() - 2.0 * self.cage_rim() - 2.0;
        let min_fit = self.inner_race_d() + self.roller_len() + 2.0 * self.wall();
        under_root.min(max_fit).max(min_fit)
    }
    fn usable_bed(&self) -> [f64; 3] {
        [
            self.printer.bed_mm[0] - 2.0 * self.printer.margin_mm,
            self.printer.bed_mm[1] - 2.0 * self.printer.margin_mm,
            self.printer.bed_mm[2] - 2.0 * self.printer.margin_mm,
        ]
    }
    fn blade_tip_r(&self) -> f64 {
        self.wing_radius() + self.chord_tip() * 0.15
    }
    fn rotor_d(&self) -> f64 {
        self.blade_tip_r() * 2.0
    }
    fn base_envelope(&self) -> f64 {
        self.post_circle_r() * 2.0 + self.pad_d()
    }
    fn rotor_print_h(&self) -> f64 {
        self.wing_h()
            .max(self.hub_z() + self.hub_h() - self.plate_z())
    }
    fn fits_x2d_at_max(&self) -> bool {
        let bed = self.usable_bed();
        self.rotor_d() / self.scale <= bed[0]
            && self.rotor_print_h() / self.scale <= bed[2]
            && self.base_envelope() / self.scale <= bed[0]
    }
    fn flange_z(&self) -> f64 {
        0.0
    }
    fn race_z(&self) -> f64 {
        self.base_h()
    }
    fn race_h(&self) -> f64 {
        self.journal_h()
    }
    fn cage_z(&self) -> f64 {
        self.race_z()
    }
    fn shoulder_z(&self) -> f64 {
        self.groove_z()
    }
    fn groove_z(&self) -> f64 {
        self.plate_z() + self.hub_deck_h() + self.thrust_float
    }
    fn bead_z(&self) -> f64 {
        self.groove_z() + self.groove_h()
    }
    fn z_mid(&self) -> f64 {
        self.cage_z() + self.pack_h() * 0.5
    }
    fn hub_z(&self) -> f64 {
        self.plate_z()
    }
    fn plate_z(&self) -> f64 {
        self.cage_z() + self.pack_h() + self.thrust_float
    }
    fn retainer_z(&self) -> f64 {
        self.groove_z()
    }
    fn assemble_ok(&self) -> bool {
        self.pass_d() + 1e-9 < self.plate_bore()
            && (self.pass_d() - self.journal_d()).abs() < 1e-9
            && (self.bead_d() - self.pass_d()).abs() < 1e-9
            && (self.shoulder_d() - self.pass_d()).abs() < 1e-9
            && self.groove_d() + 1e-9 < self.pass_d()
            && self.retainer_d_hole() + 1e-9 < self.pass_d()
            && self.retainer_d_hole() + 1e-9 >= self.groove_d()
            && self.snap_gap() + 1e-9 >= self.pass_d() - self.retainer_d_hole()
            && self.clip_mouth() + 1e-9 < self.groove_d()
            && (self.retainer_od() - (self.retainer_d_hole() + 2.0 * self.clip_arm_t())).abs()
                < 1e-9
            && self.groove_z() + 1e-9 >= self.plate_z() + self.hub_deck_h()
            && (self.retainer_z() - self.groove_z()).abs() < 1e-9
            && self.groove_h() + 1e-9 >= self.retainer_h()
    }
    fn post_h(&self) -> f64 {
        self.journal_h()
    }
    fn wing_angle_deg(&self, index: usize) -> f64 {
        self.wing_offset_deg + 360.0 / self.wing_count.max(1) as f64 * index as f64
    }
    fn helix_azimuth_deg(&self, index: usize, t: f64) -> f64 {
        self.wing_angle_deg(index) + self.helix_deg * t
    }
    fn helix_center(&self, index: usize, t: f64) -> [f64; 2] {
        let radians = self.helix_azimuth_deg(index, t).to_radians();
        let r = self.wing_radius();
        [r * radians.cos(), r * radians.sin()]
    }
    fn post_xy(&self, index: usize) -> [f64; 2] {
        let radians = (360.0 / self.post_count.max(1) as f64 * index as f64).to_radians();
        let r = self.post_circle_r();
        [r * radians.cos(), r * radians.sin()]
    }
    fn roller_angle_deg(&self, index: usize) -> f64 {
        360.0 / self.roller_count.max(1) as f64 * index as f64
    }
    fn roller_xy(&self, index: usize) -> [f64; 2] {
        let radians = self.roller_angle_deg(index).to_radians();
        let r = self.pcd() * 0.5;
        [r * radians.cos(), r * radians.sin()]
    }
    fn roller_axis(&self, index: usize) -> [f64; 3] {
        let radians = self.roller_angle_deg(index).to_radians();
        [radians.cos(), radians.sin(), 0.0]
    }
    fn solidity(&self) -> f64 {
        (self.wing_count as f64) * self.chord_root() / (std::f64::consts::PI * self.rotor_d())
    }
    fn airfoil_ok(&self) -> bool {
        self.airfoil.to_ascii_uppercase().contains("NACA")
            && self.airfoil_t_c >= 0.18
            && self.airfoil_t_c <= 0.26
            && self.airfoil_xt_c >= 0.275
            && self.airfoil_xt_c <= 0.40
            && self.airfoil_le_index >= 4.0
            && self.airfoil_le_index <= 6.0
            && (self.wing_thick - self.wing_chord_root * self.airfoil_t_c).abs() < 0.2
            && self.te_min() + 1e-9 >= self.nozzle_mm * 2.0
            && self.chord_root() > self.chord_tip()
    }
    fn fits_ok(&self) -> bool {
        (self.fit_running_mm - self.nozzle_mm).abs() < 1e-9
            && self.fit_friction_mm + 1e-9 < self.fit_slip_mm
            && self.fit_slip_mm + 1e-9 < self.fit_running_mm
            && self.fit_running_mm + 1e-9 < self.fit_pip_mm
            && (self.clearance_mm - self.fit_running_mm).abs() < 1e-9
            && (self.bed_relief_mm - self.nozzle_mm * 2.0).abs() < 1e-9
            && (self.fit_pip_mm - self.nozzle_mm * 2.0).abs() < 1e-9
    }
    fn pack_outer_r(&self) -> f64 {
        self.pcd() * 0.5 + self.roller_len() * 0.5
    }
    fn captured_ok(&self) -> bool {
        let roller_inner = self.pcd() * 0.5 - self.roller_len() * 0.5;
        let roller_outer = self.pcd() * 0.5 + self.roller_len() * 0.5;
        let pocket_inner = self.pcd() * 0.5 - self.pocket_len() * 0.5;
        let pocket_outer = self.pcd() * 0.5 + self.pocket_len() * 0.5;
        self.keeper() + 1e-9 >= self.nozzle_mm * 2.0
            && self.race_id() * 0.5 + 1e-9 <= roller_inner
            && pocket_inner + 1e-9 > self.race_id() * 0.5
            && roller_outer + self.keeper() <= self.cage_od() * 0.5 + 1e-9
            && pocket_outer + 1e-9 < self.cage_od() * 0.5
    }
    fn rollers_ok(&self) -> bool {
        let axis0 = self.roller_axis(0);
        self.roller_count >= 6
            && self.roller_d() + 1e-9 >= self.roller_min_d
            && self.roller_len() + 1e-9 >= 8.0
            && (self.pack_h() - self.roller_d()).abs() < 1e-9
            && self.pack_outer_r() + 1e-9 >= self.wing_radius() * 0.9
            && self.pcd() > self.inner_race_d() + self.roller_len()
            && self.cage_od() + 1e-9 < self.hub_deck_od()
            && (self.plate_bore() - (self.inner_race_d() + self.fit_running_mm)).abs() < 1e-9
            && self.axle_flange_d() + 1e-9 >= self.cage_od()
            && self.axle_flange_d() + 1e-9 < self.hub_deck_od()
            && (self.cage_pocket() - (self.roller_d() + self.fit_running_mm)).abs() < 1e-9
            && self.fence_h() + 1e-9 < self.pack_h()
            && self.fence_h() + 1.2 <= self.pack_h() + 1e-9
            && self.top_load_pocket() + 1e-9 > self.cage_pocket()
            && self.window_w() + 1e-9 > self.cage_pocket()
            && self.mouth_w() + 1e-9 > self.window_w()
            && self.window_floor() + 1e-9 > self.race_z()
            && self.funnel_h() + 1e-9 < self.fence_h()
            && self.land_len() + 1e-9 >= self.nozzle_mm * 6.0
            && self.land_len() + 1e-9 < self.roller_len() * 0.35
            && self.roller_end_d() + 1e-9 < self.roller_d()
            && self.crown_drop() + 1e-9 >= 0.30
            && self.roller_bore_d() + 1e-9 >= self.nozzle_mm * 6.0
            && (self.roller_d() * 0.5 - self.roller_bore_d() * 0.5) > 2.5
            && self.witness_d() + 1e-9 >= self.nozzle_mm * 6.0
            && self.cage_id() + 1e-9 > self.plate_bore()
            && self.cage_rim() + 1e-9 >= self.wall() * 2.0
            && self.assemble_ok()
            && self.lock_flat_x() + 1e-9 < self.inner_race_d() * 0.5
            && self.race_id() + 1e-9 > self.base_boss_d()
            && self.race_id() + 1e-9 < self.axle_flange_d()
            && self.cage_id() + 1e-9 >= self.race_id()
            && self.post_circle_r() + self.pad_d() * 0.5 + 1e-9 >= self.race_id() * 0.5
            && axis0[2].abs() < 1e-9
            && (axis0[0] - 1.0).abs() < 1e-9
            && self.cage_od() * 0.5 + 1e-9 >= self.pack_outer_r()
            && self.captured_ok()
    }
    fn helix_ok(&self) -> bool {
        self.helix_deg >= 45.0 && self.helix_stations >= 2
    }
    fn scale_ok(&self) -> bool {
        self.scale > 0.0
            && self.scale <= self.max_scale + 1e-9
            && self.fits_x2d_at_max()
            && self.printer.bed_mm[2] >= 260.0
    }
    fn print_flat_ok(&self) -> bool {
        self.journal_h() <= self.axle_flange_d() && self.rotor_print_h() > self.journal_h()
    }
    fn stack_ok(&self) -> bool {
        (self.race_z() - self.base_h()).abs() < 1e-9
            && (self.cage_z() - self.race_z()).abs() < 1e-9
            && (self.plate_z() - (self.race_z() + self.pack_h() + self.thrust_float)).abs() < 1e-9
            && (self.hub_z() - self.plate_z()).abs() < 1e-9
            && (self.groove_z() - (self.plate_z() + self.hub_deck_h() + self.thrust_float)).abs()
                < 1e-9
            && (self.retainer_z() - self.groove_z()).abs() < 1e-9
            && (self.z_mid() - (self.race_z() + self.pack_h() * 0.5)).abs() < 1e-9
            && self.fence_h() + 1e-9 < self.pack_h()
            && self.fence_h() + 1.2 <= self.pack_h() + 1e-9
            && self.bed_relief_h() + 1e-9 < self.hub_deck_h()
            && self.bed_relief_h() + 1e-9 < self.retainer_h()
            && (self.hub_h() - self.hub_deck_h()).abs() < 1e-9
            && self.hub_deck_od() + 1e-9 > self.axle_flange_d()
            && self.hub_deck_od() + 1e-9 >= self.wing_radius() * 2.0
            && (self.blade_root_z() - (self.plate_z() + self.hub_deck_h())).abs() < 1e-9
            && (self.pack_h() - self.roller_d()).abs() < 1e-9
            && self.hub_deck_h() + 1e-9 >= 3.2
            && self.captured_ok()
            && self.root_blend_h() + self.tip_taper_h() + 1e-9 < self.wing_h()
            && self.tip_chord() + 1e-9 < self.chord_tip()
            && self.root_scale() > 1.0
            && self.base_boss_d() + 1e-9 < self.hub_deck_od()
            && self.pack_outer_r() + 1e-9 >= self.wing_radius() * 0.9
            && self.cage_id() + 1e-9 > self.plate_bore()
            && self.assemble_ok()
            && self.race_id() + 1e-9 > self.base_boss_d()
            && self.race_id() + 1e-9 < self.axle_flange_d()
            && self.cage_id() + 1e-9 >= self.race_id()
            && self.post_circle_r() + self.pad_d() * 0.5 + 1e-9 >= self.race_id() * 0.5
    }
    fn assembly_component_count(&self) -> usize {
        3 + self.roller_count
    }
    fn assembly_joint_count(&self) -> usize {
        2 + self.roller_count
    }
    fn sanity_ok(&self) -> bool {
        self.base_envelope() / self.rotor_d() <= 1.55
            && self.wing_h() + 1e-9 >= self.chord_root() * 2.5
            && self.solidity() >= 0.24
            && self.solidity() <= 0.45
    }
    fn estimated_solid_cm3(&self) -> f64 {
        let plate = std::f64::consts::PI
            * ((self.hub_deck_od() * 0.5).powi(2) - (self.plate_bore() * 0.5).powi(2))
            * self.hub_deck_h();
        let wings = (self.wing_count as f64)
            * 0.62
            * ((self.chord_root() + self.chord_tip()) * 0.5)
            * self.wing_thick()
            * self.wing_h();
        let stator = std::f64::consts::PI * (self.base_boss_d() * 0.5).powi(2) * self.base_h()
            + (self.post_count as f64) * self.rib_w() * self.post_circle_r() * self.base_h()
            + std::f64::consts::PI
                * ((self.axle_flange_d() * 0.5).powi(2) - (self.race_id() * 0.5).powi(2))
                * self.base_h()
            + std::f64::consts::PI
                * ((self.cage_od() * 0.5).powi(2) - (self.cage_id() * 0.5).powi(2))
                * self.fence_h()
            + std::f64::consts::PI * (self.inner_race_d() * 0.5).powi(2) * self.journal_h();
        let rollers = (self.roller_count as f64)
            * std::f64::consts::PI
            * ((self.roller_d() * 0.5).powi(2) - (self.roller_bore_d() * 0.5).powi(2))
            * self.roller_len();
        let retainer = (std::f64::consts::PI * (self.retainer_od() * 0.5).powi(2)
            - (self.inner_race_d() * 0.5).powi(2))
            * self.retainer_h();
        (plate + wings + stator + rollers + retainer) / 1000.0
    }
    fn estimated_pla_cm3(&self) -> f64 {
        let all = self.estimated_solid_cm3();
        let petg = self.estimated_petg_cm3();
        (all - petg).max(0.0)
    }
    fn estimated_petg_cm3(&self) -> f64 {
        let rollers = (self.roller_count as f64)
            * std::f64::consts::PI
            * ((self.roller_d() * 0.5).powi(2) - (self.roller_bore_d() * 0.5).powi(2))
            * self.roller_len();
        let retainer = (std::f64::consts::PI * (self.retainer_od() * 0.5).powi(2)
            - (self.inner_race_d() * 0.5).powi(2))
            * self.retainer_h();
        (rollers + retainer) / 1000.0
    }
    fn estimated_print_mass_g(&self) -> f64 {
        let factor = self.filament.print_volume_factor;
        self.estimated_pla_cm3() * 1.24 * factor + self.estimated_petg_cm3() * 1.27 * factor
    }
    fn estimated_filament_usd(&self) -> f64 {
        self.estimated_print_mass_g() / 1000.0 * self.filament.price_usd_per_kg
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

#[allow(dead_code)]
struct Built {
    stator_id: u64,
    rotor_id: u64,
    roller_ids: Vec<u64>,
    retainer_id: u64,
    assembly_ok: bool,
    assembly_detail: String,
    drawing_ok: bool,
    drawing_detail: String,
    blank_ok: bool,
    blank_detail: String,
}

pub fn run(call: &mut impl FnMut(&str, Value) -> Result<Value, String>) -> Result<Report, String> {
    let spec = load_spec()?;
    if !spec.fits_ok() || !spec.stack_ok() {
        return Ok(Report::fail(
            &spec.id,
            "fits",
            "running/slip/friction stack or axial float is not a 0.4 mm PLA set",
        ));
    }

    let mut step = 0u32;
    let mut call = |name: &str, arguments: Value| {
        step += 1;
        call(name, arguments).map_err(|error| format!("step {step} {name}: {error}"))
    };

    call("cad_new_project", json!({}))?;
    let blank_detail = require_blank_document(&mut call)?;
    call(
        "cad_set_document_name",
        json!({ "name": spec.document_name }),
    )?;

    let stator_id = build_stator(&mut call, &spec)?;
    require_solid_ok(&mut call, "stator")?;
    let rotor_id = build_rotor(&mut call, &spec, &[stator_id])?;
    require_solid_ok(&mut call, "rotor")?;
    let mut known = vec![stator_id, rotor_id];
    let roller_ids = place_rollers(&mut call, &spec, &known)?;
    known.extend(roller_ids.iter().copied());
    require_solid_ok(&mut call, "rollers")?;
    let retainer_id = build_retainer(&mut call, &spec, &known)?;
    require_solid_ok(&mut call, "kit")?;

    let (assembly_ok, assembly_detail) = match form_assembly(
        &mut call,
        &spec,
        stator_id,
        rotor_id,
        &roller_ids,
        retainer_id,
    ) {
        Ok(detail) => (true, detail),
        Err(error) => (false, error),
    };
    let (drawing_ok, drawing_detail) = match make_assembly_drawing(&mut call, &spec) {
        Ok(()) => (
            true,
            "A3 assembly sheet with fit / scale / print / BOM notes".to_string(),
        ),
        Err(error) => (false, error),
    };

    call(
        "cad_set_focus",
        json!({ "focus": "print", "explicit": true }),
    )?;
    for (id, preset) in [
        (stator_id, spec.materials.orange.as_str()),
        (rotor_id, spec.materials.glow.as_str()),
        (retainer_id, spec.materials.petg.as_str()),
    ] {
        call(
            "set_body_appearance",
            json!({ "body_id": id, "preset_id": preset }),
        )?;
    }
    for id in &roller_ids {
        call(
            "set_body_appearance",
            json!({ "body_id": id, "preset_id": spec.materials.petg }),
        )?;
    }
    let hide_detail = hide_construction(&mut call)?;
    let preflight = call("solid_export_preflight", json!({}))?;
    let scene = call("solid_scene", json!({}))?;
    let document = call("cad_document", json!({}))?;
    let assembly = call("assembly_document", json!({})).unwrap_or(json!({}));
    let plate_exports = export_print_plates(
        &mut call,
        &spec,
        stator_id,
        rotor_id,
        &roller_ids,
        retainer_id,
    )?;
    Ok(grade(
        &spec,
        &scene,
        &document,
        &assembly,
        &preflight,
        &plate_exports,
        Built {
            stator_id,
            rotor_id,
            roller_ids,
            retainer_id,
            assembly_ok,
            assembly_detail,
            drawing_ok,
            drawing_detail,
            blank_ok: true,
            blank_detail: format!("{blank_detail}; {hide_detail}"),
        },
    ))
}

fn export_print_plates(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    stator_id: u64,
    rotor_id: u64,
    roller_ids: &[u64],
    retainer_id: u64,
) -> Result<Vec<(String, Value)>, String> {
    layout_print_plate(call, spec, stator_id, rotor_id, roller_ids, retainer_id)?;
    let mut body_ids = vec![stator_id, rotor_id, retainer_id];
    body_ids.extend(roller_ids.iter().copied());
    let name = spec
        .print_plates
        .first()
        .map(String::as_str)
        .unwrap_or("01-kit");
    let exported = call(
        "solid_export_3mf",
        json!({
            "slicer_target": spec.slicer_target,
            "include_appearance": true,
            "body_ids": body_ids
        }),
    )?;
    Ok(vec![(name.to_string(), exported)])
}

fn move_bodies(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    body_ids: &[u64],
    translation: [f64; 3],
) -> Result<(), String> {
    transform_bodies(
        call,
        body_ids,
        translation,
        [0.0, 0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
    )
}

fn transform_bodies(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    body_ids: &[u64],
    translation: [f64; 3],
    rotation: [f64; 4],
    pivot: [f64; 3],
) -> Result<(), String> {
    if body_ids.is_empty() {
        return Ok(());
    }
    call(
        "solid_move_copy",
        json!({
            "body_ids": body_ids,
            "translation": translation,
            "rotation": rotation,
            "pivot": pivot,
            "copy": false
        }),
    )?;
    Ok(())
}

fn quat_axis_angle(axis: [f64; 3], deg: f64) -> [f64; 4] {
    let n = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    let a = if n < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [axis[0] / n, axis[1] / n, axis[2] / n]
    };
    let half = deg.to_radians() * 0.5;
    let s = half.sin();
    [a[0] * s, a[1] * s, a[2] * s, half.cos()]
}

fn cut_u_windows(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    stator_id: u64,
) -> Result<(), String> {
    for index in 0..spec.roller_count {
        let [x, y] = spec.roller_xy(index);
        let angle = spec.roller_angle_deg(index);
        cut_oriented_slot(
            call,
            stator_id,
            spec.window_floor(),
            [x, y],
            spec.pocket_len(),
            spec.window_w(),
            angle,
            spec.fence_h() + 0.4,
            &format!("U-window {index}"),
        )?;
        let mouth_z = spec.race_z() + spec.fence_h() - spec.funnel_h();
        cut_oriented_slot(
            call,
            stator_id,
            mouth_z,
            [x, y],
            spec.pocket_len() + spec.nozzle_mm * 2.0,
            spec.mouth_w(),
            angle,
            spec.funnel_h() + 0.4,
            &format!("U-window mouth {index}"),
        )?;
    }
    Ok(())
}

fn cut_oriented_slot(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    stator_id: u64,
    z: f64,
    center: [f64; 2],
    length: f64,
    width: f64,
    angle_deg: f64,
    depth: f64,
    label: &str,
) -> Result<(), String> {
    let deck = offset_xy(call, z)?;
    begin_datum(call, deck)?;
    add_oriented_rect(call, center, length, width, angle_deg)?;
    let sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": depth },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [stator_id]
            }),
        )?,
        label,
    )?;
    Ok(())
}

fn cut_witness_holes(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    stator_id: u64,
    known: &[u64],
) -> Result<(), String> {
    let pocket_outer = spec.pcd() * 0.5 + spec.pocket_len() * 0.5;
    let outer = spec.cage_od() * 0.5;
    let center_r = (pocket_outer + outer) * 0.5;
    let length = (outer - pocket_outer).max(spec.keeper()) + 4.0;
    let mut seen = known.to_vec();
    seen.push(stator_id);
    for index in 0..spec.roller_count {
        let tool_id = place_radial_cylinder_at(
            call,
            spec,
            spec.witness_d(),
            length,
            center_r,
            index,
            &seen,
            &format!("witness hole {index}"),
        )?;
        require_clean(
            call(
                "solid_combine",
                json!({
                    "target_body_id": stator_id,
                    "tool_body_ids": [tool_id],
                    "operation": "cut",
                    "keep_tools": false
                }),
            )?,
            &format!("witness hole cut {index}"),
        )?;
    }
    Ok(())
}

#[allow(dead_code)]
fn place_radial_cylinder(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    diameter: f64,
    length: f64,
    index: usize,
    known: &[u64],
    label: &str,
) -> Result<u64, String> {
    place_radial_cylinder_at(
        call,
        spec,
        diameter,
        length,
        spec.pcd() * 0.5,
        index,
        known,
        label,
    )
}

fn place_radial_cylinder_at(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    diameter: f64,
    length: f64,
    center_r: f64,
    index: usize,
    known: &[u64],
    label: &str,
) -> Result<u64, String> {
    let x0 = center_r - length * 0.5;
    let deck = offset_yz(call, x0)?;
    begin_datum(call, deck)?;
    add_circle(call, [0.0, spec.z_mid()], diameter)?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": length },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        label,
    )?;
    let id = newest_body_id(&update, known)?;
    let theta = spec.roller_angle_deg(index);
    if theta.abs() > 1e-9 {
        transform_bodies(
            call,
            &[id],
            [0.0, 0.0, 0.0],
            quat_axis_angle([0.0, 0.0, 1.0], theta),
            [0.0, 0.0, spec.z_mid()],
        )?;
    }
    Ok(id)
}

fn stand_roller(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    roller_id: u64,
    index: usize,
) -> Result<(), String> {
    let theta = spec.roller_angle_deg(index).to_radians();
    let [x, y] = spec.roller_xy(index);
    // −90° about e_θ takes e_r onto +Z so the roller prints standing.
    transform_bodies(
        call,
        &[roller_id],
        [0.0, 0.0, 0.0],
        quat_axis_angle([-theta.sin(), theta.cos(), 0.0], -90.0),
        [x, y, spec.z_mid()],
    )
}

fn layout_print_plate(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    stator_id: u64,
    rotor_id: u64,
    roller_ids: &[u64],
    retainer_id: u64,
) -> Result<(), String> {
    let gap = 10.0;
    let rotor_r = spec.rotor_d().max(spec.hub_deck_od()) * 0.5;
    let stator_r = spec.base_envelope().max(spec.axle_flange_d()) * 0.5;
    let ret_r = spec.retainer_od() * 0.5;
    let col_x = rotor_r + gap + stator_r;
    move_bodies(
        call,
        &[rotor_id],
        [-rotor_r - gap * 0.5, 0.0, -spec.plate_z()],
    )?;
    move_bodies(call, &[stator_id], [col_x, stator_r + gap * 0.5, 0.0])?;
    let roll_pitch = spec.roller_d() + 4.0;
    let slot_x = col_x + stator_r + gap + spec.roller_d() * 0.5;
    for (index, roller_id) in roller_ids.iter().enumerate() {
        stand_roller(call, spec, *roller_id, index)?;
        let [x, y] = spec.roller_xy(index);
        let slot_y = -(stator_r * 0.4) + index as f64 * roll_pitch;
        move_bodies(
            call,
            &[*roller_id],
            [
                slot_x - x,
                slot_y - y,
                -(spec.z_mid() - spec.roller_len() * 0.5),
            ],
        )?;
    }
    move_bodies(
        call,
        &[retainer_id],
        [col_x, -(stator_r + gap + ret_r), -spec.retainer_z()],
    )?;
    Ok(())
}

fn build_stator(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<u64, String> {
    begin_xy(call)?;
    add_circle(call, [0.0, 0.0], spec.base_boss_d())?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.base_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "stator hub",
    )?;
    let stator_id = newest_body_id(&update, &[])?;
    for index in 0..spec.post_count {
        let [px, py] = spec.post_xy(index);
        let angle = 360.0 / spec.post_count.max(1) as f64 * index as f64;
        begin_xy(call)?;
        add_oriented_rect(
            call,
            [px * 0.5, py * 0.5],
            spec.post_circle_r(),
            spec.rib_w(),
            angle,
        )?;
        let rib = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": rib,
                    "profile_indices": [0],
                    "operation": "join",
                    "extent": { "type": "distance", "distance": spec.base_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": [stator_id]
                }),
            )?,
            &format!("stator rib {index}"),
        )?;
        begin_xy(call)?;
        add_circle(call, [px, py], spec.pad_d())?;
        let pad = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": pad,
                    "profile_indices": [0],
                    "operation": "join",
                    "extent": { "type": "distance", "distance": spec.base_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": [stator_id]
                }),
            )?,
            &format!("stator pad {index}"),
        )?;
    }
    begin_xy(call)?;
    add_circle(call, [0.0, 0.0], spec.axle_flange_d())?;
    add_circle(call, [0.0, 0.0], spec.race_id())?;
    let race = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": race,
                "profile_indices": [0],
                "operation": "join",
                "extent": { "type": "distance", "distance": spec.base_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [stator_id]
            }),
        )?,
        "stator race ring",
    )?;

    let race_deck = offset_xy(call, spec.race_z())?;
    begin_datum(call, race_deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.journal_d())?;
    let journal = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": journal,
                "profile_indices": [0],
                "operation": "join",
                "extent": { "type": "distance", "distance": spec.journal_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [stator_id]
            }),
        )?,
        "stator journal",
    )?;
    begin_datum(call, race_deck)?;
    add_circle(call, [0.0, 0.0], spec.cage_od())?;
    add_circle(call, [0.0, 0.0], spec.cage_id())?;
    let fence = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": fence,
                "profile_indices": [0],
                "operation": "join",
                "extent": { "type": "distance", "distance": spec.fence_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [stator_id]
            }),
        )?,
        "stator fence",
    )?;

    let groove_deck = offset_xy(call, spec.groove_z())?;
    begin_datum(call, groove_deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.journal_d() + 4.0)?;
    add_circle(call, [0.0, 0.0], spec.groove_d())?;
    let groove = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": groove,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.groove_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [stator_id]
            }),
        )?,
        "stator snap groove",
    )?;

    begin_datum(call, groove_deck)?;
    add_oriented_rect(
        call,
        [spec.lock_flat_x() + spec.journal_d(), 0.0],
        spec.journal_d() * 2.0,
        spec.journal_d() * 2.0,
        0.0,
    )?;
    let flat = finish_sketch(call)?;
    let neck_h = spec.groove_h() + spec.tip_h();
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": flat,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": neck_h },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [stator_id]
            }),
        )?,
        "stator D-flat",
    )?;
    cut_u_windows(call, spec, stator_id)?;
    cut_witness_holes(call, spec, stator_id, &[stator_id])?;
    Ok(stator_id)
}

fn blade_loft_stations(spec: &Spec) -> Vec<(f64, f64, f64)> {
    let stations = spec.helix_stations.max(2);
    let span = spec.wing_h() - spec.root_blend_h() - spec.tip_taper_h();
    let mut out = vec![(
        spec.blade_loft_z(),
        0.0,
        spec.chord_root() * spec.root_scale(),
    )];
    for station in 0..stations {
        let t = station as f64 / (stations - 1) as f64;
        let chord = spec.chord_root() * (1.0 - t) + spec.chord_tip() * t;
        let z = spec.blade_loft_z() + spec.root_blend_h() + span * t;
        out.push((z, t, chord));
    }
    out.push((spec.blade_loft_z() + spec.wing_h(), 1.0, spec.tip_chord()));
    out
}

fn build_rotor(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<u64, String> {
    let z0 = spec.plate_z();
    let deck = offset_xy(call, z0)?;
    begin_datum(call, deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.hub_deck_od())?;
    add_circle(call, [0.0, 0.0], spec.plate_bore())?;
    let seat = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": seat,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.hub_deck_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "rotor root plate",
    )?;
    let rotor_id = newest_body_id(&update, known)?;

    for index in 0..spec.wing_count {
        begin_datum(call, deck.clone())?;
        add_airfoil(
            call,
            spec.helix_center(index, 0.0),
            spec.helix_azimuth_deg(index, 0.0) + 90.0,
            spec.chord_root(),
            spec.airfoil_t_c,
            spec.airfoil_xt_c,
            spec.airfoil_le_index,
            spec.airfoil_stations,
            spec.te_min(),
        )?;
        let stump = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": stump,
                    "profile_indices": [0],
                    "operation": "join",
                    "extent": { "type": "distance", "distance": spec.hub_deck_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": [rotor_id]
                }),
            )?,
            &format!("blade root base {index}"),
        )?;
        let mut sections = Vec::new();
        for (z, t, chord) in blade_loft_stations(spec) {
            let loft_deck = offset_xy(call, z)?;
            begin_datum(call, loft_deck)?;
            add_airfoil(
                call,
                spec.helix_center(index, t),
                spec.helix_azimuth_deg(index, t) + 90.0,
                chord,
                spec.airfoil_t_c,
                spec.airfoil_xt_c,
                spec.airfoil_le_index,
                spec.airfoil_stations,
                spec.te_min(),
            )?;
            sections.push(finish_sketch(call)?);
        }
        require_clean(
            call(
                "solid_loft",
                json!({
                    "sections": sections.iter().map(|name| json!({
                        "sketch_name": name,
                        "profile_index": 0
                    })).collect::<Vec<_>>(),
                    "ruled": false,
                    "operation": "join",
                    "target_body_ids": [rotor_id],
                    "continuity": "g1"
                }),
            )?,
            &format!("helical blade {index}"),
        )?;
    }
    cut_bed_relief_circle(
        call,
        spec,
        spec.plate_z(),
        spec.plate_bore() + spec.bed_relief_d(),
        rotor_id,
        "plate bore bed lead-in",
    )?;
    Ok(rotor_id)
}

fn place_crowned_roller(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    index: usize,
    known: &[u64],
    label: &str,
) -> Result<u64, String> {
    let mid_r = spec.roller_d() * 0.5;
    let end_r = spec.roller_end_d() * 0.5;
    let bore_r = spec.roller_bore_d() * 0.5;
    let half_l = spec.roller_len() * 0.5;
    let half_land = spec.land_len() * 0.5;
    let x0 = spec.pcd() * 0.5;
    let deck = offset_xy(call, spec.z_mid())?;
    begin_datum(call, deck)?;
    add_poly(
        call,
        &[
            [x0 - half_l, bore_r],
            [x0 - half_l, end_r],
            [x0 - half_land, mid_r],
            [x0 + half_land, mid_r],
            [x0 + half_l, end_r],
            [x0 + half_l, bore_r],
            [x0 - half_l, bore_r],
        ],
        true,
    )?;
    let sketch = finish_sketch(call)?;
    call(
        "cad_set_focus",
        json!({ "focus": "solid", "explicit": true }),
    )?;
    let update = require_clean(
        call(
            "solid_revolve",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "axis_origin": {"x": 0.0, "y": 0.0},
                "axis_direction": {"x": 1.0, "y": 0.0},
                "axis_line_entity_id": null,
                "angle_deg": 360.0,
                "flip": false,
                "operation": "new_body",
                "target_body_ids": []
            }),
        )?,
        label,
    )?;
    let id = newest_body_id(&update, known)?;
    let theta = spec.roller_angle_deg(index);
    if theta.abs() > 1e-9 {
        transform_bodies(
            call,
            &[id],
            [0.0, 0.0, 0.0],
            quat_axis_angle([0.0, 0.0, 1.0], theta),
            [0.0, 0.0, spec.z_mid()],
        )?;
    }
    Ok(id)
}

fn place_rollers(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<Vec<u64>, String> {
    let mut seen = known.to_vec();
    let mut roller_ids = Vec::new();
    for index in 0..spec.roller_count {
        let id =
            place_crowned_roller(call, spec, index, &seen, &format!("crowned roller {index}"))?;
        seen.push(id);
        roller_ids.push(id);
    }
    Ok(roller_ids)
}

fn build_retainer(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    known: &[u64],
) -> Result<u64, String> {
    let deck = offset_xy(call, spec.retainer_z())?;
    begin_datum(call, deck.clone())?;
    add_circle(call, [0.0, 0.0], spec.retainer_od())?;
    let sketch = finish_sketch(call)?;
    let update = require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "new_body",
                "extent": { "type": "distance", "distance": spec.retainer_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": []
            }),
        )?,
        "retainer",
    )?;
    let retainer_id = newest_body_id(&update, known)?;
    begin_datum(call, deck.clone())?;
    add_d_profile(call, spec.retainer_d_hole(), spec.retainer_flat_x())?;
    let bore = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": bore,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.retainer_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [retainer_id]
            }),
        )?,
        "retainer D-hole",
    )?;
    begin_datum(call, deck.clone())?;
    add_oriented_rect(
        call,
        [-spec.retainer_od() * 0.5, 0.0],
        spec.retainer_od(),
        spec.clip_mouth(),
        0.0,
    )?;
    let gap = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": gap,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.retainer_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [retainer_id]
            }),
        )?,
        "retainer E-clip mouth",
    )?;
    let hole = spec.retainer_d_hole();
    let od = spec.retainer_od();
    let tab_w = spec.clip_tab_w();
    let tab_l = spec.clip_tab_l();
    let mouth = spec.clip_mouth();
    for sign in [1.0, -1.0] {
        begin_datum(call, deck.clone())?;
        add_oriented_rect(
            call,
            [-(od + hole) * 0.25, sign * (mouth * 0.5 + tab_w * 0.35)],
            tab_l,
            tab_w,
            0.0,
        )?;
        let tab = finish_sketch(call)?;
        require_clean(
            call(
                "solid_extrude",
                json!({
                    "sketch_name": tab,
                    "profile_indices": [0],
                    "operation": "join",
                    "extent": { "type": "distance", "distance": spec.retainer_h() },
                    "taper_angle_deg": 0.0,
                    "flip": false,
                    "target_body_ids": [retainer_id]
                }),
            )?,
            &format!(
                "retainer finger tab {}",
                if sign > 0.0 { "plus" } else { "minus" }
            ),
        )?;
    }
    cut_bed_relief_circle(
        call,
        spec,
        spec.retainer_z(),
        spec.retainer_d_hole() + spec.bed_relief_d(),
        retainer_id,
        "retainer D-hole bed lead-in",
    )?;
    Ok(retainer_id)
}

fn purge_orphan_components(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
) -> Result<(), String> {
    let scene = call("solid_scene", json!({}))?;
    let live: std::collections::HashSet<u64> = scene["bodies"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|body| body["id"].as_u64())
        .collect();
    let mut document = call("assembly_document", json!({}))?;
    let Some(defs) = document["component_structure"]["definitions"]
        .as_array()
        .cloned()
    else {
        return Ok(());
    };
    let keep: std::collections::HashSet<u64> = defs
        .iter()
        .filter(|definition| {
            definition["body_ids"]
                .as_array()
                .map(|ids| {
                    !ids.is_empty()
                        && ids
                            .iter()
                            .all(|id| id.as_u64().is_some_and(|id| live.contains(&id)))
                })
                .unwrap_or(false)
        })
        .filter_map(|definition| definition["id"].as_u64())
        .collect();
    if let Some(definitions) = document["component_structure"]["definitions"].as_array_mut() {
        definitions.retain(|definition| {
            definition["id"]
                .as_u64()
                .is_some_and(|id| keep.contains(&id))
        });
    }
    if let Some(occurrences) = document["component_structure"]["occurrences"].as_array_mut() {
        occurrences.retain(|occurrence| {
            occurrence["component_id"]
                .as_u64()
                .is_some_and(|id| keep.contains(&id))
        });
    }
    call("assembly_set_document", document)?;
    Ok(())
}

fn form_assembly(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    stator_id: u64,
    rotor_id: u64,
    roller_ids: &[u64],
    retainer_id: u64,
) -> Result<String, String> {
    call(
        "cad_set_focus",
        json!({ "focus": "assembly", "explicit": true }),
    )?;
    let _ = call("cad_set_workspace", json!({ "workspace": "assembly" }));
    purge_orphan_components(call)?;
    let mut parts: Vec<(String, Vec<u64>)> = vec![
        ("stator".to_string(), vec![stator_id]),
        ("rotor".to_string(), vec![rotor_id]),
    ];
    for (index, roller_id) in roller_ids.iter().enumerate() {
        parts.push((format!("roller_{index}"), vec![*roller_id]));
    }
    parts.push(("retainer".to_string(), vec![retainer_id]));
    let mut stator_occurrence_id = None;
    for (name, body_ids) in &parts {
        let created = call(
            "assembly_create_component",
            json!({
                "name": name,
                "body_ids": body_ids,
                "absorb_promoted_bodies": true
            }),
        )?;
        let component_id = created["id"]
            .as_u64()
            .or_else(|| created["component"]["id"].as_u64())
            .ok_or_else(|| format!("component {name} missing id: {created}"))?;
        let document = call("assembly_document", json!({}))?;
        let occurrence_id = authored_occurrence_id(&document, component_id)
            .ok_or_else(|| format!("component {name} has no root occurrence"))?;
        if name == "stator" {
            stator_occurrence_id = Some(occurrence_id);
        }
    }
    if let Some(occurrence_id) = stator_occurrence_id {
        call(
            "assembly_set_occurrence_grounded",
            json!({ "occurrence_id": occurrence_id, "grounded": true }),
        )?;
    }
    let _ = call(
        "assembly_set_grounded_body",
        json!({ "body_id": stator_id }),
    );
    let scene = call("solid_scene", json!({}))?;
    let plate_spin = axis_connector_at(
        &scene,
        rotor_id,
        [0.0, 0.0],
        spec.plate_z() + spec.hub_deck_h(),
        spec.plate_bore() * 0.5,
    )
    .ok_or_else(|| "no on-axis plate bore for rotor_spin".to_string())?;
    create_stable_joint(
        call,
        "rotor_spin",
        "revolute",
        journal_axis_at(
            &scene,
            stator_id,
            connector_z(&plate_spin).unwrap_or(spec.plate_z() + spec.hub_deck_h()),
            spec.journal_d() * 0.5,
        )
        .ok_or_else(|| "no on-axis stator journal for rotor_spin".to_string())?,
        plate_spin,
        stator_id,
    )?;
    for (index, roller_id) in roller_ids.iter().enumerate() {
        let [x, y] = spec.roller_xy(index);
        let z = spec.z_mid();
        let axis = spec.roller_axis(index);
        create_stable_joint(
            call,
            &format!("roller_{index}_spin"),
            "revolute",
            radial_connector_at(&scene, stator_id, [x, y], z, spec.witness_d() * 0.5, axis)
                .ok_or_else(|| format!("no stator witness radial axis for roller {index}"))?,
            radial_connector_at(&scene, *roller_id, [x, y], z, spec.roller_d() * 0.5, axis)
                .ok_or_else(|| format!("no roller radial axis for roller {index}"))?,
            stator_id,
        )?;
    }
    create_stable_joint(
        call,
        "retainer_sit",
        "rigid",
        axis_connector_at(
            &scene,
            stator_id,
            [0.0, 0.0],
            spec.retainer_z(),
            spec.groove_d() * 0.5,
        )
        .or_else(|| {
            axis_connector_at(
                &scene,
                stator_id,
                [0.0, 0.0],
                spec.retainer_z(),
                spec.journal_d() * 0.5,
            )
        })
        .ok_or_else(|| "no on-axis stator groove for retainer_sit".to_string())?,
        axis_connector_at(
            &scene,
            retainer_id,
            [0.0, 0.0],
            spec.retainer_z(),
            spec.retainer_od() * 0.5,
        )
        .ok_or_else(|| "no on-axis retainer E-clip for retainer_sit".to_string())?,
        stator_id,
    )?;
    if let Some(occurrence_id) = stator_occurrence_id {
        let _ = call(
            "assembly_set_occurrence_grounded",
            json!({ "occurrence_id": occurrence_id, "grounded": true }),
        );
    }
    let document = call("assembly_document", json!({}))?;
    let defs = document["component_structure"]["definitions"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let occs = document["component_structure"]["occurrences"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let joints = document["joints"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    if defs != spec.assembly_component_count() || occs != spec.assembly_component_count() {
        return Err(format!(
            "expected {} linked parts / occurrences, got {defs} components / {occs} occurrences",
            spec.assembly_component_count()
        ));
    }
    if joints < spec.assembly_joint_count() {
        return Err(format!(
            "expected ≥{} joints (rotor + rollers + retainer), got {joints}",
            spec.assembly_joint_count()
        ));
    }
    require_linked_solution(call)?;
    Ok(format!(
        "{defs} linked parts, {joints} joints; one stator; radial-axis pack under the blade roots; U-window fence; clocked E-clip in a groove; crowned rollers spin about e_r"
    ))
}

fn create_stable_joint(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    name: &str,
    kind: &str,
    connector_a: Value,
    connector_b: Value,
    grounded_body_id: u64,
) -> Result<(), String> {
    create_stable_joint_flips(
        call,
        name,
        kind,
        connector_a,
        connector_b,
        grounded_body_id,
        [false, true],
    )
}

fn create_stable_joint_flips(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    name: &str,
    kind: &str,
    connector_a: Value,
    connector_b: Value,
    grounded_body_id: u64,
    flips: [bool; 2],
) -> Result<(), String> {
    let mut last_error = String::new();
    for flipped in flips {
        let created = call(
            "assembly_create_joint",
            json!({
                "name": name,
                "kind": kind,
                "connector_a": connector_a,
                "connector_b": connector_b,
                "flipped": flipped,
                "grounded_body_id": grounded_body_id
            }),
        );
        match created {
            Ok(joint) => match require_linked_solution(call) {
                Ok(()) => return Ok(()),
                Err(error) => {
                    if let Some(joint_id) = joint["id"]
                        .as_u64()
                        .or_else(|| joint["joint"]["id"].as_u64())
                    {
                        let _ = call("assembly_delete_joint", json!({ "joint_id": joint_id }));
                    }
                    last_error =
                        format!("{name} yanked or failed to solve (flipped={flipped}): {error}");
                }
            },
            Err(error) => last_error = format!("{name} (flipped={flipped}): {error}"),
        }
    }
    Err(last_error)
}

fn require_linked_solution(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
) -> Result<(), String> {
    let solution = call("assembly_solution", json!({}))?;
    if solution["solved"] != true {
        return Err(format!(
            "assembly_solution not solved: {}",
            solution["diagnostics"]
        ));
    }
    let poses = solution["occurrence_poses"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for pose in poses {
        let translation = xyz(&pose["translation"]).unwrap_or([0.0, 0.0, 0.0]);
        let yank = translation[0].hypot(translation[1]).hypot(translation[2]);
        if yank > 8.0 {
            return Err(format!(
                "occurrence {} yanked {:.1} mm (t=[{:.2},{:.2},{:.2}]) — connectors are off-axis or flipped",
                pose["occurrence_id"],
                yank,
                translation[0],
                translation[1],
                translation[2]
            ));
        }
    }
    Ok(())
}

fn make_assembly_drawing(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
) -> Result<(), String> {
    call(
        "cad_set_focus",
        json!({ "focus": "drawing", "explicit": true }),
    )?;
    call(
        "cad_drawing_create_sheet",
        json!({
            "standard": "iso",
            "format": "a3",
            "orientation": "landscape",
            "title": spec.title,
            "drawing_number": "PK-VAWT-001",
            "revision": "A"
        }),
    )?;
    call("cad_drawing_auto_layout", json!({}))?;
    let notes = [
        (
            [18.0, 28.0],
            format!(
                "ASSEMBLY  scale={:.2} (1.0 = {} max)  PLA+PETG  nozzle {:.1} mm",
                spec.scale, spec.printer.name, spec.nozzle_mm
            ),
        ),
        (
            [18.0, 38.0],
            format!(
                "FITS  running +{:.2}  PIP +{:.2}  slip +{:.2}  friction +{:.2}  bed lead-in {:.2}  (slicer XY hole comp = 0)",
                spec.fit_running_mm, spec.fit_pip_mm, spec.fit_slip_mm, spec.fit_friction_mm, spec.bed_relief_mm
            ),
        ),
        (
            [18.0, 48.0],
            "PRINT  one plate, laid out. Rotor STANDING on the root plate. Rollers STANDING (axis Z), assemble lying (axis radial). Others FLAT. PLA Orange stator + PLA Glow blades + PETG HF rollers/clip.".to_string(),
        ),
        (
            [18.0, 58.0],
            "GDT  one stator (thin Y-frame + race ring + keeper walls + U-window fence + flat race + constant journal). Thin plate with organic airfoil roots. Plate bore > journal pass Ø so the rotor drops on. Clocked E-clip snaps radially into an undercut groove above the plate — it does not rub the rotor. Pinch the finger tabs to remove.".to_string(),
        ),
        (
            [18.0, 68.0],
            format!(
                "BOM  stator (Y-frame + race ring + U-window fence + grooved journal) · rotor (root plate+3×{}) · {} crowned radial rollers · clocked E-clip",
                spec.airfoil, spec.roller_count
            ),
        ),
        (
            [18.0, 78.0],
            format!(
                "ROLLERS  PETG hollow Ø{:.1} × L{:.1}  bore {:.1}  crown drop {:.2}  mid land {:.1}  axis radial  PCD {:.1}  pack h={:.1}. Blades stay PLA Glow. No metal 608.",
                spec.roller_d(),
                spec.roller_len(),
                spec.roller_bore_d(),
                spec.crown_drop(),
                spec.land_len(),
                spec.pcd(),
                spec.pack_h()
            ),
        ),
    ];
    for (position, text) in notes {
        call(
            "cad_drawing_add_note",
            json!({
                "position": position,
                "text": text
            }),
        )?;
    }
    let drawing = call("cad_drawing_document", json!({}))?;
    let sheets = drawing["sheets"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    if sheets < 1 {
        return Err("drawing sheet missing".to_string());
    }
    Ok(())
}

fn authored_occurrence_id(document: &Value, component_id: u64) -> Option<u64> {
    let occurrences = document["component_structure"]["occurrences"].as_array()?;
    let matches: Vec<&Value> = occurrences
        .iter()
        .filter(|occurrence| occurrence["component_id"].as_u64() == Some(component_id))
        .collect();
    if matches.len() == 1 {
        return matches[0]["id"].as_u64();
    }
    matches
        .iter()
        .find(|occurrence| {
            !occurrence["name"]
                .as_str()
                .is_some_and(|name| name.ends_with("_1"))
        })
        .or_else(|| matches.first())
        .and_then(|occurrence| occurrence["id"].as_u64())
}

fn connector_z(connector: &Value) -> Option<f64> {
    connector["frame"]["origin"]
        .as_array()
        .and_then(|origin| origin.get(2))
        .and_then(Value::as_f64)
}

fn journal_axis_at(scene: &Value, axle_id: u64, z: f64, want_radius: f64) -> Option<Value> {
    // Prefer the journal cylinder so each revolute can sit at the partner's
    // actual edge Z. Picking the nearest journal *circle* on a short puck
    // locks cage and plate to different heights and yanks the pack.
    cylindrical_face_along(scene, axle_id, [0.0, 0.0, z], [0.0, 0.0, 1.0], want_radius).or_else(
        || circular_edge_along(scene, axle_id, [0.0, 0.0, z], [0.0, 0.0, 1.0], want_radius),
    )
}

fn axis_connector_at(
    scene: &Value,
    body_id: u64,
    xy: [f64; 2],
    z: f64,
    want_radius: f64,
) -> Option<Value> {
    circular_edge_at(scene, body_id, xy, z, want_radius).or_else(|| {
        cylindrical_face_along(
            scene,
            body_id,
            [xy[0], xy[1], z],
            [0.0, 0.0, 1.0],
            want_radius,
        )
    })
}

fn radial_connector_at(
    scene: &Value,
    body_id: u64,
    xy: [f64; 2],
    z: f64,
    want_radius: f64,
    axis: [f64; 3],
) -> Option<Value> {
    // Prefer the cylinder so both pocket and roller sit at the PCD.
    // Circular end edges are at different X for pocket_len vs roller_len,
    // and the joint canonicalizer overwrites a circular_edge frame.
    cylindrical_face_along(scene, body_id, [xy[0], xy[1], z], axis, want_radius)
        .or_else(|| circular_edge_along(scene, body_id, [xy[0], xy[1], z], axis, want_radius))
}

fn axis_dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn axis_norm(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if n < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [v[0] / n, v[1] / n, v[2] / n]
    }
}
fn axis_aligned(axis: [f64; 3], want: [f64; 3]) -> bool {
    axis_dot(axis_norm(axis), axis_norm(want)).abs() >= 0.85
}

fn perp_and_along(origin: [f64; 3], axis: [f64; 3], point: [f64; 3]) -> (f64, f64) {
    let a = axis_norm(axis);
    let d = [
        point[0] - origin[0],
        point[1] - origin[1],
        point[2] - origin[2],
    ];
    let along = axis_dot(d, a);
    let closest = [
        origin[0] + a[0] * along,
        origin[1] + a[1] * along,
        origin[2] + a[2] * along,
    ];
    let perp = (point[0] - closest[0])
        .hypot(point[1] - closest[1])
        .hypot(point[2] - closest[2]);
    (perp, along)
}

fn frame_along(point: [f64; 3], primary: [f64; 3]) -> Value {
    let p = axis_norm(primary);
    let secondary = if p[2].abs() < 0.5 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    json!({
        "origin": [point[0], point[1], point[2]],
        "primary_axis": p,
        "secondary_axis": secondary
    })
}

fn cylindrical_face_along(
    scene: &Value,
    body_id: u64,
    point: [f64; 3],
    want_axis: [f64; 3],
    want_radius: f64,
) -> Option<Value> {
    let body = scene["bodies"]
        .as_array()?
        .iter()
        .find(|body| body["id"].as_u64() == Some(body_id))?;
    let mut best: Option<(&Value, f64)> = None;
    for face in body["faces"].as_array()? {
        let Some(cylinder) = face.get("cylinder") else {
            continue;
        };
        let Some(origin) = xyz(&cylinder["origin"]) else {
            continue;
        };
        let Some(axis) = xyz(&cylinder["axis"]) else {
            continue;
        };
        if !axis_aligned(axis, want_axis) {
            continue;
        }
        let (perp, _) = perp_and_along(origin, axis, point);
        if perp > 3.0 {
            continue;
        }
        let Some(radius) = cylinder["radius"].as_f64() else {
            continue;
        };
        let score = (radius - want_radius).abs() + perp;
        if score > 2.5 + 3.0 {
            continue;
        }
        if (radius - want_radius).abs() > 2.5 {
            continue;
        }
        if best.is_none_or(|(_, current)| score < current) {
            best = Some((face, score));
        }
    }
    let (face, _) = best?;
    let origin = xyz(&face["cylinder"]["origin"]).unwrap_or(point);
    let frame = if want_axis[2].abs() >= 0.85 {
        json!({
            "origin": [origin[0], origin[1], point[2]],
            "primary_axis": [0.0, 0.0, 1.0],
            "secondary_axis": [1.0, 0.0, 0.0]
        })
    } else {
        frame_along(point, want_axis)
    };
    // Do not send source_surface_frame unless it is the live analytic
    // cylinder frame. A made-up frame is treated as a pick-time snapshot
    // and the canonicalizer applies a bogus rigid delta (30 mm Z yanks).
    Some(json!({
        "body_id": body_id,
        "face_id": face["id"],
        "face_key": face["key"],
        "kind": "cylindrical_face",
        "radius": face["cylinder"]["radius"],
        "frame": frame
    }))
}

fn circular_edge_at(
    scene: &Value,
    body_id: u64,
    xy: [f64; 2],
    z: f64,
    want_radius: f64,
) -> Option<Value> {
    circular_edge_along(
        scene,
        body_id,
        [xy[0], xy[1], z],
        [0.0, 0.0, 1.0],
        want_radius,
    )
}

fn circular_edge_along(
    scene: &Value,
    body_id: u64,
    point: [f64; 3],
    want_axis: [f64; 3],
    want_radius: f64,
) -> Option<Value> {
    let body = scene["bodies"]
        .as_array()?
        .iter()
        .find(|body| body["id"].as_u64() == Some(body_id))?;
    let mut best: Option<(&Value, f64, f64)> = None;
    for edge in body["edges"].as_array()? {
        let Some(circle) = edge.get("circle") else {
            continue;
        };
        if circle["closed"] != true {
            continue;
        }
        let Some(center) = xyz(&circle["center"]) else {
            continue;
        };
        let Some(normal) = xyz(&circle["normal"]) else {
            continue;
        };
        if !axis_aligned(normal, want_axis) {
            continue;
        }
        let (perp, along) = perp_and_along(center, want_axis, point);
        if perp > 2.0 {
            continue;
        }
        let Some(radius) = circle["radius"].as_f64() else {
            continue;
        };
        let radius_err = (radius - want_radius).abs();
        if radius_err > 2.5 {
            continue;
        }
        let score = perp + 0.15 * along.abs() + radius_err;
        if best.is_none_or(|(_, current, _)| score < current) {
            best = Some((edge, score, radius));
        }
    }
    let (edge, _, radius) = best?;
    let center = xyz(&edge["circle"]["center"]).unwrap_or(point);
    let frame = if want_axis[2].abs() >= 0.85 {
        json!({
            "origin": [center[0], center[1], center[2]],
            "primary_axis": [0.0, 0.0, 1.0],
            "secondary_axis": [1.0, 0.0, 0.0]
        })
    } else {
        frame_along(point, want_axis)
    };
    Some(json!({
        "body_id": body_id,
        "face_id": 0,
        "face_key": "",
        "edge_id": edge["id"],
        "edge_key": edge["key"],
        "kind": "circular_edge",
        "radius": radius,
        "frame": frame
    }))
}

fn grade(
    spec: &Spec,
    scene: &Value,
    document: &Value,
    assembly: &Value,
    preflight: &Value,
    plate_exports: &[(String, Value)],
    built: Built,
) -> Report {
    let bodies = scene["bodies"].as_array().cloned().unwrap_or_default();
    let features = document["features"].as_array().cloned().unwrap_or_default();
    let rotor = bodies
        .iter()
        .find(|body| body["id"].as_u64() == Some(built.rotor_id));
    let plate_bytes: Vec<Vec<u8>> = plate_exports
        .iter()
        .map(|(_, exported)| decode_3mf_bytes(exported))
        .collect();
    let bytes_len: usize = plate_bytes.iter().map(Vec::len).sum();
    let component_count = assembly["component_structure"]["definitions"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let joint_count = assembly["joints"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let occurrence_count = assembly["component_structure"]["occurrences"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);

    let mut lessons = Vec::new();
    push_lesson(
        &mut lessons,
        "blank",
        built.blank_ok,
        built.blank_detail.clone(),
    );
    push_lesson(
        &mut lessons,
        "fits",
        spec.fits_ok() && spec.stack_ok(),
        format!(
            "running +{:.2}  PIP +{:.2}  slip +{:.2}  friction +{:.2}  bed lead-in {:.2}",
            spec.fit_running_mm,
            spec.fit_pip_mm,
            spec.fit_slip_mm,
            spec.fit_friction_mm,
            spec.bed_relief_mm
        ),
    );
    push_lesson(
        &mut lessons,
        "no_press",
        spec.fit_friction_mm > 0.0 && spec.fit_friction_mm < spec.nozzle_mm && spec.assemble_ok(),
        "no press: plate drops over a constant journal; clocked E-clip snaps into a groove; pinch tabs to remove; retainer does not rub the rotor".to_string(),
    );
    push_lesson(
        &mut lessons,
        "assemble",
        built.assembly_ok
            && component_count == spec.assembly_component_count()
            && occurrence_count == spec.assembly_component_count()
            && joint_count >= spec.assembly_joint_count()
            && bodies.len() >= spec.min_bodies
            && built.roller_ids.len() == spec.roller_count,
        format!(
            "{} bodies, {} components, {} occurrences, {} joints; {}",
            bodies.len(),
            component_count,
            occurrence_count,
            joint_count,
            built.assembly_detail
        ),
    );
    push_lesson(
        &mut lessons,
        "rollers",
        spec.rollers_ok() && built.roller_ids.len() == spec.roller_count,
        format!(
            "{}× hollow PETG Ø{:.1}×L{:.1} radial rollers on PCD {:.1}; plate bore {:.1}; journal Ø{:.1}×h{:.1}",
            spec.roller_count,
            spec.roller_d(),
            spec.roller_len(),
            spec.pcd(),
            spec.plate_bore(),
            spec.inner_race_d(),
            spec.race_h()
        ),
    );
    push_lesson(
        &mut lessons,
        "even",
        spec.wing_count == 3
            && spec.post_count == 3
            && (spec.wing_offset_deg + spec.helix_deg * 0.5 - 60.0).abs() < 1e-9,
        "3 blades at 120°, 60° helix from 30° root".to_string(),
    );

    let rotor_faces = rotor
        .and_then(|body| body["faces"].as_array().map(|faces| faces.len()))
        .unwrap_or(0);
    let rotor_box = rotor.and_then(bbox);
    let rotor_span = rotor_box.map(|box3| box3[1][2] - box3[0][2]).unwrap_or(0.0);
    push_lesson(
        &mut lessons,
        "one_piece_rotor",
        rotor_faces >= spec.min_rotor_faces
            && rotor_span > spec.wing_h() * 0.7
            && spec.chord_root() > spec.chord_tip()
            && spec.root_blend_h() + 1e-9 >= 3.2
            && spec.tip_chord() + 1e-9 < spec.chord_tip(),
        format!(
            "rotor faces={rotor_faces} span={rotor_span:.1} (thin plate {plate:.1} + organic roots + tip taper to {tip:.1})",
            plate = spec.hub_deck_h(),
            tip = spec.tip_chord()
        ),
    );
    push_lesson(
        &mut lessons,
        "airfoil",
        spec.airfoil_ok() && rotor_faces >= spec.min_rotor_faces,
        format!(
            "{} t/c={:.2} TE≥{:.1}; σ={:.2}; root/tip chord {:.1}/{:.1}",
            spec.airfoil,
            spec.airfoil_t_c,
            spec.te_min(),
            spec.solidity(),
            spec.chord_root(),
            spec.chord_tip()
        ),
    );
    push_lesson(
        &mut lessons,
        "scale",
        spec.scale_ok() && spec.sanity_ok(),
        format!(
            "scale {:.2} of {}  rotor Ø{:.0} h{:.0}  bed {:.0}×{:.0}×{:.0}",
            spec.scale,
            spec.printer.name,
            spec.rotor_d(),
            spec.rotor_print_h(),
            spec.printer.bed_mm[0],
            spec.printer.bed_mm[1],
            spec.printer.bed_mm[2]
        ),
    );
    push_lesson(
        &mut lessons,
        "print_flat",
        spec.print_flat_ok(),
        format!(
            "stator prints flat (race ring Ø{:.1}/ID {:.1}, keepers {:.1}, U-window fence h{:.1} < pack {:.1}); rotor stands on deck {:.1}; crowned rollers print standing, window +{:.2}",
            spec.axle_flange_d(),
            spec.race_id(),
            spec.keeper(),
            spec.fence_h(),
            spec.pack_h(),
            spec.hub_deck_h(),
            spec.top_load()
        ),
    );
    push_lesson(
        &mut lessons,
        "helix",
        spec.helix_ok() && rotor_span > spec.wing_h() * 0.7,
        format!(
            "{}° helix, {} stations, organic root + tip taper to a flat landing",
            spec.helix_deg, spec.helix_stations
        ),
    );
    push_lesson(
        &mut lessons,
        "drawing",
        built.drawing_ok,
        built.drawing_detail.clone(),
    );
    push_lesson(
        &mut lessons,
        "report",
        spec.estimated_filament_usd() > 0.05 && spec.airfoil_ok(),
        format!(
            "solid_cm3={:.1} mass_g={:.1} usd={:.2} {}",
            spec.estimated_solid_cm3(),
            spec.estimated_print_mass_g(),
            spec.estimated_filament_usd(),
            spec.filament.name
        ),
    );
    let plates_ok = plate_bytes.len() >= spec.min_print_plates
        && plate_bytes
            .iter()
            .all(|bytes| bytes.len() > 32 && bytes[0] == 0x50 && bytes[1] == 0x4b);
    push_lesson(
        &mut lessons,
        "export",
        features.iter().all(|feature| {
            feature
                .get("status")
                .and_then(|status| status.get("state"))
                .and_then(Value::as_str)
                != Some("error")
        }) && (preflight["ok"] == true
            || preflight["timeline_errors"]
                .as_array()
                .is_some_and(|errors| errors.is_empty()))
            && plates_ok,
        format!(
            "{} plates, {} bytes, preflight={}",
            plate_exports.len(),
            bytes_len,
            preflight["ok"]
        ),
    );

    Report {
        ok: lessons.iter().all(|lesson| lesson.pass),
        spec_id: spec.id.clone(),
        lessons,
        body_count: bodies.len(),
        byte_length: bytes_len,
    }
}

fn push_lesson(lessons: &mut Vec<LessonResult>, id: &str, pass: bool, detail: String) {
    lessons.push(LessonResult {
        id: id.to_string(),
        pass,
        detail,
    });
}

fn decode_3mf_bytes(exported: &Value) -> Vec<u8> {
    exported["bytes_base64"]
        .as_str()
        .and_then(|text| {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD.decode(text).ok()
        })
        .unwrap_or_default()
}

/// NACA 4-digit modified half-thickness / chord (Ladson TM 4741 family).
/// `t` is t/c, `xt` is xt/c, `le_index` is I. The 20% model is scaled by t/0.20.
fn naca4_modified_yt_over_c(x: f64, t: f64, xt: f64, le_index: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    let p = xt.clamp(0.22, 0.42);
    let i = le_index.clamp(3.0, 9.0);
    let a0 = 0.2969 * (i / 6.0);
    let d0 = 0.002;
    let d1 = 0.234;
    let u = 1.0 - p;
    let rhs_aft = 0.10 - d0 - d1 * u;
    let d3 = -2.0 * (rhs_aft + d1 * u * 0.5) / u.powi(3);
    let d2 = (-d1 - 3.0 * d3 * u * u) / (2.0 * u);
    let ypp = 2.0 * d2 + 6.0 * d3 * u;
    let s = p.sqrt();
    let rhs0 = 0.10 - a0 * s;
    let rhs1 = -0.5 * a0 / s;
    let rhs2 = ypp + 0.25 * a0 / p.powf(1.5);
    let a3 = (rhs0 - p * rhs1 + 0.5 * p * p * rhs2) / p.powi(3);
    let a2 = rhs2 * 0.5 - 3.0 * a3 * p;
    let a1 = rhs1 - p * rhs2 + 3.0 * a3 * p * p;
    let y20 = if x <= p {
        a0 * x.sqrt() + a1 * x + a2 * x * x + a3 * x.powi(3)
    } else {
        let uu = 1.0 - x;
        d0 + d1 * uu + d2 * uu * uu + d3 * uu.powi(3)
    };
    (y20 * (t / 0.20)).max(0.0)
}

fn naca_symmetric_loop(
    chord: f64,
    thickness_ratio: f64,
    xt_c: f64,
    le_index: f64,
    stations: usize,
    te_min: f64,
) -> Vec<[f64; 2]> {
    let count = stations.max(6);
    let mut xs = Vec::with_capacity(count);
    for i in 0..count {
        let beta = std::f64::consts::PI * i as f64 / (count - 1) as f64;
        xs.push(0.5 * (1.0 - beta.cos()));
    }
    let mut upper = Vec::new();
    let mut lower = Vec::new();
    for x in xs {
        let mut yt = naca4_modified_yt_over_c(x, thickness_ratio, xt_c, le_index) * chord;
        if x >= 0.75 {
            yt = yt.max(te_min / 2.0);
        }
        if x >= 0.999 {
            yt = te_min / 2.0;
        }
        let xc = (x - 0.5) * chord;
        upper.push([xc, yt]);
        lower.push([xc, -yt]);
    }
    let mut points = upper;
    for point in lower.iter().rev().skip(1) {
        points.push(*point);
    }
    if let Some(first) = points.first().copied() {
        points.push(first);
    }
    points
}

fn add_airfoil(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    center: [f64; 2],
    angle_deg: f64,
    chord: f64,
    thickness_ratio: f64,
    xt_c: f64,
    le_index: f64,
    stations: usize,
    te_min: f64,
) -> Result<(), String> {
    let angle = angle_deg.to_radians();
    let (cos, sin) = (angle.cos(), angle.sin());
    let world: Vec<[f64; 2]> =
        naca_symmetric_loop(chord, thickness_ratio, xt_c, le_index, stations, te_min)
            .into_iter()
            .map(|[x, y]| [center[0] + cos * x - sin * y, center[1] + sin * x + cos * y])
            .collect();
    add_poly(call, &world, true)
}

fn cut_bed_relief_circle(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    z: f64,
    diameter: f64,
    body_id: u64,
    label: &str,
) -> Result<(), String> {
    let deck = offset_xy(call, z)?;
    begin_datum(call, deck)?;
    add_circle(call, [0.0, 0.0], diameter)?;
    let sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.bed_relief_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [body_id]
            }),
        )?,
        label,
    )?;
    Ok(())
}

fn cut_bed_relief_square(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    spec: &Spec,
    z: f64,
    size: f64,
    body_id: u64,
    label: &str,
) -> Result<(), String> {
    let deck = offset_xy(call, z)?;
    begin_datum(call, deck)?;
    add_oriented_rect(call, [0.0, 0.0], size, size, 0.0)?;
    let sketch = finish_sketch(call)?;
    require_clean(
        call(
            "solid_extrude",
            json!({
                "sketch_name": sketch,
                "profile_indices": [0],
                "operation": "cut",
                "extent": { "type": "distance", "distance": spec.bed_relief_h() },
                "taper_angle_deg": 0.0,
                "flip": false,
                "target_body_ids": [body_id]
            }),
        )?,
        label,
    )?;
    Ok(())
}

fn add_d_profile(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    diameter: f64,
    flat_x: f64,
) -> Result<(), String> {
    let r = diameter / 2.0;
    let half = (r * r - flat_x * flat_x).max(0.0).sqrt();
    let start = (-half).atan2(flat_x);
    let end = half.atan2(flat_x);
    let mut short = end - start;
    if short <= 0.0 {
        short += std::f64::consts::TAU;
    }
    let long = std::f64::consts::TAU - short;
    let mut pts = vec![[flat_x, -half]];
    for i in 1..=20 {
        let a = start - long * (i as f64) / 20.0;
        pts.push([r * a.cos(), r * a.sin()]);
    }
    pts.push([flat_x, -half]);
    add_poly(call, &pts, true)
}

fn add_oriented_rect(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    center: [f64; 2],
    length: f64,
    width: f64,
    angle_deg: f64,
) -> Result<(), String> {
    let angle = angle_deg.to_radians();
    let ux = [angle.cos(), angle.sin()];
    let uy = [-angle.sin(), angle.cos()];
    let half_l = length / 2.0;
    let half_w = width / 2.0;
    let corner = |s: f64, t: f64| {
        [
            center[0] + ux[0] * s + uy[0] * t,
            center[1] + ux[1] * s + uy[1] * t,
        ]
    };
    add_poly(
        call,
        &[
            corner(half_l, half_w),
            corner(half_l, -half_w),
            corner(-half_l, -half_w),
            corner(-half_l, half_w),
            corner(half_l, half_w),
        ],
        true,
    )
}

fn add_poly(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    points: &[[f64; 2]],
    ctrl_held: bool,
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
                "ctrl_held": ctrl_held
            }),
        )?;
    }
    Ok(())
}

fn require_solid_ok(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    label: &str,
) -> Result<Value, String> {
    let check = call("solid_check", json!({}))?;
    if check.get("ok") != Some(&Value::Bool(true)) {
        return Err(format!("{label} solid_check failed: {check}"));
    }
    Ok(check)
}

fn require_blank_document(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
) -> Result<String, String> {
    let scene = call("solid_scene", json!({}))?;
    let document = call("cad_document", json!({}))?;
    let bodies = scene["bodies"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    let features = document["features"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    if bodies != 0 {
        return Err(format!(
            "cad_new_project left {bodies} bodies / {features} features — do not continue a recovered document"
        ));
    }
    Ok(format!("blank: {bodies} bodies, {features} features"))
}

fn hide_construction(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
) -> Result<String, String> {
    let planes = call("construction_plane_definitions", json!({}))?;
    let plane_list = if let Some(items) = planes.as_array() {
        items.clone()
    } else {
        planes["planes"].as_array().cloned().unwrap_or_default()
    };
    let datum_ids: Vec<Value> = plane_list
        .iter()
        .filter_map(|plane| plane.get("datum_id").cloned())
        .collect();
    if datum_ids.is_empty() {
        return Err("helix stations created no construction planes to hide".to_string());
    }
    let document = call("cad_document", json!({}))?;
    let sketch_names: Vec<String> = document["features"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|feature| feature["kind"] == "sketch")
        .filter_map(|feature| feature["name"].as_str().map(str::to_string))
        .collect();
    let visibility = call(
        "cad_set_project_visibility",
        json!({
            "visibility": {
                "hidden_body_ids": [],
                "hidden_datum_plane_ids": datum_ids,
                "hidden_sketch_names": sketch_names
            }
        }),
    )?;
    let hidden = visibility["hidden_datum_plane_ids"]
        .as_array()
        .map(|items| items.len())
        .unwrap_or(0);
    if hidden == 0 {
        return Err("cad_set_project_visibility did not hide construction planes".to_string());
    }
    Ok(format!(
        "hid {hidden} datums, {} sketches",
        sketch_names.len()
    ))
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
    offset_origin_plane(call, "xy", 2, z)
}

fn offset_yz(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    x: f64,
) -> Result<Value, String> {
    offset_origin_plane(call, "yz", 0, x)
}

fn offset_origin_plane(
    call: &mut impl FnMut(&str, Value) -> Result<Value, String>,
    plane: &str,
    axis: usize,
    distance: f64,
) -> Result<Value, String> {
    call(
        "cad_set_focus",
        json!({ "focus": "datums", "explicit": true }),
    )?;
    let created = call(
        "construction_plane_offset",
        json!({
            "reference": { "type": "origin_plane", "plane": plane },
            "distance": distance
        }),
    )?;
    let planes = created["planes"].as_array().cloned().unwrap_or_default();
    let mut best: Option<(Value, f64)> = None;
    for item in &planes {
        let Some(origin) = item["basis"]["origin"].as_array() else {
            continue;
        };
        let Some(component) = origin.get(axis).and_then(Value::as_f64) else {
            continue;
        };
        let err = (component - distance).abs();
        if best.as_ref().is_none_or(|(_, current)| err < *current) {
            if let Some(id) = item.get("datum_id").cloned().filter(|id| !id.is_null()) {
                best = Some((id, err));
            }
        }
    }
    best.map(|(id, _)| id)
        .or_else(|| {
            planes
                .last()
                .and_then(|plane| plane.get("datum_id").cloned())
                .filter(|id| !id.is_null())
        })
        .ok_or_else(|| format!("no datum on {plane} at {distance}"))
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
        assert_eq!(spec.id, "fdm-print-vawt");
        assert_eq!(spec.nozzle_mm, 0.4);
        assert!(spec.fits_ok());
        assert!(spec.rollers_ok());
        assert!(spec.airfoil_ok());
        assert!(spec.helix_ok());
        assert!(spec.scale_ok());
        assert!(spec.sanity_ok());
        assert!(spec.print_flat_ok());
        assert_eq!(spec.wing_count, 3);
        assert!(spec.chord_root() > spec.chord_tip());
        assert!(spec.estimated_filament_usd() > 0.05);
        assert!(spec.min_bodies >= 8);
        assert!(spec.min_print_plates >= 1);
        assert_eq!(spec.print_plates.len(), 1);
        assert_eq!(spec.print_plates[0], "01-kit");
        assert!(spec
            .retired_print_plates
            .iter()
            .any(|name| name == "02-shaft"));
        assert!(spec
            .retired_print_plates
            .iter()
            .any(|name| name == "01-base"));
        assert!(spec
            .retired_print_plates
            .iter()
            .any(|name| name == "06-bushing"));
        assert_eq!(spec.materials.orange, "bambu.pla.basic.orange");
        assert_eq!(spec.materials.glow, "bambu.pla.glow.green");
        assert_eq!(spec.materials.petg, "bambu.petg.hf.black");
        assert!(!spec.materials.glow.contains("petg"));
        assert!(spec.stack_ok());
        assert!(spec.retainer_od() < spec.hub_od());
        assert_eq!(spec.assembly_component_count(), 3 + spec.roller_count);
        assert_eq!(spec.assembly_joint_count(), 2 + spec.roller_count);
        assert!((spec.airfoil_t_c - 0.24).abs() < 1e-9);
        assert!((spec.airfoil_xt_c - 0.35).abs() < 1e-9);
        assert!((spec.airfoil_le_index - 4.5).abs() < 1e-9);
        assert!(spec.flange_z().abs() < 1e-9);
        assert!((spec.wing_offset_deg + spec.helix_deg * 0.5 - 60.0).abs() < 1e-9);
        assert!(spec.rotor_print_h() / spec.scale <= spec.usable_bed()[2] + 1e-6);
        assert!(spec.cage_od() < spec.hub_deck_od());
        assert!((spec.plate_bore() - (spec.inner_race_d() + spec.fit_running_mm)).abs() < 1e-9);
        assert!(spec.axle_flange_d() + 1e-9 >= spec.cage_od());
        assert!(spec.cage_id() > spec.plate_bore());
        assert!(spec.pack_outer_r() + 1e-9 >= spec.wing_radius() * 0.9);
        assert!(spec.base_boss_d() + 8.0 <= spec.axle_flange_d() + 1e-9);
        assert!(spec.race_id() > spec.base_boss_d());
        assert!(spec.race_id() < spec.axle_flange_d());
        assert!(spec.cage_id() + 1e-9 >= spec.race_id());
        assert!(spec.post_circle_r() + spec.pad_d() * 0.5 + 1e-9 >= spec.race_id() * 0.5);
        assert!(spec.cage_rim() + 1e-9 >= spec.wall() * 2.0);
        assert!(spec.fence_h() + 1e-9 < spec.pack_h());
        assert!(spec.top_load_pocket() + 1e-9 > spec.cage_pocket());
        assert!(spec.window_w() + 1e-9 > spec.cage_pocket());
        assert!(spec.mouth_w() + 1e-9 > spec.window_w());
        assert!(spec.window_floor() > spec.race_z());
        assert!(spec.land_len() + 1e-9 >= 2.4);
        assert!(spec.land_len() < spec.roller_len() * 0.35);
        assert!(spec.roller_end_d() < spec.roller_d());
        assert!((spec.crown_drop() - 0.40).abs() < 1e-9);
        assert!((spec.roller_bore_d() - 2.4).abs() < 1e-9);
        assert!((spec.roller_d() * 0.5 - spec.roller_bore_d() * 0.5) > 2.5);
        assert!((spec.clip_mouth() - 8.0).abs() < 1e-9);
        assert!(spec.clip_mouth() < spec.groove_d());
        assert!(
            (spec.retainer_od() - (spec.retainer_d_hole() + 2.0 * spec.clip_arm_t())).abs() < 1e-9
        );
        assert!((spec.witness_d() - 2.4).abs() < 1e-9);
        assert!(spec.fence_h() + 1.2 <= spec.pack_h() + 1e-9);
        assert!(spec.fit_pip_mm > spec.fit_running_mm);
        let yt_max = naca4_modified_yt_over_c(0.35, 0.24, 0.35, 4.5);
        assert!((yt_max - 0.12).abs() < 0.004);
        assert!(
            naca4_modified_yt_over_c(0.35, 0.24, 0.35, 4.5)
                > naca4_modified_yt_over_c(0.10, 0.24, 0.35, 4.5)
        );
        assert!(
            naca4_modified_yt_over_c(0.35, 0.24, 0.35, 4.5)
                > naca4_modified_yt_over_c(0.70, 0.24, 0.35, 4.5)
        );
        assert!((spec.bed_relief_mm - spec.nozzle_mm * 2.0).abs() < 1e-9);
        assert!((spec.cage_pocket() - (spec.roller_d() + spec.fit_running_mm)).abs() < 1e-9);
        assert!(spec.hub_deck_od() > spec.axle_flange_d());
        assert!(spec.hub_deck_od() >= spec.wing_radius() * 2.0);
        assert!((spec.hub_h() - spec.hub_deck_h()).abs() < 1e-9);
        assert_eq!(spec.cage_h(), spec.fence_h());
        assert!((spec.pack_h() - spec.roller_d()).abs() < 1e-9);
        assert!(spec.roller_len() + 1e-9 >= 8.0);
        assert!(spec.base_boss_d() + 1e-9 < spec.hub_deck_od());
        assert!((spec.roller_axis(0)[0] - 1.0).abs() < 1e-9);
        assert!(spec.roller_axis(0)[2].abs() < 1e-9);
        assert!(spec.pack_outer_r() + 1e-6 >= spec.wing_radius() * 0.9);
        assert!(spec.assemble_ok());
        assert!(spec.captured_ok());
        assert!(spec.hub_deck_h() + 1e-9 >= 3.2);
        assert!(spec.hub_deck_h() + 1e-9 < 4.6);
        assert!(spec.base_h() + 1e-9 < 5.2);
        assert!(spec.root_blend_h() + 1e-9 >= 3.2);
        assert!(spec.tip_chord() + 1e-9 < spec.chord_tip());
        assert!(spec.root_scale() > 1.0);
        assert!(spec.pass_d() + 1e-9 < spec.plate_bore());
        assert!(spec.groove_d() + 1e-9 < spec.pass_d());
        assert!(spec.lock_flat_x() + 1e-9 < spec.inner_race_d() * 0.5);
        assert!((spec.plate_z() - spec.hub_z()).abs() < 1e-9);
        assert!((spec.blade_root_z() - (spec.plate_z() + spec.hub_deck_h())).abs() < 1e-9);
        assert!((spec.cage_z() - spec.race_z()).abs() < 1e-9);
        assert!(
            (spec.plate_z() - (spec.cage_z() + spec.pack_h() + spec.thrust_float)).abs() < 1e-9
        );
        assert!((spec.retainer_z() - spec.groove_z()).abs() < 1e-9);
    }
}
