use std::env;

use graph::{GrapherConfig, Grapher};
use config::Config;

pub use image::{PPMImage, DeferredSDFDrawer, Color, ColorAlpha};
pub use point::Point2;

mod point;

mod graph;
mod image;
mod config;


fn main()
{
    let config = Config::parse(env::args().skip(1)).unwrap();

    let grapher_config = GrapherConfig{
        log_scale: config.log_scale,
        min_avg: config.min_avg,
        min_height: config.min_height,
        max_height: config.max_height,
        running_avg: config.running_avg
    };

    let mut grapher = Grapher::new(grapher_config);

    for data in config.paths
    {
        grapher.parse(data).unwrap();
    }

    grapher.save("graph.ppm").unwrap();
}
