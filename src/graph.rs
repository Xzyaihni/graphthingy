use std::{
    error::Error,
    io::{self, BufReader, BufRead},
    fs::File,
    path::Path
};

use crate::{PPMImage, Color, Point2};


type PointType = Point2<f64>;

struct RunningAverage
{
    amount: u32,
    values: Vec<f64>
}

impl RunningAverage
{
    pub fn new(amount: u32) -> Self
    {
        Self{
            amount,
            values: Vec::new()
        }
    }

    pub fn is_empty(&self) -> bool
    {
        self.values.is_empty()
    }

    pub fn push(&mut self, points: &[PointType])
    {
        let take_amount = points.len().min(self.amount as usize);

        let s: f64 = points.iter().map(|point|
        {
            point.y
        }).rev().take(take_amount).sum();

        let value = s / take_amount as f64;

        self.values.push(value);
    }

    pub fn values(&self) -> &[f64]
    {
        &self.values
    }
}

pub struct Graph(GraphBuilder);

impl Graph
{
    #[allow(dead_code)]
    pub fn highest(&self) -> Option<f64>
    {
        self.0.highest_point
    }

    pub fn lowest(&self) -> Option<f64>
    {
        self.0.lowest_point
    }

    pub fn last(&self) -> Option<PointType>
    {
        self.0.points.last().copied()
    }

    pub fn points(&self) -> &[PointType]
    {
        &self.0.points
    }

    pub fn averages(&self) -> Option<&[f64]>
    {
        self.0.running_avg.as_ref().map(|running_avg| running_avg.values())
    }

    #[allow(dead_code)]
    pub fn first(&self) -> Option<PointType>
    {
        self.0.points.first().copied()
    }
}

pub struct GraphBuilder
{
    points: Vec<PointType>,
    running_avg: Option<RunningAverage>,
    lowest_point: Option<f64>,
    highest_point: Option<f64>
}

impl GraphBuilder
{
    pub fn new(running_avg: Option<u32>) -> Self
    {
        Self{
            points: Vec::new(),
            running_avg: running_avg.map(|amount| RunningAverage::new(amount)),
            lowest_point: None,
            highest_point: None
        }
    }

    pub fn push(&mut self, p: PointType)
    {
        self.points.push(p);

        let Point2{x: _x, y} = p;

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

    pub fn complete(mut self) -> Graph
    {
        self.points.sort_unstable_by(|a, b|
        {
            a.x.partial_cmp(&b.x).expect("values must be comparable")
        });

        if let Some(running_avg) = self.running_avg.as_mut()
        {
            (0..self.points.len()).for_each(|x| running_avg.push(&self.points[0..x]));
        }

        Graph(self)
    }
}

pub struct GrapherConfig
{
    pub log_scale: Option<f64>,
    pub min_avg: Option<f64>,
    pub min_height: Option<f64>,
    pub max_height: Option<f64>,
    pub running_avg: Option<u32>
}

impl Default for GrapherConfig
{
    fn default() -> Self
    {
        Self{
            log_scale: None,
            min_avg: None,
            min_height: None,
            max_height: None,
            running_avg: None
        }
    }
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

    #[allow(dead_code)]
    pub fn from_graphs(mut graphs: Vec<Graph>, config: GrapherConfig) -> Self
    {
        let mut this = Self::new(config);

        graphs.iter().for_each(|graph|
        {
            this.fit_graph(graph);
        });

        this.graphs = graphs;

        this
    }

    pub fn parse(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>>
    {
        let f = File::open(path)?;
        let reader = BufReader::new(f);

        let mut x_step = 1.0;
        let mut x = 0.0;

        let mut this_graph = GraphBuilder::new(self.config.running_avg);

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
            this_graph.push(Point2{x, y: value});
        }

        let this_graph = this_graph.complete();

        self.fit_graph(&this_graph);
        self.graphs.push(this_graph);

        Ok(())
    }

    fn fit_graph(&mut self, graph: &Graph)
    {
        if let Some(last) = graph.last()
        {
            eprintln!("right: {}", last.x);
            self.right = self.right.max(last.x);
        }

        let mut lowest = f64::MAX;

        let mut average = 0.0;
        graph.points().iter().for_each(|point|
        {
            if let Some(max_height) = self.config.max_height
            {
                self.top = max_height;
            } else
            {
                self.top = self.top.max(point.y);
            }

            if self.config.min_avg.is_some()
            {
                average += point.y;
            }

            lowest = lowest.min(point.y);
        });

        eprintln!("bottom: {lowest}");

        if let Some(min_height) = self.config.min_height
        {
            self.bottom = min_height;
        } else if self.bottom > lowest
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

    fn to_local(&self, point: &Point2<f64>, pad: Point2<f64>) -> Point2<f64>
    {
        Self::fit(&self.position(&point), pad)
    }

    fn position(&self, point: &Point2<f64>) -> Point2<f64>
    {
        let x = (point.x - self.left) / (self.right - self.left);
        let y = (point.y - self.bottom) / (self.top - self.bottom);

        let y = if let Some(scale) = self.config.log_scale
        {
            y.powf(scale)
        } else
        {
            y
        };

        Point2{x, y}
    }

    fn fit(point: &Point2<f64>, pad: Point2<f64>) -> Point2<f64>
    {
        point * ((-pad * 2.0) + 1.0) + pad
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()>
    {
        let width = 4000;
        let height = 2000;

        let thickness = 0.005;

        let aspect = width as f64 / height as f64;

        let mut pad = Point2{x: 0.05, y: 0.05};
        pad.x = pad.x / aspect;

        let pad = pad;

        let mut image = PPMImage::new(width, height, Color::white());

        let border_color = Color::black();

        {
            let l = 235;
            let c = Color{r: l, g: l, b: l};

            Self::draw_guides(&mut image, pad, thickness * 0.75, border_color, c);
        }
        
        let c = Color{r: 210, g: 210, b: 210};
        for graph in &self.graphs
        {
            if let Some(lowest) = graph.lowest()
            {
                let left = Point2{x: 0.0, y: lowest};
                let right = Point2{x: 0.0, y: lowest};

                let left = Point2{x: 0.0, y: self.position(&left).y};
                let right = Point2{x: 1.0, y: self.position(&right).y};

                image.line_thick(Self::fit(&left, pad), Self::fit(&right, pad), thickness, c);
            }
        }
        
        Self::draw_borders(&mut image, pad, thickness, border_color);

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

            self.draw_graph(&mut image, graph, pad, thickness, color);
        }

        image.save(path)
    }

    fn draw_graph(
        &self,
        image: &mut PPMImage,
        graph: &Graph,
        pad: Point2<f64>,
        thickness: f64,
        c: Color
    )
    {
        let points = graph.points();
        let pairs = points.iter().zip(points.iter().skip(1));

        for (input, output) in pairs
        {
            image.line_thick(self.to_local(input, pad), self.to_local(output, pad), thickness, c);
        }

        let averages = graph.averages();
        if let Some(values) = averages
        {
            let values = values.iter().zip(points).map(|(value, point)|
            {
                Point2{
                    x: point.x,
                    y: *value
                }
            });

            let average_pairs = values.clone().zip(values.skip(1));

            let avg_c = Color::white().lerp(c, 0.6);

            for (input, output) in average_pairs
            {
                image.line_thick(
                    self.to_local(&input, pad),
                    self.to_local(&output, pad),
                    thickness,
                    avg_c
                );
            }
        }
    }

    fn draw_guides(
        image: &mut PPMImage,
        pad: Point2<f64>,
        original_thickness: f64,
        border_color: Color,
        c: Color
    )
    {
        let lerp = |a: f64, b: f64, t: f64|
        {
            a * (1.0 - t) + b * t
        };

        let cap_at = |image: &mut PPMImage, y: f64, thickness: f64|
        {
            let y = lerp(pad.y, 1.0 - pad.y, y);

            let thickness_ratio = thickness / original_thickness;
            let guide_width = pad.x * 0.5 * thickness_ratio;
            image.line_thick(
                Point2{x: pad.x - guide_width, y},
                Point2{x: pad.x + guide_width, y},
                thickness,
                border_color
            );
        };

        let line_at = |image: &mut PPMImage, y: f64, thickness: f64|
        {
            {
                let y = lerp(pad.y, 1.0 - pad.y, y);

                image.line_thick(Point2{x: pad.x, y}, Point2{x: 1.0 - pad.x, y}, thickness, c);
            }

            cap_at(image, y, thickness);
        };

        let half_thickness = original_thickness * 0.5;

        line_at(image, 0.5, original_thickness);

        let divisions = 10;
        let step = 0.5 / divisions as f64;
        for i in 1..divisions
        {
            line_at(image, i as f64 * step, half_thickness);
            line_at(image, 0.5 + i as f64 * step, half_thickness);
        }

        cap_at(image, 0.0, original_thickness);
        cap_at(image, 1.0, original_thickness);
    }

    fn draw_borders(image: &mut PPMImage, pad: Point2<f64>, thickness: f64, c: Color)
    {
        image.line_thick(pad, Point2{x: pad.x, y: 1.0 - pad.y}, thickness, c);
        image.line_thick(pad, Point2{x: 1.0 - pad.x, y: pad.y}, thickness, c);
    }
}
