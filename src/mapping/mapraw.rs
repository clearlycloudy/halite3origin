use std::collections::HashMap;

pub struct RawMaps {
    pub map_r: ResourceMap,
    pub map_u: UnitMap,
    pub map_d: DropoffMap,
}

#[derive(Debug)]
pub struct ResourceMap {
    pub map: Vec<Vec<usize>>,
    pub dim: (i32,i32), //num rows, num columns
}

impl ResourceMap {
    pub fn get( & self, row: i32, col: i32 ) -> usize {
        let r = ( row % self.dim.0 + self.dim.0 ) % self.dim.0;
        let c = ( col % self.dim.1 + self.dim.1 ) % self.dim.1;
        self.map[r as usize][c as usize]
    }
    pub fn total_remain(& self) -> usize {
        let mut ret = 0;
        for i in self.map.iter() {
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
        id: usize,
        halite: usize
    },
    None,
}

impl Default for Unit {
    fn default() -> Unit {
        Unit::None
    }
}

#[derive(Default)]
pub struct UnitMap {
    pub map: Vec<Vec<Unit>>,
    pub dim: (i32,i32), //num rows, num columns
    pub invmap: HashMap<usize,HashMap<usize,(i32,i32)>>, //player_id -> unit id -> (y,x)
}

impl From<(i32,i32)> for UnitMap {
    fn from(dim: (i32,i32)) -> Self {
        Self {
            map: vec![ vec![ Unit::None; dim.1 as usize]; dim.0 as usize],
            dim: dim,
            invmap: HashMap::new(),
        }
    }
}

impl UnitMap {
    pub fn get( & self, row: i32, col: i32 ) -> Unit {
        let r = ( row % self.dim.0 + self.dim.0 ) % self.dim.0;
        let c = ( col % self.dim.1 + self.dim.1 ) % self.dim.1;
        self.map[r as usize][c as usize]
    }
    pub fn remove( & mut self, row: i32, col: i32 ) {
        let r = ( row % self.dim.0 + self.dim.0 ) % self.dim.0;
        let c = ( col % self.dim.1 + self.dim.1 ) % self.dim.1;
        match self.get( r, c ) {
            Unit::Ship { player, id, halite } => {
                if !self.invmap.contains_key( &player ) {
                    self.invmap.insert( player, HashMap::new() );
                }
                self.invmap.get_mut(&player).unwrap().remove( &id );
            },
            _ => {},
        }
        self.set( r, c, Unit::None );
    }
    pub fn set( & mut self, row: i32, col: i32, unit: Unit ) {
        let r = ( row % self.dim.0 + self.dim.0 ) % self.dim.0;
        let c = ( col % self.dim.1 + self.dim.1 ) % self.dim.1;
        self.map[r as usize][c as usize] = unit;
        match unit {
            Unit::Ship{ player, id,.. } => {
                if !self.invmap.contains_key( &player ) {
                    self.invmap.insert( player, HashMap::new() );
                }
                self.invmap.get_mut(&player).unwrap().insert( id, (r, c) );
            },
            _ => {},
        }
    }
    pub fn get_player_agents( & mut self, player_id : &usize) -> Vec<(usize,(i32,i32),usize)> {
        if let Some(agents) = self.invmap.get(player_id) {
            agents.iter().map(|x|{
                let unit = self.get((x.1).0,(x.1).1);
                let h = match unit {
                    Unit::Ship { halite,.. } => {
                        halite
                    },
                    _ => {panic!("unit type unexpected");},
                };
                (*x.0,*x.1,h) //return (id, (posy,posx), halite)

            }).collect()
        } else {
            vec![]
        }
    }
}

#[derive(Default)]
pub struct DropoffMap {
    pub map: Vec<Vec<Option<Player>>>, //player id
    pub dim: (i32,i32), //num rows, num columns
    pub invmap: HashMap<usize,HashMap<i32,(i32,i32)>>, //player_id -> dropoffid -> (y,x)
}

impl From<(i32,i32)> for DropoffMap {
    fn from(dim: (i32,i32)) -> Self {
        Self {
            map: vec![ vec![ None; dim.1 as usize]; dim.0 as usize],
            dim: dim,
            invmap: HashMap::new(),
        }
    }
}

#[derive(Clone,Copy,Default)]
pub struct Player(pub usize); //player id
    
impl DropoffMap {
    pub fn get( & self, row: i32, col: i32 ) -> Option<Player> {
        let r = ( row % self.dim.0 + self.dim.0 ) % self.dim.0;
        let c = ( col % self.dim.1 + self.dim.1 ) % self.dim.1;
        self.map[r as usize][c as usize]
    }
    pub fn set( & mut self, dropoff_id: i32, row: i32, col: i32, unit: Player ) {
        let r = ( row % self.dim.0 + self.dim.0 ) % self.dim.0;
        let c = ( col % self.dim.1 + self.dim.1 ) % self.dim.1;
        self.map[r as usize][c as usize] = Some(unit);
        if !self.invmap.contains_key(&unit.0) {
            self.invmap.insert(unit.0,HashMap::new());
        }
        self.invmap.get_mut(&unit.0).unwrap().insert(dropoff_id,(row,col));
    }
}
