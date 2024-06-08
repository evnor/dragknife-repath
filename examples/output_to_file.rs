use std::f32::consts::PI;

use dragknife_repath::types::{DragknifeConfig, LiftConfig};
use dragknife_repath::DragknifePath;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .target(env_logger::Target::Stdout)
        .init();
    let fc: String = std::fs::read_to_string("test_input2.cnc").unwrap();
    let got: Vec<_> = gcode::parse(&fc).collect();
    let path = DragknifePath::from_gcode(got.iter());
    let config = DragknifeConfig::new(0.35, LiftConfig::RelativeHeight(0.5), 10. * PI / 180., 300.);
    let fixed = path.to_fixed_gcode(&config);
    std::fs::write(
        "output.cnc",
        fixed.iter().map(|g| format!("{}\n", g)).collect::<String>(),
    )
    .unwrap();
}
