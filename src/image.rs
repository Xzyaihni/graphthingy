use std::{
    mem,
    io::{self, Write},
    fs::File,
    path::Path,
    ops::{Index, IndexMut}
};


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

struct SignedDistance
{
    point: (f64, f64)
}

impl SignedDistance
{
    pub fn new(point: (f64, f64)) -> Self
    {
        Self{point}
    }

    pub fn translate(&mut self, translation: (f64, f64))
    {
        self.point.0 -= translation.0;
        self.point.1 -= translation.1;
    }

    pub fn rotate(&mut self, rotation: f64)
    {
        let (r_sin, r_cos) = rotation.sin_cos();

        self.point = (
            r_cos * self.point.0 + r_sin * self.point.1,
            r_cos * self.point.1 - r_sin * self.point.0
        );
    }

    pub fn scale(&mut self, scale: (f64, f64))
    {
        self.point.0 /= scale.0;
        self.point.1 /= scale.1;
    }

    pub fn circle(&self, size: f64) -> f64
    {
        self.point.0.hypot(self.point.1) - size
    }

    pub fn rectangle(&self, size: f64) -> f64
    {
        let dist = (self.point.0.abs() - size, self.point.1.abs() - size);

        let n_dist = (dist.0.max(0.0), dist.1.max(0.0));
        let out_dist = n_dist.0.hypot(n_dist.1);

        let in_dist = dist.0.max(dist.1).min(0.0);

        out_dist + in_dist
    }
}

#[derive(Clone, Copy)]
pub struct Line
{
    p0: (f64, f64),
    p1: (f64, f64),
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
    pub fn line(&mut self, p0: (f64, f64), p1: (f64, f64), thickness: f64, c: Color)
    {
        let p0 = self.image.with_aspect(p0);
        let p1 = self.image.with_aspect(p1);

        let p_offset = (p1.0 - p0.0, p1.1 - p0.1);

        let rotation = p_offset.1.atan2(p_offset.0);
        let length = p_offset.0.hypot(p_offset.1);

        let half_length = length / 2.0;
        let local_length = half_length / thickness;
        
        let clip_distance =
            (p_offset.0.powi(2) + p_offset.1.powi(2))
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

    fn with_aspect(&self, point: (f64, f64)) -> (f64, f64)
    {
        if self.width_bigger
        {
            (point.0 * self.aspect, point.1)
        } else
        {
            (point.0, point.1 * self.aspect)
        }
    }

    fn sdf_lines(&mut self, lines: Vec<Line>)
    {
        let mut i = 0;
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                let curr = (
                    x as f64 / self.width as f64,
                    1.0 - (y as f64 / self.height as f64)
                );

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

                    let curr_distance = (curr.0 - p0.0, curr.1 - p0.1);
                    let curr_distance = curr_distance.0.powi(2) + curr_distance.1.powi(2);

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

                    body.translate((half_length, 0.0));
                    body.scale((local_length, 1.0));

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

    pub fn line(&mut self, p0: (f64, f64), p1: (f64, f64), c: Color)
    {
        let mut p0 = self.to_local_f(p0);
        let mut p1 = self.to_local_f(p1);

        let is_steep = (p1.1 - p0.1).abs() > (p1.0 - p0.0).abs();

        if is_steep
        {
            mem::swap(&mut p0.0, &mut p0.1);
            mem::swap(&mut p1.0, &mut p1.1);
        }

        if p0.0 > p1.0
        {
            mem::swap(&mut p0.0, &mut p1.0);
            mem::swap(&mut p0.1, &mut p1.1);
        }

        let d = (p1.0 - p0.0, p1.1 - p0.1);

        let gradient = if d.0 == 0.0
        {
            1.0
        } else
        {
            d.1 / d.0
        };

        let rfpart = |v: f64|
        {
            1.0 - v.fract()
        };

        let mut plot = |x: f64, y: f64, brightness: f64|
        {
            let pos = (x as usize, y as usize);

            let prev_v = &mut self[pos];
            *prev_v = prev_v.lerp(c, brightness as f32);
        };

        let mut draw_endpoint = |point: (f64, f64), recip: bool|
        {
            let x_end = point.0.round();
            let y_end = point.1 + gradient * (x_end - point.0);

            let x_gap = {
                let v = point.0 + 0.5;

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

    fn to_local_f(&self, point: (f64, f64)) -> (f64, f64)
    {
        (
            point.0 * self.width as f64,
            (1.0 - point.1) * self.height as f64
        )
    }

    pub fn to_local(&self, point: (f64, f64)) -> (usize, usize)
    {
        let p = self.to_local_f(point);

        ((p.0 as usize).max(0).min(self.width - 1), (p.1 as usize).max(0).min(self.height - 1))
    }

    fn index(&self, pos: (usize, usize)) -> usize
    {
        assert!(pos.1 < self.height);
        assert!(pos.0 < self.width);

        pos.0 + pos.1 * self.width
    }
}

impl Index<(usize, usize)> for PPMImage
{
    type Output = Color;

    fn index(&self, index: (usize, usize)) -> &Self::Output
    {
        &self.data[self.index(index)]
    }
}

impl IndexMut<(usize, usize)> for PPMImage
{
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output
    {
        let index = self.index(index);
        &mut self.data[index]
    }
}
