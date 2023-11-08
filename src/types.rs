use crate::vec3::Vec3;
use gcode::GCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default)]
pub enum GCodeUnit {
    #[default]
    Millimeters,
    Inches,
}

pub enum GCodeAxis {
    X,
    Y,
    Z,
}

impl GCodeAxis {
    pub fn main_name(&self) -> char {
        match self {
            GCodeAxis::X => 'X',
            GCodeAxis::Y => 'Y',
            GCodeAxis::Z => 'Z',
        }
    }
    pub fn center_name(&self) -> char {
        match self {
            GCodeAxis::X => 'I',
            GCodeAxis::Y => 'J',
            GCodeAxis::Z => 'K',
        }
    }
    pub fn rotation_name(&self) -> char {
        match self {
            GCodeAxis::X => 'A',
            GCodeAxis::Y => 'B',
            GCodeAxis::Z => 'C',
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum GCodePlane {
    #[default]
    XY,
    ZX,
    YZ,
}

impl GCodePlane {
    pub fn axis_1(&self) -> GCodeAxis {
        match &self {
            GCodePlane::XY => GCodeAxis::X,
            GCodePlane::ZX => GCodeAxis::Z,
            GCodePlane::YZ => GCodeAxis::Y,
        }
    }
    pub fn axis_2(&self) -> GCodeAxis {
        match &self {
            GCodePlane::XY => GCodeAxis::Y,
            GCodePlane::ZX => GCodeAxis::X,
            GCodePlane::YZ => GCodeAxis::Z,
        }
    }
    pub fn axis_3(&self) -> GCodeAxis {
        match &self {
            GCodePlane::XY => GCodeAxis::Z,
            GCodePlane::ZX => GCodeAxis::Y,
            GCodePlane::YZ => GCodeAxis::X,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum GCodePositioning {
    Relative,
    #[default]
    Absolute,
}

#[derive(Debug, Clone, Copy)]
pub struct GCodeState {
    pub unit: GCodeUnit,
    pub plane: GCodePlane,
    pub positioning: GCodePositioning,
    pub feedrate: f32,
}

impl Default for GCodeState {
    fn default() -> Self {
        GCodeState {
            unit: Default::default(),
            plane: Default::default(),
            positioning: Default::default(),
            feedrate: 3000.,
        }
    }
}

impl GCodeState {
    pub fn unit_factor(&self) -> f32 {
        match self.unit {
            GCodeUnit::Millimeters => 1.,
            GCodeUnit::Inches => 2.54,
        }
    }

    pub fn get_target(&mut self, mut pos: Vec3, gcode: &GCode) -> Vec3 {
        let unit = self.unit_factor();
        if let GCodePositioning::Absolute = self.positioning {
            pos.x = gcode
                .value_for(GCodeAxis::X.main_name())
                .map(|e| e * unit)
                .unwrap_or(pos.x);
            pos.y = gcode
                .value_for(GCodeAxis::Y.main_name())
                .map(|e| e * unit)
                .unwrap_or(pos.y);
            pos.z = gcode
                .value_for(GCodeAxis::Z.main_name())
                .map(|e| e * unit)
                .unwrap_or(pos.z);
        } else {
            pos.x += gcode
                .value_for(GCodeAxis::X.main_name())
                .map(|e| e * unit)
                .unwrap_or(0.);
            pos.y += gcode
                .value_for(GCodeAxis::Y.main_name())
                .map(|e| e * unit)
                .unwrap_or(0.);
            pos.z += gcode
                .value_for(GCodeAxis::Z.main_name())
                .map(|e| e * unit)
                .unwrap_or(0.);
        }
        pos
    }

    pub fn get_center_offset(&self, gcode: &GCode) -> Vec3 {
        let unit = self.unit_factor();
        let x = gcode
            .value_for(GCodeAxis::X.center_name())
            .map(|e| e * unit)
            .unwrap_or(0.);
        let y = gcode
            .value_for(GCodeAxis::Y.center_name())
            .map(|e| e * unit)
            .unwrap_or(0.);
        let z = gcode
            .value_for(GCodeAxis::Z.center_name())
            .map(|e| e * unit)
            .unwrap_or(0.);
        Vec3 { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LiftConfig {
    AbsoluteHeight(f32),
    RelativeHeight(f32),
}

impl Default for LiftConfig {
    fn default() -> Self {
        LiftConfig::RelativeHeight(1.0)
    }
}

impl LiftConfig {
    pub fn calcute_height(&self, from: f32) -> f32 {
        match self {
            LiftConfig::AbsoluteHeight(h) => *h,
            LiftConfig::RelativeHeight(h) => from + h,
        }
    }

    pub fn get_height_mut(&mut self) -> &mut f32 {
        match self {
            LiftConfig::AbsoluteHeight(h) => h,
            LiftConfig::RelativeHeight(h) => h,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct DragknifeConfig {
    pub knife_offset: f32,
    pub lift_config: LiftConfig,
    pub sharp_angle_threshold: f32,
    pub swivel_feedrate: f32,
}

impl DragknifeConfig {
    pub fn new(knife_offset: f32, lift_config: LiftConfig, sharp_angle_threshold: f32, swivel_feedrate: f32) -> Self {
        DragknifeConfig {
            knife_offset,
            lift_config,
            sharp_angle_threshold,
            swivel_feedrate,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DragknifeState {
    pub next_feedrate: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct HomeMovement<'a> {
    pub original: &'a GCode,
    pub start: Vec3,
}

#[derive(Debug, Clone)]
pub struct RapidMovement<'a> {
    pub original: &'a GCode,
    pub start: Vec3,
    pub end: Vec3,
}

#[derive(Debug, Clone)]
pub struct LinearMovement<'a> {
    pub original: &'a GCode,
    pub start: Vec3,
    pub end: Vec3,
    pub angle: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum ArcDirection {
    CW,
    CCW,
}

#[derive(Debug, Clone)]
pub struct ArcMovement<'a> {
    pub original: &'a GCode,
    pub direction: ArcDirection,
    pub start: Vec3,
    pub end: Vec3,
    pub center: Vec3,
    pub start_angle: f32,
    pub end_angle: f32,
}

#[derive(Debug, Clone)]
pub struct OtherCommand<'a> {
    pub original: &'a GCode,
    pub pos: Vec3,
    pub angle: Option<f32>,
}

impl<'a> OtherCommand<'a> {
    pub fn update_settings(&self, settings: &mut GCodeState) {
        match self.original.major_number() {
            17 /* Select XY plane */=> {
                settings.plane = GCodePlane::XY;
            },
            18 /* Select ZX plane */=> {
                settings.plane = GCodePlane::ZX;
            },
            19 /* Select YZ plane */=> {
                settings.plane = GCodePlane::YZ;
            },
            20 /* Select inches */=> {
                settings.unit = GCodeUnit::Inches;
            },
            21 /* Select mm */=> {
                settings.unit = GCodeUnit::Millimeters;
            },
            90 /* Select absolute positioning */=> {
                settings.positioning = GCodePositioning::Absolute;
            },
            91 /* Select relative positioning */=> {
                settings.positioning = GCodePositioning::Relative;
            },
            40..=44 /* Tool compensation: NOOP */ => {},
            54..=59 /* Set coord systems: NOOP */ => {},
            _ => {},
        }
    }
}

#[derive(Debug, Clone)]
pub enum Command<'a> {
    Other(OtherCommand<'a>),
    Linear(LinearMovement<'a>),
    Arc(ArcMovement<'a>),
    Home(HomeMovement<'a>),
    Rapid(RapidMovement<'a>),
}

impl<'a> Command<'a> {
    pub fn original(&self) -> &'a GCode {
        match self {
            Command::Other(command) => command.original,
            Command::Linear(command) => command.original,
            Command::Arc(command) => command.original,
            Command::Home(command) => command.original,
            Command::Rapid(command) => command.original,
        }
    }

    pub fn update_settings(&self, settings: &mut GCodeState) -> bool {
        match self {
            Command::Other(command) => command.update_settings(settings),
            _ => {
                if let Some(feedrate) = self.original().value_for('F') {
                    settings.feedrate = feedrate * settings.unit_factor();
                    return true;
                };
            }
        }
        false
    }
}

pub trait Movement {
    fn start_pos(&self) -> Vec3;
    fn end_pos(&self) -> Vec3;
    fn start_angle(&self) -> Option<f32>;
    fn end_angle(&self) -> Option<f32>;
}

impl<'a> Movement for Command<'a> {
    fn start_pos(&self) -> Vec3 {
        match self {
            Command::Other(movement) => movement.pos,
            Command::Linear(movement) => movement.start,
            Command::Arc(movement) => movement.start,
            Command::Home(movement) => movement.start,
            Command::Rapid(movement) => movement.start,
        }
    }

    fn end_pos(&self) -> Vec3 {
        match self {
            Command::Other(movement) => movement.pos,
            Command::Linear(movement) => movement.end,
            Command::Arc(movement) => movement.end,
            Command::Home(_) => Vec3::zero(),
            Command::Rapid(movement) => movement.end,
        }
    }

    fn start_angle(&self) -> Option<f32> {
        match self {
            Command::Other(movement) => movement.angle,
            Command::Linear(movement) => movement.angle,
            Command::Arc(movement) => Some(movement.start_angle),
            Command::Home(_) => None,
            Command::Rapid(_) => None,
        }
    }

    fn end_angle(&self) -> Option<f32> {
        match self {
            Command::Other(movement) => movement.angle,
            Command::Linear(movement) => movement.angle,
            Command::Arc(movement) => Some(movement.end_angle),
            Command::Home(_) => None,
            Command::Rapid(_) => None,
        }
    }
}

impl<'a> Movement for Option<&Command<'a>> {
    fn start_pos(&self) -> Vec3 {
        self.map_or(Vec3::zero(), |c| c.start_pos())
    }

    fn end_pos(&self) -> Vec3 {
        self.map_or(Vec3::zero(), |c| c.end_pos())
    }

    fn start_angle(&self) -> Option<f32> {
        self.map_or(None, |c| c.start_angle())
    }

    fn end_angle(&self) -> Option<f32> {
        self.map_or(None, |c| c.end_angle())
    }
}
