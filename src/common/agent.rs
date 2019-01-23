
use rand::Rng;
use mapping::mapraw::{ResourceMap, RawMaps, UnitMap, DropoffMap, Unit, *};
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
    pub create_dropoff: Option<Coord>,
}

impl Agent {

    pub fn update_state( & mut self, mapraw: & mut RawMaps, kernel_dim: usize ){
        match self.status {
            AgentStatus::Idle => {},
            AgentStatus::Mining => {

                //determine if agent needs to change state
                
                let mut rng = rand::thread_rng();
                let num_gen: f32 = rng.gen();
                
                if self.halite >= 975 ||
                    ( self.halite >= 900 && num_gen < 0.8 ) ||
                    ( self.halite >= 700 && self.halite < 900 && num_gen < 0.2 ) ||
                    ( num_gen < 0.0005 ) {
                        self.status = AgentStatus::MoveToDropoff;
                    }
            },
            AgentStatus::MoveToMine => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                let dist = self.pos.diff_wrap_around( &mine_pos, &mapraw.map_r.dim() );
                if dist.abs() <= kernel_dim as i32 / 2 {
                    self.status = AgentStatus::Mining;
                }
            },
            AgentStatus::MoveToDropoff => {
                let dropoff_pos = self.assigned_dropoff.expect("dropoff pos empty");
                if self.pos == dropoff_pos {
                    self.status = AgentStatus::Idle;
                }
            },
            _ => {},
        }

        match self.status {
            AgentStatus::Mining => {
                let mine_resource = mapraw.map_r.get( self.pos );
                
                //calculate to see if assigned resource area is depleted and need to be reassigned
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                let resource_remain = mapraw.map_r.area_remain( mine_pos, kernel_dim as i32 / 2 );
                if mine_resource < 10 && (resource_remain as f32 / (kernel_dim * kernel_dim) as f32) < 10. {
                    self.status = AgentStatus::Idle; //reassign
                }
            },
            _ => {},
        }
    }
    
    //return current pos and desired destination
    pub fn execute( & mut self, map_r: &ResourceMap ) -> (i32,Coord,Coord) {

        match self.status {
            AgentStatus::Mining => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                ( self.id,self.pos,mine_pos )
            },
            AgentStatus::MoveToMine => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
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
            AgentStatus::CreateDropoff => {
                let dropoff_pos = self.create_dropoff.expect("create dropoff pos empty");
                ( self.id, self.pos, dropoff_pos )
            },
            _ => {
                panic!("unexpected status");
            },
        }
    }

    pub fn assign_mine_locally( & mut self, rawmaps: & mut RawMaps, kernel_dim: usize ) {

        match self.status {
            AgentStatus::Mining => {},
            _ => { return },
        }

        let current_cell_halite = rawmaps.map_r.get( self.pos );
        if current_cell_halite > 20 {
            return
        }
        
        //trace out a square path and find cells that have halite amount above a threshold
        let mine_pos = self.assigned_mine.expect("mine pos empty");
        let mut p = mine_pos.0;
        let mut step_stop = 2;
        let mut p_n = (p.0+step_stop/2, p.1+step_stop/2);
        // log.log(&format!("p_n init: {:?}", p_n));
        let mut step_count = 0;
        #[derive(Clone,Copy)]
        enum TraceDir {
            L,R,U,D
        }
        let mut d = TraceDir::U;

        let mut cell_total = 0;
        while cell_total < kernel_dim * kernel_dim * 3 {

            if step_count >= step_stop {
                let new_d = { match d {
                    TraceDir::L => { TraceDir::D },
                    TraceDir::D => { TraceDir::R },
                    TraceDir::R => {
                        step_stop += 1;
                        p_n.0 = p.0 + (step_stop)/2;
                        p_n.1 = p.1 + (step_stop)/2;
                        TraceDir::U
                    },
                    TraceDir::U => { TraceDir::L },
                } };
                d = new_d;
                step_count = 0;
            }
            match d {
                TraceDir::L => { p_n.1 -= 1; },
                TraceDir::D => { p_n.0 += 1; },
                TraceDir::R => { p_n.1 += 1; },
                TraceDir::U => { p_n.0 -= 1; },
            };

            // log.log(&format!("p_n: {:?}, step count: {}", p_n, step_count));

            step_count += 1;
            let halite_in_cell = rawmaps.map_r.get( Coord( (p_n.0, p_n.1) ) );

            let mut rng = rand::thread_rng();
            let num_gen_2: f32 = rng.gen();

            if (halite_in_cell >= 750 && num_gen_2 < 0.85 ) ||
                (halite_in_cell >= 500 && halite_in_cell < 750 && num_gen_2 < 0.7) ||
                ( halite_in_cell >= 250 && halite_in_cell < 500 && num_gen_2 < 0.5) ||
                ( halite_in_cell >= 100 && halite_in_cell < 200 && num_gen_2 < 0.25) ||
                ( halite_in_cell >= 30 && halite_in_cell < 100 && num_gen_2 < 0.10) ||
                ( halite_in_cell < 30 && num_gen_2 < 0.01 ) {
                    match rawmaps.map_u.get( Coord( (p_n.0, p_n.1) ) ) {
                        Unit::None => {
                            let Coord( (y,x) ) = Coord::from( p_n ).modulo( &rawmaps.map_r.dim() );
                            
                            if let AgentStatus::Idle = self.status {
                                self.status = AgentStatus::MoveToMine;
                            }

                            self.assigned_mine = Some( Coord( (y,x) ) );

                            // log.log(&format!("agent after action change: {:?}", a));
                            break;
                        },
                        _ => {},
                    }
                }
        }
    }

}
