pub use graph::{GrapherConfig, Grapher};

pub use image::{PPMImage, DeferredSDFDrawer, Color, ColorAlpha};
pub use point::Point2;

pub mod point;

pub mod graph;
mod image;
