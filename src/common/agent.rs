
use rand::Rng;
use mapping::mapraw::{ResourceMap, RawMaps, UnitMap, DropoffMap};
use common::coord::Coord;

#[derive(Clone,Copy,Debug)]
pub enum AgentStatus {
    MoveToMine,
    MoveToDropoff,
    Mining,
    Idle,
    EndGame,
    CreateDropoff,
    // NewlyCreated,
}

impl Default for AgentStatus {
    fn default() -> AgentStatus {
        AgentStatus::Idle
    }
}

#[derive(Clone,Copy,Debug)]
pub struct Agent {
    pub assigned_mine: Option<Coord>,
    pub assigned_dropoff: Option<Coord>,
    pub status: AgentStatus,
    pub halite: usize,
    pub pos: Coord,
    pub id: i32,
    pub cooldown_mine: i32,
    pub cooldown_movetomine: i32,
}

impl Agent {

    pub fn reset_cooldown_mine( & mut self ) {
        self.cooldown_mine = 2;
    }
    pub fn reset_cooldown_movetomine( & mut self ) {
        self.cooldown_movetomine = 10;
    }
    pub fn tick_cooldown_mine( & mut self ) {
        self.cooldown_mine -= 1;
    }
    pub fn tick_cooldown_movetomine( & mut self ) {
        self.cooldown_movetomine -= 1;
    }
    pub fn cooldown_movetomine( & self ) -> i32 {
        self.cooldown_movetomine
    }
    pub fn cooldown_mine( & self ) -> i32 {
        self.cooldown_mine
    }
    
    pub fn get_halite_capacity( & self ) -> f32 {
        self.halite as f32 / 1000.
    }
    pub fn is_idle( & self ) -> bool {
        match self.status {
            AgentStatus::Idle => { true },
            _ => { false },
        }
    }
    pub fn is_moving_to_mine( & self ) -> bool {
        match self.status {
            AgentStatus::MoveToMine => { true },
            _ => { false },
        }
    }
    pub fn is_moving_to_dropoff( & self ) -> bool {
        match self.status {
            AgentStatus::MoveToDropoff => { true },
            _ => { false },
        }
    }    
    pub fn set_task_mine( & mut self, pos: Coord ) {
        self.assigned_mine = Some(pos);
    }
    pub fn set_task_dropoff( & mut self, pos: Coord ) {
        self.assigned_dropoff = Some(pos);
    }
    //return current pos and desired destination
    pub fn execute( & mut self, map_r: &ResourceMap ) -> (i32,Coord,Coord) {

        match self.status {
            AgentStatus::Idle => {},
            AgentStatus::Mining => {
                let mine_resource = map_r.get( self.pos );
                
                let mut rng = rand::thread_rng();
                let num_gen: f32 = rng.gen();
                
                if self.halite >= 950 ||
                    (self.cooldown_mine() <= 0 &&
                     ( self.halite >= 850 && num_gen < 0.2 ) ||
                     ( self.halite >= 500 && self.halite < 850 && num_gen < 0.02 ) ||
                     ( self.halite >= 200 && self.halite < 500 && num_gen < 0.01 )
                    ) {
                        self.status = AgentStatus::MoveToDropoff;
                } else if self.cooldown_mine() <= 0 && mine_resource < 20 {
                    let mut rng = rand::thread_rng();
                    let num_gen: f32 = rng.gen();
                    if num_gen < 0.95 {
                        self.status = AgentStatus::Idle; //wait to be assign a new mine location by planner
                    } else {
                        self.status = AgentStatus::MoveToDropoff;
                    }
                }
            },
            AgentStatus::MoveToMine => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                if self.pos == mine_pos {
                    self.status = AgentStatus::Mining;
                    self.reset_cooldown_mine();
                    self.cooldown_movetomine = 0;
                }
            },
            AgentStatus::MoveToDropoff => {
                let dropoff_pos = self.assigned_dropoff.expect("dropoff pos empty");
                if self.pos == dropoff_pos {
                    self.status = AgentStatus::MoveToMine;
                    self.reset_cooldown_movetomine();
                }
            },
            _ => {},
        }
        match self.status {
            AgentStatus::Mining => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                // assert_eq!( mine_pos, self.pos );
                self.tick_cooldown_mine();
                ( self.id,self.pos,mine_pos )
            },
            AgentStatus::MoveToMine => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                self.tick_cooldown_movetomine();
                ( self.id,self.pos,mine_pos )
            },
            AgentStatus::MoveToDropoff => {
                let dropoff_pos = self.assigned_dropoff.expect("dropoff pos empty");
                ( self.id,self.pos,dropoff_pos )
            },
            AgentStatus::Idle => {
                ( self.id,self.pos,self.pos )
            },
            AgentStatus::EndGame => {
                let dropoff_pos = self.assigned_dropoff.expect("dropoff pos empty");
                ( self.id,self.pos,dropoff_pos )
            },
            _ => {
                panic!("unexpected status");
            },
        }
    }
}
