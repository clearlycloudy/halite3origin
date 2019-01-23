use std::collections::HashMap;

use common::coord::{Coord,Dir};

#[derive(Clone,Copy,Default)]
pub struct Player(pub usize); //player id

pub struct MapGeneric < T > where T: Clone + Copy + Default {
    pub map: Vec<Vec<T>>,
    pub dim: Coord,
    pub invmap: HashMap<usize,HashMap<i32,Coord>>, //player_id -> item_id -> (y,x) optional
}

impl < T > MapGeneric < T > where T: Clone + Copy + Default {
    pub fn new( dim: Coord ) -> Self {
        assert!( dim.x() >= 0 );
        assert!( dim.y() >= 0 );
        Self {
            map: vec![ vec![ T::default(); dim.x() as usize ]; dim.y() as usize ],
            dim: dim,
            invmap: Default::default(),
        }
    }
    pub fn get_map( & self, c: Coord ) -> T {
        let coord = c.modulo( &self.dim );
        assert!( (coord.y() as usize) < self.map.len() );
        assert!( (coord.x() as usize) < self.map[coord.y() as usize].len() );
        self.map[ coord.y() as usize][ coord.x() as usize]
    }
    pub fn set_map( & mut self, c: Coord, item: T ) {
        let coord = c.modulo( &self.dim );
        assert!( (coord.y() as usize) < self.map.len() );
        assert!( (coord.x() as usize) < self.map[coord.y() as usize].len() );        
        self.map[coord.y() as usize][coord.x() as usize] = item;
    }
    pub fn get_inv_map( & self, player_id: usize ) -> HashMap<i32, Coord> {
        if let Some(item) = self.invmap.get(&player_id) {
            item.clone()
        } else {
            HashMap::new()
        }
    }
    pub fn remove_item_from_inv_map( & mut self, player_id: usize, item_id: i32 ) -> bool {
        if let Some(items) = self.invmap.get_mut(&player_id) {
            items.remove( &item_id ).is_some()
        } else {
            false
        }
    }
    pub fn add_item_inv_map( & mut self, player_id: usize, item_id: i32, c: Coord ) {
        let coord = c.modulo( &self.dim );
        let exists = match self.invmap.get_mut(&player_id) {
            Some(items) => {
                items.insert( item_id, coord );
                true
            },
            _ => {
                false
            }
        };
        if !exists {
            self.invmap.insert(player_id, HashMap::new() );
            self.invmap.get_mut(&player_id).unwrap().insert( item_id, coord );
        }
    }
    pub fn clear_player_item_inv_map( & mut self, player_id: usize ) {
        for (item_id,coord) in self.get_inv_map( player_id ) {
            self.set_map( coord, T::default() );
            self.remove_item_from_inv_map( player_id, item_id );
        }
    }
}

pub struct RawMaps {
    pub map_r: ResourceMap,
    pub map_u: UnitMap,
    pub map_d: DropoffMap,
}

pub struct UnitMap {
    pub base: MapGeneric<Unit>,
}

pub struct ResourceMap {
    pub base: MapGeneric<usize>,
    //(y,x) -> halite amount
}

pub struct DropoffMap {
    pub base: MapGeneric< Option<Player> >,
    //(y,x) -> player_id
    //player_id -> dropoffid -> (y,x)
}

impl From<(i32,i32)> for ResourceMap {
    fn from(dim: (i32,i32)) -> Self {
        Self {
            base: MapGeneric::new( Coord( ( dim.0, dim.1 ) ) ),
        }
    }
}

impl ResourceMap {
    pub fn dim( & self ) -> Coord {
        self.base.dim
    }
    pub fn get( & self, c: Coord ) -> usize {
        self.base.get_map( c )
    }
    pub fn set( & mut self, c: Coord, halite: usize ) {
        self.base.set_map( c, halite );
    }
    pub fn total_remain(& self) -> usize {
        let mut ret = 0;
        for i in self.base.map.iter() {
            for j in i.iter() {
                ret += *j;
            }
        }
        ret
    }
    pub fn area_remain( & self, c: Coord, radius: i32 ) -> usize {
        let mut ret = 0;
        for y in c.y() - radius ..= c.y() + radius {
            for x in c.x() - radius ..= c.x() + radius {
                ret += self.get( Coord( (y,x) ) );
            }
        }
        ret
    }
    pub fn avg_remain(&self) -> f32 {
        self.total_remain() as f32 / (self.dim().y() * self.dim().x()) as f32
    }
}

#[derive(Clone,Copy)]
pub enum Unit {
    Ship {
        player: usize,
        id: i32,
        halite: usize
    },
    None,
}

impl Default for Unit {
    fn default() -> Unit {
        Unit::None
    }
}

impl From<(i32,i32)> for UnitMap {
    fn from(dim: (i32,i32)) -> Self {
        Self {
            base: MapGeneric::new( Coord( ( dim.0, dim.1 ) ) ),
        }
    }
}

impl UnitMap {
    pub fn dim( & self ) -> Coord {
        self.base.dim
    }
    pub fn get( & self, c: Coord ) -> Unit {
        self.base.get_map( c )
    }
    pub fn remove( & mut self, c: Coord ) {
        match self.get( c ) {
            Unit::Ship { player, id, halite } => {
                self.base.remove_item_from_inv_map( player, id );
            },
            _ => {},
        }
        self.set( c, Unit::None );
    }
    pub fn set( & mut self, c: Coord, unit: Unit ) {
        self.base.set_map( c, unit );
        match unit {
            Unit::Ship{ player, id,.. } => {
                self.base.add_item_inv_map( player, id, c );
            },
            _ => {},
        }
    }
    pub fn get_player_agents( & mut self, player_id: usize) -> Vec<(i32,Coord,usize)> {
        self.base.get_inv_map( player_id ).iter().map( |(item_id,coord)| {
            let h = match self.get(*coord) {
                Unit::Ship { halite,.. } => {
                    halite
                },
                _ => {panic!("unit type unexpected");},
            };
            (*item_id,*coord,h) //return (id, (posy,posx), halite)
        }).collect()
        
    }
    pub fn clear_player_agents( & mut self, player_id: usize ) {
        self.base.clear_player_item_inv_map( player_id );
    }
}

impl From<(i32,i32)> for DropoffMap {
    fn from(dim: (i32,i32)) -> Self {
        Self {
            base: MapGeneric::new( Coord( ( dim.0, dim.1 ) ) ),
        }
    }
}
    
impl DropoffMap {
    pub fn dim( & self ) -> Coord {
        self.base.dim
    }
    pub fn get( & self, c: Coord ) -> Option<Player> {
        self.base.get_map( c )
    }
    pub fn set( & mut self, dropoff_id: i32, c: Coord, player: Player ) {
        self.base.set_map( c, Some(player) );
        let player_id = player.0;
        self.base.add_item_inv_map( player_id, dropoff_id, c );
    }
    pub fn get_player_dropoffs( & self, player_id: usize) -> Vec<(i32,Coord)> {
        self.base.get_inv_map( player_id ).iter().map(|(dropoff_id,coord)| {
            (*dropoff_id,*coord) //return (dropoff_id, (posy,posx) )
        }).collect()
    }
}

use std::ops::Add;
use std::ops::Mul;

//add resource map filtering
pub fn convolution<T>( map: &Vec<Vec<T>>, dim: Coord, kernel: Vec<Vec<T>> ) -> Vec<Vec<T>>
where T: Add<Output=T> + Mul<Output=T> + Default + Clone + Copy
{
    assert!( kernel.len() > 0 );
    assert!( kernel[0].len() > 0 );
    let kernel_mid = kernel.len() as i32/2;
    let mut ret = vec![ vec![T::default(); dim.x() as usize]; dim.y() as usize ];
    for e1 in 0..dim.y() as usize {
        for e2 in 0..dim.x() as usize {
            let mut sum = T::default();
            for (idxi, i) in kernel.iter().enumerate() {
                for (idxj, j) in i.iter().enumerate() {
                    let offset = Coord( (-kernel_mid + e1 as i32, -kernel_mid + e2 as i32 ) ).modulo( &dim );
                    let coord = (offset + Coord( (idxi as i32, idxj as i32) )).modulo( &dim );
                    let v = map[coord.y() as usize][coord.x() as usize];
                    sum = sum + v * *j;
                }
            }
            ret[e1][e2] = sum;
        }
    }
    ret
}

use std::cmp;

pub fn get_max<T>( v: &Vec<Vec<T>>, num_loc: Option<usize> ) -> Vec<(Coord,T)>
    where T: PartialOrd + Default + Clone
{
    assert!( v.len() > 0 );
    let mut ret = vec![];
    for (idy, i) in v.iter().enumerate() {
        for (idx, j ) in i.iter().enumerate() {
            ret.push( ( Coord( (idy as _,idx as _) ), j.clone() ) );
        }
    }
    use rand::Rng;
    rand::thread_rng().shuffle( & mut ret[..] );
    
    ret.sort_unstable_by( |a,b| (b.1).partial_cmp(&a.1).unwrap() );
    if let Some(n) = num_loc {
        ret.truncate( n );
    }
    ret
}

#[derive(Debug,Copy,Clone)]
pub enum ConvolveKernelVal {
    Uniform,
    Gaussian,
}

pub fn get_kernel_builtin( ker_val: ConvolveKernelVal,  ker_size: usize ) -> Vec<Vec<f32>> {
    assert!( ker_size > 0 );
    let mut ret = vec![ vec![0.; ker_size]; ker_size ];
    match ker_val {
        ConvolveKernelVal::Uniform => {
            let val = 1./((ker_size * ker_size) as f32);
            for i in ret.iter_mut() {
                for j in i.iter_mut() {
                    *j = val;
                }
            }
        },
        ConvolveKernelVal::Gaussian => {
            let center = ker_size as f32 / 2.;
            for (idy,i) in ret.iter_mut().enumerate() {
                for (idx,j) in i.iter_mut().enumerate() {
                    let expo = -0.5*((idy as f32 - center).powi(2)+(idx as f32 - center).powi(2));
                    *j = 1./((2.*std::f32::consts::PI).sqrt()) * expo.exp();
                }
            }
        },
    }
    ret
}

pub fn get_best_locations( ker_val: ConvolveKernelVal,  ker_size: usize, map: &Vec<Vec<usize>>, num_loc: Option<usize> ) -> Vec<Coord> {
    assert!( map.len() > 0 );
    let kernel = get_kernel_builtin( ker_val,  ker_size );
    let dim = Coord( ( map.len() as _, map[0].len() as _ ) );
    let mut map_float = vec![ vec![0.; map[0].len()]; map.len() ];
    for (idy,i) in map.iter().enumerate(){
        for (idx,j) in i.iter().enumerate(){
            map_float[idy][idx] = *j as f32;
        }
    }
    let filt_map = convolution( &map_float, dim, kernel );
    
    let (coords,vals) : (Vec<_>,Vec<_>) = get_max( &filt_map, num_loc).iter().cloned().unzip();
    coords
}

pub enum PathAction {
    Subtract,
    Add,
}

pub fn get_approx_halite_in_path( halite_amount: f32, start: Coord, end: Coord, map: &RawMaps, path_action: PathAction, avg_map_resource: f32 ) -> f32 {

    let mut halite = halite_amount as f32;
    let mut s = start;
    if s == end {
        return halite
    }

    let mut get_first = false;

    // let halite_negative = if halite < 0. {
    //     halite
    // } else {
    //     0.
    // };
    
    while s != end {
        let dir = s.get_prioritized_dir( &end, map );
        let deposit = map.map_r.get( s );
        match path_action {
            PathAction::Subtract => {
                halite = halite - 0.1 * deposit as f32;
                if halite < 0. {
                    halite = 0.;
                    break;
                }
            },
            _ => {
                if !get_first {
                    halite = halite + deposit as f32;
                    get_first = true;
                } else {
                    halite = halite - 0.07 * deposit as f32;
                }
            },
        }
        s = (s + dir[0]).modulo(&map.map_r.dim() );
    }
    
    let dist = start.diff_wrap_around( &end, &map.map_r.dim()).abs();

    // if halite_negative < 0. {
    //     halite = halite_negative - (dist as f32) * (dist as f32) * 0.15 * map.map_r.avg_remain();
    // } else {
    //     halite = halite - (dist as f32) * (dist as f32) * 0.15 * map.map_r.avg_remain();
    // }
    
    // halite = halite - (map.map_r.avg_remain()).powf(2.*dist as f32); 
    // halite = halite - (0.25 * map.map_r.avg_remain() * dist as f32).exp();
    halite = halite - (0.1 * avg_map_resource * dist as f32).exp();
    
    
    halite
}
