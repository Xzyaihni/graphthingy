use std::collections::HashMap;

use crate::Point2;


pub struct Line
{
    pub start: Point2<f64>,
    pub end: Point2<f64>
}

pub struct FontChar
{
    lines: Vec<Line>,
    width: f64,
    step: f64
}

impl FontChar
{
    pub fn lines(&self) -> &[Line]
    {
        &self.lines
    }

    pub fn width(&self) -> f64
    {
        self.width
    }

    pub fn step(&self) -> f64
    {
        self.step
    }

    pub fn total_step(&self) -> f64
    {
        self.width + self.step
    }
}

pub struct Font
{
    chars: HashMap<char, FontChar>
}

impl Default for Font
{
    fn default() -> Self
    {
        let default_step = 0.35;

        struct Builder(Vec<Line>);

        impl Builder
        {
            pub fn begin(start: Point2<f64>, end: Point2<f64>) -> Self
            {
                Self(vec![Line{start, end}])
            }

            #[allow(dead_code)]
            pub fn teleport(mut self, start: Point2<f64>, end: Point2<f64>) -> Self
            {
                self.0.push(Line{start, end});

                self
            }

            fn last(&self) -> &Line
            {
                self.0.last()
                    .expect("move_to must be called with at least 1 line")
            }

            pub fn move_from(self, index: usize, position: Point2<f64>) -> Self
            {
                let start = self.0[index].end;

                self.teleport(start, position)
            }

            pub fn move_to(self, position: Point2<f64>) -> Self
            {
                let start = self.last().end;

                self.teleport(start, position)
            }

            pub fn move_to_index(self, index: usize) -> Self
            {
                let start = self.last().end;
                let end = self.0[index].start;

                self.teleport(start, end)
            }

            pub fn build(self) -> Vec<Line>
            {
                self.0
            }
        }

        let six_lines = Builder::begin(Point2{x: 1.0, y: 1.0}, Point2{x: 0.0, y: 0.6})
            .move_to(Point2{x: 0.0, y: 0.0})
            .move_to(Point2{x: 1.0, y: 0.0})
            .move_to(Point2{x: 1.0, y: 0.6})
            .move_to_index(1)
            .build();

        // load a font? nah, id lose multiple hrs B)
        let chars = [
            ('0', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to_index(0)
                    .teleport(Point2{x: 0.0, y: 1.0}, Point2{x: 1.0, y: 0.0})
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('1', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 0.0}, Point2{x: 1.0, y: 1.0})
                    .build(),
                width: 0.1,
                step: default_step
            }),
            ('2', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 0.8}, Point2{x: 0.2, y: 1.0})
                    .move_to(Point2{x: 0.9, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 0.8})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .build(),
                width: 0.8,
                step: default_step
            }),
            ('3', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 0.2, y: 0.6})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .build(),
                width: 0.8,
                step: default_step
            }),
            ('4', FontChar{
                lines: Builder::begin(Point2{x: 0.8, y: 0.0}, Point2{x: 0.8, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.3})
                    .move_to(Point2{x: 1.0, y: 0.3})
                    .build(),
                width: 0.8,
                step: default_step
            }),
            ('5', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 1.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.6})
                    .move_to(Point2{x: 1.0, y: 0.6})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .build(),
                width: 0.8,
                step: default_step
            }),
            ('9', FontChar{
                lines: six_lines.iter().map(|x|
                {
                    let f = |p: Point2<f64>|
                    {
                        let g = |v: f64|
                        {
                            1.0 - v
                        };

                        Point2{
                            x: g(p.x),
                            y: g(p.y)
                        }
                    };

                    Line{start: f(x.start), end: f(x.end)}
                }).collect(),
                width: 0.6,
                step: default_step
            }),
            ('6', FontChar{
                lines: six_lines,
                width: 0.6,
                step: default_step
            }),
            ('7', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 0.1, y: 0.0})
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('8', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to_index(0)
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('.', FontChar{
                lines: Builder::begin(Point2{x: 0.4, y: 0.0}, Point2{x: 0.4, y: 0.0})
                    .build(),
                width: 0.1,
                step: default_step
            }),
            ('A', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 0.0}, Point2{x: 0.5, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .teleport(Point2{x: 0.15, y: 0.3}, Point2{x: 0.85, y: 0.3})
                    .build(),
                width: 0.8,
                step: default_step
            }),
            ('B', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 0.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.8, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 0.8})
                    .move_to(Point2{x: 0.8, y: 0.5})
                    .move_to(Point2{x: 0.0, y: 0.5})
                    .move_from(4, Point2{x: 1.0, y: 0.2})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to_index(0)
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('C', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 1.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .build(),
                width: 0.5,
                step: default_step
            }),
            ('D', FontChar{
                lines: Builder::begin(Point2{x: 0.7, y: 1.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 0.7, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.5})
                    .move_to_index(0)
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('E', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 1.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .teleport(Point2{x: 0.0, y: 0.5}, Point2{x: 1.0, y: 0.5})
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('F', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 1.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .teleport(Point2{x: 0.0, y: 0.5}, Point2{x: 0.9, y: 0.5})
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('G', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 1.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.5})
                    .move_to(Point2{x: 0.5, y: 0.5})
                    .build(),
                width: 0.8,
                step: default_step
            }),
            ('H', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 0.0, y: 0.0})
                    .teleport(Point2{x: 1.0, y: 1.0}, Point2{x: 1.0, y: 0.0})
                    .teleport(Point2{x: 0.0, y: 0.5}, Point2{x: 1.0, y: 0.5})
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('I', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 1.0}, Point2{x: 0.0, y: 1.0})
                    .teleport(Point2{x: 1.0, y: 0.0}, Point2{x: 0.0, y: 0.0})
                    .teleport(Point2{x: 0.5, y: 0.0}, Point2{x: 0.5, y: 1.0})
                    .build(),
                width: 0.4,
                step: default_step
            }),
            ('J', FontChar{
                lines: Builder::begin(Point2{x: 0.3, y: 1.0}, Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 0.0, y: 0.2})
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('K', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 0.0, y: 0.0})
                    .teleport(Point2{x: 0.8, y: 1.0}, Point2{x: 0.0, y: 0.5})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('L', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('M', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 0.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.5, y: 0.4})
                    .move_to(Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('N', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 0.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 1.0})
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('O', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to_index(0)
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('P', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 0.5}, Point2{x: 1.0, y: 0.5})
                    .move_to(Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('Q', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 0.9, y: 1.0})
                    .move_to(Point2{x: 0.9, y: 0.05})
                    .move_to(Point2{x: 0.0, y: 0.05})
                    .move_to_index(0)
                    .teleport(Point2{x: 0.5, y: 0.2}, Point2{x: 1.0, y: 0.0})
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('R', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 0.0}, Point2{x: 0.0, y: 0.5})
                    .move_to(Point2{x: 0.9, y: 0.5})
                    .move_to(Point2{x: 0.9, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('S', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 1.0}, Point2{x: 0.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.6})
                    .move_to(Point2{x: 1.0, y: 0.4})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .build(),
                width: 0.5,
                step: default_step
            }),
            ('T', FontChar{
                lines: Builder::begin(Point2{x: 1.0, y: 1.0}, Point2{x: 0.0, y: 1.0})
                    .teleport(Point2{x: 0.5, y: 1.0}, Point2{x: 0.5, y: 0.0})
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('U', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 1.0})
                    .build(),
                width: 0.6,
                step: default_step
            }),
            ('V', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 0.5, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 1.0})
                    .build(),
                width: 0.5,
                step: default_step
            }),
            ('W', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 0.2, y: 0.0})
                    .move_to(Point2{x: 0.5, y: 0.6})
                    .move_to(Point2{x: 0.8, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 1.0})
                    .build(),
                width: 0.9,
                step: default_step
            }),
            ('X', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 1.0, y: 0.0})
                    .teleport(Point2{x: 0.0, y: 0.0}, Point2{x: 1.0, y: 1.0})
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('Y', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 0.5, y: 0.6})
                    .move_to(Point2{x: 0.5, y: 0.0})
                    .move_from(0, Point2{x: 1.0, y: 1.0})
                    .build(),
                width: 0.7,
                step: default_step
            }),
            ('Z', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .build(),
                width: 0.7,
                step: default_step
            })
        ].into_iter().collect();

        Self{chars}
    }
}

impl Font
{
    pub fn get(&self, c: char) -> Option<&FontChar>
    {
        self.chars.get(&c)
    }
}
