use std::{
    f64,
    mem,
    io::{self, Write},
    fs::File,
    path::Path,
    ops::{Index, IndexMut}
};

use crate::Point2;


pub trait ColorRepr: Copy
{
    fn set(self, previous: Color) -> Color;
}

#[derive(Clone, Copy)]
pub struct Color
{
    pub r: u8,
    pub g: u8,
    pub b: u8
}

#[allow(dead_code)]
impl Color
{
    pub fn black() -> Self
    {
        Self{r: 0, g: 0, b: 0}
    }

    pub fn white() -> Self
    {
        Self{r: 255, g: 255, b: 255}
    }

    pub fn gradient_lerp(gradient: &[Self], amount: f32) -> Self
    {
        let colors_amount = gradient.len();

        let amount = amount * (colors_amount - 1) as f32;

        let amount_lower = (amount.floor() as usize).min(colors_amount.saturating_sub(2));

        gradient[amount_lower].lerp(gradient[amount_lower + 1], amount - amount_lower as f32)
    }

    pub fn lerp(self, other: Self, amount: f32) -> Self
    {
        Self{
            r: Self::lerp_single(self.r, other.r, amount),
            g: Self::lerp_single(self.g, other.g, amount),
            b: Self::lerp_single(self.b, other.b, amount)
        }
    }

    fn lerp_single(a: u8, b: u8, lerp: f32) -> u8
    {
        ((a as f32) * (1.0 - lerp) + (b as f32) * lerp) as u8
    }
}

impl From<ColorAlpha> for Color
{
    fn from(value: ColorAlpha) -> Self
    {
        Self{
            r: value.r,
            g: value.g,
            b: value.b
        }
    }
}

impl ColorRepr for Color
{
    fn set(self, _previous: Color) -> Color
    {
        self
    }
}

#[derive(Clone, Copy)]
pub struct ColorAlpha
{
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8
}

#[allow(dead_code)]
impl ColorAlpha
{
    pub fn black() -> Self
    {
        Self{r: 0, g: 0, b: 0, a: 255}
    }

    pub fn white() -> Self
    {
        Self{r: 255, g: 255, b: 255, a: 255}
    }

    pub fn lerp(self, other: Self, amount: f32) -> Self
    {
        Self{
            r: Self::lerp_single(self.r, other.r, amount),
            g: Self::lerp_single(self.g, other.g, amount),
            b: Self::lerp_single(self.b, other.b, amount),
            a: Self::lerp_single(self.a, other.a, amount)
        }
    }

    fn lerp_single(a: u8, b: u8, lerp: f32) -> u8
    {
        ((a as f32) * (1.0 - lerp) + (b as f32) * lerp) as u8
    }
}

impl From<Color> for ColorAlpha
{
    fn from(value: Color) -> Self
    {
        Self{
            r: value.r,
            g: value.g,
            b: value.b,
            a: 255
        }
    }
}

impl ColorRepr for ColorAlpha
{
    fn set(self, previous: Color) -> Color
    {
        Color::from(self).lerp(previous, 1.0 - (self.a as f32 / u8::MAX as f32))
    }
}

struct SignedDistance
{
    point: Point2<f64>
}

impl SignedDistance
{
    pub fn new(point: Point2<f64>) -> Self
    {
        Self{point}
    }

    pub fn translate(&mut self, translation: Point2<f64>)
    {
        self.point -= translation;
    }

    pub fn rotate(&mut self, rotation: f64)
    {
        self.point = self.point.rotate(rotation);
    }

    pub fn scale(&mut self, scale: Point2<f64>)
    {
        self.point /= scale;
    }

    pub fn circle(&self, size: f64) -> f64
    {
        self.point.x.hypot(self.point.y) - size
    }

    pub fn rectangle(&self, size: f64) -> f64
    {
        let dist = self.point.abs() - size;

        let n_dist = Point2{x: dist.x.max(0.0), y: dist.y.max(0.0)};
        let out_dist = n_dist.x.hypot(n_dist.y);

        let in_dist = dist.x.max(dist.y).min(0.0);

        out_dist + in_dist
    }
}

#[derive(Clone, Copy)]
pub struct Line
{
    p0: Point2<f64>,
    p1: Point2<f64>,
    thickness: f64,
    c: Color,
    rotation: f64,
    half_length: f64,
    local_length: f64,
    clip_distance: f64
}

pub struct DeferredSDFDrawer<'a>
{
    image: &'a mut PPMImage,
    lines: Vec<Line>
}

impl<'a> DeferredSDFDrawer<'a>
{
    pub fn line(&mut self, p0: Point2<f64>, p1: Point2<f64>, thickness: f64, c: Color)
    {
        let p0 = self.image.with_aspect(p0);
        let p1 = self.image.with_aspect(p1);

        let p_offset = p1 - p0;

        let rotation = p_offset.y.atan2(p_offset.x);
        let length = p_offset.x.hypot(p_offset.y);

        let half_length = length / 2.0;
        let local_length = half_length / thickness;
        
        let clip_distance =
            (p_offset.x.powi(2) + p_offset.y.powi(2))
            + 2.0 * length * thickness
            + thickness.powi(2);

        let line = Line{
            p0, p1,
            thickness,
            c,
            rotation,
            half_length,
            local_length,
            clip_distance
        };

        self.lines.push(line);
    }

    pub fn submit(self)
    {
        self.image.sdf_lines(self.lines);
    }
}

pub struct PPMImage
{
    data: Vec<Color>,
    width: usize,
    height: usize,
    width_bigger: bool,
    aspect: f64
}

#[allow(dead_code)]
impl PPMImage
{
    pub fn new(width: usize, height: usize, c: Color) -> Self
    {
        let width_bigger = width >= height;
        let aspect = width as f64 / height as f64;

        let aspect = if width_bigger
        {
            aspect
        } else
        {
            2.0 - aspect
        };

        Self{data: vec![c; width * height], width, height, width_bigger, aspect}
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()>
    {
        if self.width == 0 || self.height == 0
        {
            panic!("cant save a 0 sized image");
        }

        let mut f = File::create(path)?;

        let header = format!("P6\n{} {}\n255\n", self.width, self.height);

        f.write_all(header.as_bytes())?;

        let data = self.data.iter().flat_map(|c| [c.r, c.g, c.b]).collect::<Vec<u8>>();
        f.write_all(&data)
    }

    pub fn sdf_drawer(&mut self) -> DeferredSDFDrawer
    {
        DeferredSDFDrawer{image: self, lines: Vec::new()}
    }

    fn with_aspect(&self, point: Point2<f64>) -> Point2<f64>
    {
        if self.width_bigger
        {
            Point2{x: point.x * self.aspect, y: point.y}
        } else
        {
            Point2{x: point.x, y: point.y * self.aspect}
        }
    }

    fn without_aspect(&self, point: Point2<f64>) -> Point2<f64>
    {
        if self.width_bigger
        {
            Point2{x: point.x / self.aspect, y: point.y}
        } else
        {
            Point2{x: point.x, y: point.y / self.aspect}
        }
    }

    fn sdf_lines(&mut self, lines: Vec<Line>)
    {
        let mut i = 0;
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                let curr = Point2{
                    x: x as f64 / self.width as f64,
                    y: 1.0 - (y as f64 / self.height as f64)
                };

                let curr = self.with_aspect(curr);

                for line in lines.iter().copied().rev()
                {
                    let Line{
                        p0,
                        p1,
                        thickness,
                        c,
                        half_length,
                        local_length,
                        clip_distance,
                        rotation
                    } = line;

                    let curr_distance = curr - p0;
                    let curr_distance = curr_distance.x.powi(2) + curr_distance.y.powi(2);

                    if curr_distance > clip_distance
                    {
                        continue;
                    }

                    let mut start_cap = SignedDistance::new(curr);
                    let mut end_cap = SignedDistance::new(curr);

                    let mut body = SignedDistance::new(curr);

                    start_cap.translate(p0);
                    end_cap.translate(p1);

                    body.translate(p0);

                    body.rotate(rotation);

                    body.translate(Point2{x: half_length, y: 0.0});
                    body.scale(Point2{x: local_length, y: 1.0});

                    let is_cap = (start_cap.circle(thickness) < 0.0)
                        || (end_cap.circle(thickness) < 0.0);

                    let is_body = body.rectangle(thickness) < 0.0;

                    if is_cap || is_body
                    {
                        *unsafe{ self.data.get_unchecked_mut(i) } = c;
                        break;
                    }
                }

                i += 1;
            }
        }
    }

    pub fn line_thick(
        &mut self,
        p0: Point2<f64>,
        p1: Point2<f64>,
        thickness: f64,
        c: impl ColorRepr
    )
    {
        let diff = p1 - p0;
        let angle = -diff.y.atan2(diff.x);

        // borrow checker why r u like this
        let direction = |this: &Self, raw: Point2<f64>|
        {
            let direction = raw.rotate(angle);

            this.without_aspect(direction) * thickness
        };

        let up = direction(self, Point2{x: 0.0, y: 1.0});

        // the caps
        let cap_points = 3;

        for i in 0..cap_points
        {
            let (middle, end, middle_n, end_n) = {
                let point_at = |i, x_scale|
                {
                    // where the point is from 0 to 1
                    let p_n = i as f64 / (cap_points + 1) as f64;

                    // where the point is from -1 to 1
                    let p_s = p_n * 2.0 - 1.0;
                    
                    let p = p_n * f64::consts::PI;

                    let up = p_s;
                    let right = p.sin() * x_scale;

                    let point = Point2{x: right, y: up};

                    direction(self, point)
                };

                (
                    point_at(i + 1, 1.0), point_at(i + 2, 1.0),
                    point_at(i + 1, -1.0), point_at(i + 2, -1.0)
                )
            };

            self.triangle(p0 - up, p0 + middle_n, p0 + end_n, c);
            self.triangle(p1 - up, p1 + middle, p1 + end, c);
        }

        // the line
        self.triangle(p0 + up, p1 + up, p0 - up, c);
        self.triangle(p0 - up, p1 + up, p1 - up, c);
    }

    pub fn triangle(
        &mut self,
        p0: Point2<f64>,
        p1: Point2<f64>,
        p2: Point2<f64>,
        c: impl ColorRepr
    )
    {
        self.triangle_local(self.to_local(p0), self.to_local(p1), self.to_local(p2), c);
    }

    fn triangle_local(
        &mut self,
        p0: Point2<usize>,
        p1: Point2<usize>,
        p2: Point2<usize>,
        c: impl ColorRepr
    )
    {
        let y_lowest = p0.y.min(p1.y.min(p2.y));
        let y_highest = p0.y.max(p1.y.max(p2.y));

        let mut low_high_pairs = vec![(usize::MAX, usize::MIN); y_highest - y_lowest + 1];

        let mut on_pixel = |pos: Point2<usize>|
        {
            let index = pos.y - y_lowest;
            let this_pair = &mut low_high_pairs[index];

            this_pair.0 = this_pair.0.min(pos.x);
            this_pair.1 = this_pair.1.max(pos.x);
        };

        self.line_pixels(p0, p1).into_iter().for_each(&mut on_pixel);
        self.line_pixels(p1, p2).into_iter().for_each(&mut on_pixel);
        self.line_pixels(p2, p0).into_iter().for_each(on_pixel);

        for (index, (low, high)) in low_high_pairs.into_iter().enumerate()
        {
            let y = index + y_lowest;

            for x in low..=high
            {
                let point = Point2{x, y};

                self[point] = c.set(self[point]);
            }
        }
    }

    pub fn line_pixels(&self, p0: Point2<usize>, p1: Point2<usize>) -> Vec<Point2<usize>>
    {
        let mut p0 = p0.cast::<i32>();
        let p1 = p1.cast::<i32>();

        let distance = (p1 - p0).abs();

        let dx = distance.x;
        let sx: i32 = if p0.x < p1.x { 1 } else { -1 };

        let dy = -distance.y;
        let sy: i32 = if p0.y < p1.y { 1 } else { -1 };

        let mut error = dx + dy;

        let mut pixels = Vec::new();
        loop
        {
            pixels.push(p0.cast());

            if p0.x == p1.x && p0.y == p1.y
            {
                break;
            }

            let error_2 = error * 2;

            if error_2 >= dy
            {
                if p0.x == p1.x
                {
                    break;
                }

                error += dy;
                p0.x += sx;
            }

            if error_2 <= dx
            {
                if p0.y == p1.y
                {
                    break;
                }

                error += dx;
                p0.y += sy;
            }
        }

        pixels
    }

    pub fn line(&mut self, p0: Point2<f64>, p1: Point2<f64>, c: Color)
    {
        let mut p0 = self.to_local_f(p0);
        let mut p1 = self.to_local_f(p1);

        let is_steep = (p1.y - p0.y).abs() > (p1.x - p0.x).abs();

        if is_steep
        {
            mem::swap(&mut p0.x, &mut p0.y);
            mem::swap(&mut p1.x, &mut p1.y);
        }

        if p0.x > p1.x
        {
            mem::swap(&mut p0.x, &mut p1.x);
            mem::swap(&mut p0.y, &mut p1.y);
        }

        let d = p1 - p0;

        let gradient = if d.x == 0.0
        {
            1.0
        } else
        {
            d.y / d.x
        };

        let rfpart = |v: f64|
        {
            1.0 - v.fract()
        };

        let mut plot = |x: f64, y: f64, brightness: f64|
        {
            let pos = Point2{x: x as usize, y: y as usize};

            let prev_v = &mut self[pos];
            *prev_v = prev_v.lerp(c, brightness as f32);
        };

        let mut draw_endpoint = |point: Point2<f64>, recip: bool|
        {
            let x_end = point.x.round();
            let y_end = point.y + gradient * (x_end - point.x);

            let x_gap = {
                let v = point.x + 0.5;

                if recip
                {
                    rfpart(v)
                } else
                {
                    v.fract()
                }
            };

            let x_pxl = x_end;
            let y_pxl = y_end.floor();

            if is_steep
            {
                plot(y_pxl, x_pxl, rfpart(y_end) * x_gap);
                plot(y_pxl + 1.0, x_pxl, y_end.fract() * x_gap);
            } else
            {
                plot(x_pxl, y_pxl, rfpart(y_end) * x_gap);
                plot(x_pxl, y_pxl + 1.0, y_end.fract() * x_gap);
            }

            (y_end, x_pxl)
        };

        let (y_end, x_pxl1) = draw_endpoint(p0, true);
        let (_, x_pxl2) = draw_endpoint(p1, false);

        let mut intery = y_end + gradient;

        for x in ((x_pxl1 + 1.0) as usize)..=((x_pxl2 - 1.0) as usize)
        {
            if is_steep
            {
                plot(intery.floor(), x as f64, rfpart(intery));
                plot(intery.floor() + 1.0, x as f64, intery.fract());
            } else
            {
                plot(x as f64, intery.floor(), rfpart(intery));
                plot(x as f64, intery.floor() + 1.0, intery.fract());
            }

            intery += gradient;
        }
    }

    fn to_local_f(&self, point: Point2<f64>) -> Point2<f64>
    {
        Point2{
            x: point.x * self.width as f64,
            y: (1.0 - point.y) * self.height as f64
        }
    }

    pub fn to_local(&self, point: Point2<f64>) -> Point2<usize>
    {
        let p = self.to_local_f(point);

        Point2{
            x: (p.x as usize).max(0).min(self.width - 1),
            y: (p.y as usize).max(0).min(self.height - 1)
        }
    }

    fn index(&self, pos: Point2<usize>) -> usize
    {
        assert!(pos.y < self.height);
        assert!(pos.x < self.width);

        pos.x + pos.y * self.width
    }
}

impl Index<Point2<usize>> for PPMImage
{
    type Output = Color;

    fn index(&self, index: Point2<usize>) -> &Self::Output
    {
        &self.data[self.index(index)]
    }
}

impl IndexMut<Point2<usize>> for PPMImage
{
    fn index_mut(&mut self, index: Point2<usize>) -> &mut Self::Output
    {
        let index = self.index(index);
        &mut self.data[index]
    }
}
