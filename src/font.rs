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

            pub fn move_to(mut self, position: Point2<f64>) -> Self
            {
                let line = self.last();

                let line = Line{
                    start: line.end,
                    end: position
                };

                self.0.push(line);

                self
            }

            pub fn move_to_index(mut self, index: usize) -> Self
            {
                let line = Line{
                    start: self.last().end,
                    end: self.0[index].start
                };

                self.0.push(line);

                self
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

        let chars = [
            ('0', FontChar{
                lines: Builder::begin(Point2{x: 0.0, y: 1.0}, Point2{x: 1.0, y: 1.0})
                    .move_to(Point2{x: 1.0, y: 0.0})
                    .move_to(Point2{x: 0.0, y: 0.0})
                    .move_to_index(0)
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
