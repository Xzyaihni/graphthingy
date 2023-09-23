use std::{
    error::Error,
    io::{self, BufReader, BufRead},
    fs::File,
    path::Path
};

use crate::{PPMImage, DeferredSDFDrawer, Color};


type PointType = (f64, f64);

pub struct Graph
{
    points: Vec<PointType>,
    lowest_point: Option<f64>,
    highest_point: Option<f64>
}

impl Graph
{
    pub fn new() -> Self
    {
        Self{
            points: Vec::new(),
            lowest_point: None,
            highest_point: None
        }
    }

    pub fn push(&mut self, p: PointType)
    {
        self.points.push(p);

        let (_x, y) = p;

        self.lowest_point = Some(self.lowest_point.map(|lowest|
        {
            if lowest > y
            {
                y
            } else
            {
                lowest
            }
        }).unwrap_or(y));

        self.highest_point = Some(self.highest_point.map(|highest|
        {
            if highest < y
            {
                y
            } else
            {
                highest
            }
        }).unwrap_or(y));
    }

    #[allow(dead_code)]
    pub fn highest(&self) -> Option<f64>
    {
        self.highest_point
    }

    pub fn lowest(&self) -> Option<f64>
    {
        self.lowest_point
    }

    pub fn last(&self) -> Option<PointType>
    {
        self.points.last().copied()
    }

    pub fn points(&self) -> &[PointType]
    {
        &self.points
    }

    #[allow(dead_code)]
    pub fn first(&self) -> Option<PointType>
    {
        self.points.first().copied()
    }
}

pub struct GrapherConfig
{
    pub log_scale: Option<f64>,
    pub min_avg: Option<f64>
}

#[allow(dead_code)]
pub struct Grapher
{
    top: f64,
    bottom: f64,
    left: f64,
    right: f64,
    config: GrapherConfig,
    graphs: Vec<Graph>
}

impl Grapher
{
    pub fn new(config: GrapherConfig) -> Self
    {
        Self{
            top: 0.0,
            bottom: f64::MAX,
            left: 0.0,
            right: 0.0,
            config,
            graphs: Vec::new()
        }
    }

    pub fn parse(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>>
    {
        let f = File::open(path)?;
        let reader = BufReader::new(f);

        let mut x_step = 1.0;
        let mut x = 0.0;

        let mut this_graph = Graph::new();

        for line in reader.lines()
        {
            let line = line?;

            if line.starts_with("step")
            {
                let step = &line["step".len()..];
                let step: f64 = step.trim().parse()?;

                x_step = step;

                continue;
            }

            let value: f64 = line.trim().parse()?;

            x += x_step;
            this_graph.push((x, value));
        }

        self.fit_graph(&this_graph);
        self.graphs.push(this_graph);

        Ok(())
    }

    fn fit_graph(&mut self, graph: &Graph)
    {
        if let Some(last) = graph.last()
        {
            self.right = self.right.max(last.0);
            eprintln!("right: {}", self.right);
        }

        let mut lowest = f64::MAX;

        let mut average = 0.0;
        graph.points().iter().for_each(|point|
        {
            self.top = self.top.max(point.1);

            if self.config.min_avg.is_some()
            {
                average += point.1;
            }

            lowest = lowest.min(point.1);
        });

        eprintln!("bottom: {lowest}");

        if self.bottom > lowest
        {
            self.bottom = if let Some(scale) = self.config.min_avg
            {
                let average = average / graph.points().len() as f64;

                let diff = (average - lowest).abs();

                lowest - (diff * scale)
            } else
            {
                lowest
            };
        }
    }

    fn to_local(&self, point: &(f64, f64), pad: (f64, f64)) -> (f64, f64)
    {
        Self::fit(&self.position(&point), pad)
    }

    fn position(&self, point: &(f64, f64)) -> (f64, f64)
    {
        let x = (point.0 - self.left) / (self.right - self.left);
        let y = (point.1 - self.bottom) / (self.top - self.bottom);

        let y = if let Some(scale) = self.config.log_scale
        {
            y.powf(scale)
        } else
        {
            y
        };

        (x, y)
    }

    fn fit(point: &(f64, f64), pad: (f64, f64)) -> (f64, f64)
    {
        (
            point.0 * (1.0 - pad.0 * 2.0) + pad.0,
            point.1 * (1.0 - pad.1 * 2.0) + pad.1
        )
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()>
    {
        let width = 4000;
        let height = 2000;

        let thickness = 0.005;

        let aspect = width as f64 / height as f64;

        let mut pad = (0.05, 0.05);
        pad.0 = pad.0 / aspect;

        let pad = pad;

        let mut image = PPMImage::new(width, height, Color::white());
        let mut sdf_drawer = image.sdf_drawer();
        
        let c = Color{r: 210, g: 210, b: 210};
        for graph in &self.graphs
        {
            if let Some(lowest) = graph.lowest()
            {
                let left = (0.0, lowest);
                let right = (0.0, lowest);

                let left = (0.0, self.position(&left).1);
                let right = (1.0, self.position(&right).1);

                sdf_drawer.line(Self::fit(&left, pad), Self::fit(&right, pad), thickness, c);
            }
        }
        
        Self::draw_borders(&mut sdf_drawer, pad, thickness, Color::black());

        let mut colors = vec![
            Color{r: 255, g: 120, b: 120},
            Color{r: 120, g: 255, b: 120},
            Color{r: 120, g: 120, b: 255},
            Color{r: 255, g: 120, b: 220},
            Color{r: 255, g: 220, b: 120}
        ].into_iter();

        for graph in &self.graphs
        {
            let color = colors.next().expect("it should have enough colors");

            self.draw_graph(&mut sdf_drawer, graph, pad, thickness, color);
        }

        sdf_drawer.submit();

        image.save(path)
    }

    fn draw_graph(
        &self,
        sdf_drawer: &mut DeferredSDFDrawer,
        graph: &Graph,
        pad: (f64, f64),
        thickness: f64,
        c: Color
    )
    {
        let points = graph.points();
        let pairs = points.iter().zip(points.iter().skip(1));

        for (input, output) in pairs
        {
            sdf_drawer.line(self.to_local(input, pad), self.to_local(output, pad), thickness, c);
        }
    }

    fn draw_borders(sdf_drawer: &mut DeferredSDFDrawer, pad: (f64, f64), thickness: f64, c: Color)
    {
        sdf_drawer.line(pad, (pad.0, 1.0 - pad.1), thickness, c);
        sdf_drawer.line(pad, (1.0 - pad.0, pad.1), thickness, c);
    }
}
