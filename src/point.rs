use std::ops::{Add, Sub, Mul, Div, Neg};


#[derive(Debug, Clone, Copy)]
pub struct Point2<T>
{
    pub x: T,
    pub y: T
}

macro_rules! op_impl
{
    ($op_trait:ident, $op_fn:ident) =>
    {
        impl<T: $op_trait<Output=T>> $op_trait<Point2<T>> for Point2<T>
        {
            type Output = Point2<T>;

            fn $op_fn(self, rhs: Point2<T>) -> Self::Output
            {
                Point2{
                    x: self.x.$op_fn(rhs.x),
                    y: self.y.$op_fn(rhs.y)
                }
            }
        }

        impl<T: $op_trait<Output=T> + Clone> $op_trait<Point2<T>> for &Point2<T>
        {
            type Output = Point2<T>;

            fn $op_fn(self, rhs: Point2<T>) -> Self::Output
            {
                Point2{
                    x: self.x.clone().$op_fn(rhs.x),
                    y: self.y.clone().$op_fn(rhs.y)
                }
            }
        }
    }
}

macro_rules! op_impl_scalar
{
    ($op_trait:ident, $op_fn:ident) =>
    {
        impl<T: $op_trait<Output=T> + Clone> $op_trait<T> for Point2<T>
        {
            type Output = Point2<T>;

            fn $op_fn(self, rhs: T) -> Self::Output
            {
                Point2{
                    x: self.x.$op_fn(rhs.clone()),
                    y: self.y.$op_fn(rhs)
                }
            }
        }

        impl<T: $op_trait<Output=T> + Clone> $op_trait<T> for &Point2<T>
        {
            type Output = Point2<T>;

            fn $op_fn(self, rhs: T) -> Self::Output
            {
                Point2{
                    x: self.x.clone().$op_fn(rhs.clone()),
                    y: self.y.clone().$op_fn(rhs)
                }
            }
        }
    }
}

impl<T: Neg<Output=T>> Neg for Point2<T>
{
    type Output = Self;

    fn neg(self) -> Self::Output
    {
        Point2{
            x: -self.x,
            y: -self.y
        }
    }
}

op_impl!{Add, add}
op_impl!{Sub, sub}
op_impl!{Mul, mul}
op_impl!{Div, div}

op_impl_scalar!{Add, add}
op_impl_scalar!{Sub, sub}
op_impl_scalar!{Mul, mul}
op_impl_scalar!{Div, div}
