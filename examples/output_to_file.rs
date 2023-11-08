use std::f32::consts::PI;

use dragknife_repath::DragknifePath;
use dragknife_repath::types::DragknifeConfig;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .target(env_logger::Target::Stdout)
        .init();
    let fc: String = std::fs::read_to_string("testpiece_multidepth.cnc").unwrap();
    let got: Vec<_> = gcode::parse(&fc).collect();
    let path = DragknifePath::from_gcode(got.iter());
    let config = DragknifeConfig::new(3.2, 1., 20. / PI * 180.);
    let fixed = path.to_fixed_gcode(&config);
    std::fs::write(
        "output.cnc",
        fixed.iter().map(|g| format!("{}\n", g)).collect::<String>(),
    )
    .unwrap();
}
