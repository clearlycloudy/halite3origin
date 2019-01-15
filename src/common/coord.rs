use rand::Rng;

use std::ops::{Add,Sub};

use mapping::mapraw::{ResourceMap, RawMaps, UnitMap, DropoffMap, Unit};

#[derive(Clone,Copy,Debug,Eq,PartialEq,Hash)]
pub struct Coord(pub(i32,i32)); //(y,x)

#[derive(Clone,Copy,Debug,PartialEq)]
pub struct Vector(pub(f32,f32)); //(y,x)

#[derive(Clone,Copy,Debug)]
pub struct Dir(pub (i32,i32)); //(y,x)

impl Default for Coord {
    fn default() -> Coord {
        Coord( (0,0) )
    }
}

impl From<(i32,i32)> for Coord {
    fn from( c: (i32,i32) ) -> Coord {
        Coord( ( c.0, c.1 ) )
    }
}

impl Coord {
    pub fn y( & self ) -> i32 {
        (self.0).0
    }
    pub fn x( & self ) -> i32 {
        (self.0).1
    }
    pub fn y_mut( & mut self ) -> & mut i32 {
        & mut (self.0).0
    }
    pub fn x_mut( & mut self ) -> & mut i32 {
        & mut (self.0).1
    }
    pub fn abs( & self ) -> i32 {
        (self.0).0.abs() + (self.0).1.abs()
    }
    pub fn mod_bound( & self, bound: &(i32,i32) ) -> Coord {
        let r = ( (self.0).0 % bound.0 + bound.0 ) % bound.0;
        let c = ( (self.0).1 % bound.1 + bound.1 ) % bound.1;
        Coord( (r,c) )
    }
    pub fn modulo( & self, bound: &Coord ) -> Coord {
        let r = ( (self.0).0 % (bound.0).0 + (bound.0).0 ) % (bound.0).0;
        let c = ( (self.0).1 % (bound.0).1 + (bound.0).1 ) % (bound.0).1;
        Coord( (r,c) )
    }
    pub fn get_prioritized_dir( & self, dest: & Coord, rawmap: & RawMaps ) -> Vec<Coord> {
        let mut dif = *dest - *self;
        
        if (dif.0).0 > rawmap.map_r.dim().y()/2 {
            (dif.0).0 -= rawmap.map_r.dim().y();
        }
        if (dif.0).0 < -rawmap.map_r.dim().y()/2 {
            (dif.0).0 += rawmap.map_r.dim().y();
        }

        if (dif.0).1 > rawmap.map_r.dim().x()/2 {
            (dif.0).1 -= rawmap.map_r.dim().x();
        }
        if (dif.0).1 < -rawmap.map_r.dim().x()/2 {
            (dif.0).1 += rawmap.map_r.dim().x();
        }

        let mut choices_no = vec![];
        let mut choices = vec![];

        if (dif.0).0 >= 1 {
            choices.push( Coord((1,0)) );
            choices_no.push( Coord((-1,0)) );
            choices_no.push( Coord((0,-1)) );
            choices_no.push( Coord((0,1)) );
        } else if (dif.0).0 <= -1 {
            choices.push( Coord((-1,0)) );
            choices_no.push( Coord((1,0)) );
            choices_no.push( Coord((0,-1)) );
            choices_no.push( Coord((0,1)) );
        }
        
        if (dif.0).1 >= 1 {
            choices.push( Coord((0,1)) );
            choices_no.push( Coord((0,-1)) );
            choices_no.push( Coord((-1,0)) );
            choices_no.push( Coord((1,0)) );
        } else if (dif.0).1 <= -1 {
            choices.push( Coord((0,-1)) );
            choices_no.push( Coord((0,1)) );
            choices_no.push( Coord((-1,0)) );
            choices_no.push( Coord((1,0)) );
        }

        rand::thread_rng().shuffle( & mut choices[..] );

        rand::thread_rng().shuffle( & mut choices_no[..] );
        choices.extend_from_slice( &choices_no[..] );
        
        choices
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


