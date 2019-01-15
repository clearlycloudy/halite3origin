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
        let exists = match self.invmap.get_mut(&player_id) {
            Some(items) => {
                items.insert( item_id, c );
                true
            },
            _ => {
                false
            }
        };
        if !exists {
            self.invmap.insert(player_id, HashMap::new() );
            self.invmap.get_mut(&player_id).unwrap().insert( item_id, c );
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
