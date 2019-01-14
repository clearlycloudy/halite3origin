use std::ops::{Add,Sub};

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Coord(pub(i32,i32)); //(y,x)

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Vector(pub(f32,f32)); //(y,x)

#[derive(Clone,Copy,Debug)]
pub struct Dir(pub (i32,i32)); //(y,x)

impl Coord {
    fn abs( & self ) -> i32 {
        (self.0).0.abs() + (self.0).1.abs()
    }
}

impl Add for Coord {
    type Output = Coord;
    fn add(self, other: Coord) -> Coord {
        Coord(((self.0).0+(other.0).0, (self.0).1+(other.0).1))
    }
}

impl Sub for Coord {
    type Output = Coord;
    fn sub(self, other: Coord) -> Coord {
        Coord(((self.0).0-(other.0).0, (self.0).1-(other.0).1))
    }
}

impl Add for Vector {
    type Output = Vector;
    fn add(self, other: Vector) -> Vector {
        Vector(((self.0).0+(other.0).0, (self.0).1+(other.0).1))
    }
}

impl Sub for Vector {
    type Output = Vector;
    fn sub(self, other: Vector) -> Vector {
        Vector(((self.0).0-(other.0).0, (self.0).1-(other.0).1))
    }
}


