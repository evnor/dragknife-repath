pub mod app;
pub mod types;
pub mod vec3;

use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;

use gcode::{GCode, Mnemonic, Span, Word};
use types::DragknifeState;
use vec3::Vec3;
use log::debug;

use types::{
    ArcDirection, ArcMovement, Command, DragknifeConfig, GCodeState, HomeMovement,
    LinearMovement, Movement, OtherCommand, RapidMovement,
};

pub struct DragknifePath<'a> {
    pub commands: Vec<Command<'a>>,
}

impl<'a> DragknifePath<'a> {
    pub fn from_gcode(gcodes: impl Iterator<Item = &'a GCode>) -> DragknifePath<'a> {
        let mut output = Vec::with_capacity(gcodes.size_hint().0);
        let mut settings = GCodeState::default();
        for gcode in gcodes {
            let command = Command::from_gcode(gcode, output.last(), &mut settings);
            output.push(command);
        }
        DragknifePath { commands: output }
    }

    pub fn to_fixed_gcode(&self, config: &DragknifeConfig) -> Vec<GCode> {
        let mut fixed = vec![];
        let mut prev_angle = None;
        let mut settings = GCodeState::default();
        let mut dragknife_state = DragknifeState::default();
        for command in self.commands.iter() {
            fixed.append(&mut command.to_fixed_gcode(prev_angle, &mut settings, &mut dragknife_state, &config));
            prev_angle = command.end_angle();
        }
        fixed
    }
}

impl<'a> Command<'a> {
    pub fn from_gcode(
        gcode: &'a GCode,
        prev_command: Option<&Command>,
        settings: &mut GCodeState,
    ) -> Command<'a> {
        let start = prev_command.end_pos();
        match gcode.mnemonic() {
            Mnemonic::Miscellaneous => Command::Other(OtherCommand {
                original: gcode,
                pos: start,
                angle: prev_command.end_angle(),
            }),
            Mnemonic::ProgramNumber => Command::Other(OtherCommand {
                original: gcode,
                pos: start,
                angle: prev_command.end_angle(),
            }),
            Mnemonic::ToolChange => Command::Other(OtherCommand {
                original: gcode,
                pos: start,
                angle: prev_command.end_angle(),
            }),
            Mnemonic::General => match gcode.major_number() {
                0 /* Rapid movement */ => {
                    let end = settings.get_target(start, gcode);
                    Command::Rapid(RapidMovement {
                        original: gcode,
                        start,
                        end,
                    })
                },
                1 /* Linear interpolation */ => {
                    let end = settings.get_target(start, gcode);
                    let angle = if (start-end).project_plane(&settings.plane).magnitude() <= f32::EPSILON {
                        prev_command.end_angle()
                    } else {
                        Some(start.angle_to(&end, &settings.plane))
                    };
                    Command::Linear(LinearMovement {
                        original: gcode,
                        start,
                        end,
                        angle,
                    })
                },
                2 /* Circular interpolation, clockwise */ => {
                    let target = settings.get_target(start, gcode);
                    let center_off = settings.get_center_offset(gcode);
                    let center = start + center_off;
                    let start_angle = center.angle_to(&start, &settings.plane) - FRAC_PI_2;
                    let end_angle = center.angle_to(&target, &settings.plane) - FRAC_PI_2;
                    let radius = (start - center).project_plane(&settings.plane).magnitude();
                    let end = (target-center).normalized()*radius + center;
                    debug!("{} {:?} {:?}", radius, target, center);
                    Command::Arc(ArcMovement {
                        original: gcode,
                        direction: ArcDirection::CW,
                        start,
                        end,
                        center,
                        start_angle,
                        end_angle,
                    })
                },
                3 /* Circular interpolation, counterclockwise */ => {
                    let target = settings.get_target(start, gcode);
                    let center_off = settings.get_center_offset(gcode);
                    let center = start + center_off;
                    let start_angle = center.angle_to(&start, &settings.plane) + FRAC_PI_2;
                    let end_angle = center.angle_to(&target, &settings.plane) + FRAC_PI_2;
                    let radius = (start - center).project_plane(&settings.plane).magnitude();
                    let end = (target-center).normalized()*radius + center;

                    Command::Arc(ArcMovement {
                        original: gcode,
                        direction: ArcDirection::CCW,
                        start,
                        end,
                        center,
                        start_angle,
                        end_angle,
                    })
                },
                28 /* Go to machine zero */=> {
                    Command::Home(HomeMovement {
                        original: gcode,
                        start,
                    })
                },
                17 /* Select XY plane */|
                18 /* Select ZX plane */|
                19 /* Select YZ plane */|
                20 /* Select inches */|
                21 /* Select mm */|
                90 /* Select absolute positioning */|
                91 /* Select relative positioning */|
                40..=44 /* Tool compensation: NOOP */ |
                54..=59 /* Set coord systems: NOOP */|
                _ => {
                    let other_command = OtherCommand {
                        original: gcode,
                        pos: start,
                        angle: prev_command.end_angle(),
                    };
                    other_command.update_settings(settings);
                    Command::Other(other_command)
                },
            },
        }
    }

    pub fn to_fixed_gcode(
        &self,
        previous_angle: Option<f32>,
        settings: &mut GCodeState,
        state: &mut DragknifeState,
        config: &DragknifeConfig,
    ) -> Vec<GCode> {
        match self {
            Command::Other(command) => {
                if command.original.major_number() == 91 {
                    // Discard relative positioning command
                    vec![]
                } else {
                    command.update_settings(settings);
                    vec![command.original.clone()]
                }
            }
            Command::Linear(command) => {
                let mut out = Command::create_swivel_path(previous_angle, self, settings, state, config);
                let target = if let Some(angle) = command.angle {
                    command.end + Vec3::unit_angle(angle, &settings.plane) * config.knife_offset
                } else {
                    command.end
                };
                let target = target.coords_for_plane(&settings.plane);
                let mut new = GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
                    .with_argument(Word::new(
                        settings.plane.axis_1().main_name(),
                        target.0,
                        Span::PLACEHOLDER,
                    ))
                    .with_argument(Word::new(
                        settings.plane.axis_2().main_name(),
                        target.1,
                        Span::PLACEHOLDER,
                    ));
                Command::add_misc_args_and_update_settings(&mut new, self, state, settings);
                out.push(new);
                out
            }
            Command::Arc(command) => {
                let mut out = Command::create_swivel_path(previous_angle, self, settings, state, config);
                let new_start = command.start
                    + Vec3::unit_angle(command.start_angle, &settings.plane) * config.knife_offset;
                let new_end = command.end
                    + Vec3::unit_angle(command.end_angle, &settings.plane) * config.knife_offset;
                debug!("{:?} {:?}", Vec3::unit_angle(command.end_angle, &settings.plane), command.end);
                let center_offset = command.center - new_start;
                let new_end = new_end.coords_for_plane(&settings.plane);
                let center_offset = center_offset.coords_for_plane(&settings.plane);
                let mut new = GCode::new(
                    Mnemonic::General,
                    if let ArcDirection::CW = command.direction {
                        2.0
                    } else {
                        3.0
                    },
                    Span::PLACEHOLDER,
                )
                .with_argument(Word::new(
                    settings.plane.axis_1().main_name(),
                    new_end.0,
                    Span::PLACEHOLDER,
                ))
                .with_argument(Word::new(
                    settings.plane.axis_2().main_name(),
                    new_end.1,
                    Span::PLACEHOLDER,
                ))
                .with_argument(Word::new(
                    settings.plane.axis_1().center_name(),
                    center_offset.0,
                    Span::PLACEHOLDER,
                ))
                .with_argument(Word::new(
                    settings.plane.axis_2().center_name(),
                    center_offset.1,
                    Span::PLACEHOLDER,
                ));
                Command::add_misc_args_and_update_settings(&mut new, self, state, settings);
                out.push(new);
                out
            }
            Command::Home(command) => vec![command.original.clone()],
            Command::Rapid(command) => vec![command.original.clone()],
        }
    }

    fn create_swivel_path(
        previous_angle: Option<f32>,
        next: &Command<'a>,
        settings: &GCodeState,
        state: &mut DragknifeState,
        config: &DragknifeConfig,
    ) -> Vec<GCode> {
        if let (Some(from_angle), Some(to_angle)) = (previous_angle, next.start_angle()) {
            let signed_angle = signed_angle(from_angle, to_angle);
            if signed_angle.abs() > config.sharp_angle_threshold {
                let mut out = vec![];
                let start_height = next.start_pos().third_coord(&settings.plane);
                out.push(
                    GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER)
                        .with_argument(Word::new(
                            settings.plane.axis_3().main_name(),
                            config.lift_config.calcute_height(start_height),
                            Span::PLACEHOLDER,
                        ))
                        .with_argument(Word::new(
                            'F',
                            config.swivel_feedrate / settings.unit_factor(),
                            Span::PLACEHOLDER,
                        )),
                );
                let center_offset = (Vec3::unit_angle(from_angle + PI, &settings.plane)
                    * config.knife_offset)
                    .coords_for_plane(&settings.plane);
                let target = (Vec3::unit_angle(to_angle, &settings.plane) * config.knife_offset
                    + next.start_pos())
                .coords_for_plane(&settings.plane);
                out.push(
                    GCode::new(
                        Mnemonic::General,
                        if signed_angle > 0. { 2.0 } else { 3.0 },
                        Span::PLACEHOLDER,
                    )
                    .with_argument(Word::new(
                        settings.plane.axis_1().main_name(),
                        target.0,
                        Span::PLACEHOLDER,
                    ))
                    .with_argument(Word::new(
                        settings.plane.axis_2().main_name(),
                        target.1,
                        Span::PLACEHOLDER,
                    ))
                    .with_argument(Word::new(
                        settings.plane.axis_1().center_name(),
                        center_offset.0,
                        Span::PLACEHOLDER,
                    ))
                    .with_argument(Word::new(
                        settings.plane.axis_2().center_name(),
                        center_offset.1,
                        Span::PLACEHOLDER,
                    )),
                );
                out.push(
                    GCode::new(Mnemonic::General, 1.0, Span::PLACEHOLDER).with_argument(Word::new(
                        settings.plane.axis_3().main_name(),
                        start_height,
                        Span::PLACEHOLDER,
                    )),
                );
                state.next_feedrate = Some(settings.feedrate / settings.unit_factor());
                return out;
            }
        }
        vec![]
    }

    fn add_misc_args_and_update_settings(
        new_gcode: &mut GCode,
        command: &Command,
        state: &mut DragknifeState,
        settings: &mut GCodeState,
    ) {
        if command.update_settings(settings) {
            state.next_feedrate = None;
            new_gcode
                .push_argument(Word::new(
                    'F',
                    settings.feedrate / settings.unit_factor(),
                    Span::PLACEHOLDER,
                ))
                .unwrap();
        } else if let Some(feedrate) = state.next_feedrate {
            state.next_feedrate = None;
            new_gcode
                .push_argument(Word::new(
                    'F',
                    feedrate / settings.unit_factor(),
                    Span::PLACEHOLDER,
                ))
                .unwrap();
        }
        let plane = settings.plane;
        for arg in command.original().arguments() {
            if ![
                plane.axis_1().main_name(),
                plane.axis_2().main_name(),
                plane.axis_1().center_name(),
                plane.axis_2().center_name(),
                'F',
            ]
            .contains(&arg.letter)
            {
                new_gcode.push_argument(*arg).unwrap();
            }
        }
    }
}

fn signed_angle(a: f32, b: f32) -> f32 {
    (a - b + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}
