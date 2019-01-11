extern crate rand;

mod hlt;
mod mapping;
mod metric;

use mapping::{mapraw};

use rand::Rng;
use rand::distributions::{Distribution,Uniform};
use rand::SeedableRng;
use rand::XorShiftRng;
use std::env;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::time::{Duration,Instant};
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::collections::{HashMap,HashSet};
use std::ops::{Add,Sub};

#[derive(Hash,Eq,PartialEq,Clone,Copy)]
struct Player(usize);

#[derive(Debug)]
struct PlayerStats {
    pub score: usize,
    pub ships: usize,
    pub dropoffs: usize,
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
struct Coord(pub(i32,i32)); //(y,x)

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

#[derive(Clone,Copy,Debug)]
struct Dir(pub (i32,i32)); //(y,x)

enum TaskLongTerm {
    Mine((usize,usize)),
}

#[derive(Clone,Copy,Debug)]
enum TaskShortTerm {
    Move(Dir),
    Stationary,
}

fn schedule( queued: Vec<(usize,Coord,Coord)>, map_r: &mapraw::ResourceMap, map_d: &mapraw::DropoffMap, map_u: & mut mapraw::UnitMap, is_end_game: &bool, my_id: &usize ) -> Vec<(usize,Dir)> {

    let map_dim = map_r.dim;
    
    let mut ret : Vec<(usize,Dir)> = vec![];
    
    for (id,from,to) in queued {

        if from == to {
            continue;
        }
        
        let mut dif = to-from;

        if (dif.0).0 > map_r.dim.0/2 {
            (dif.0).0 -= map_r.dim.0;
        }
        if (dif.0).0 < -map_r.dim.0/2 {
            (dif.0).0 += map_r.dim.0;
        }

        if (dif.0).1 > map_r.dim.1/2 {
            (dif.0).1 -= map_r.dim.1;
        }
        if (dif.0).1 < -map_r.dim.1/2 {
            (dif.0).1 += map_r.dim.1;
        }

        let mut choices_no = vec![];
        
        let mut choices = vec![];
        if (dif.0).0 >= 1 {
            choices.push( (from, Dir((1,0))) );
            choices_no.push( (from, Dir((-1,0))) );
            choices_no.push( (from, Dir((0,-1))) );
            choices_no.push( (from, Dir((0,1))) );
        } else if (dif.0).0 <= -1 {
            choices.push( (from, Dir((-1,0))) );
            choices_no.push( (from, Dir((1,0))) );
            choices_no.push( (from, Dir((0,-1))) );
            choices_no.push( (from, Dir((0,1))) );
        } else {

        }
        
        if (dif.0).1 >= 1 {
            choices.push( (from, Dir((0,1))) );
            choices_no.push( (from, Dir((0,-1))) );
            choices_no.push( (from, Dir((-1,0))) );
            choices_no.push( (from, Dir((1,0))) );
        } else if (dif.0).1 <= -1 {
            choices.push( (from, Dir((0,-1))) );
            choices_no.push( (from, Dir((0,1))) );
            choices_no.push( (from, Dir((-1,0))) );
            choices_no.push( (from, Dir((1,0))) );
        }

        rand::thread_rng().shuffle( & mut choices[..] );

        rand::thread_rng().shuffle( & mut choices_no[..] );
        choices.extend_from_slice( &choices_no[..] );


        let mut found = false;
        let choices_filtered = choices
            .iter()
            .filter_map(|(fr,dir)| {
                let y = (fr.0).0 + (dir.0).0;
                let x = (fr.0).1 + (dir.0).1;
                let agent = map_u.get( (fr.0).0, (fr.0).1 );
                if let mapraw::Unit::None = agent {
                    // panic!("ship not found at expected location: {:?}, map_u: {:?}", fr.0, map_u.invmap );
                    //already processed, skip it
                    None                    
                } else {
                    if let mapraw::Unit::None = map_u.get( y, x ) {
                        if found {
                            None
                        } else {
                            // map_u.set( (fr.0).0, (fr.0).1, mapraw::Unit::None );
                            map_u.set( y, x, agent );
                            found = true;
                            Some(dir)
                        }
                    } else {
                        if *is_end_game {
                            if let Some(dropoff_player_id) = map_d.get( y, x ) {
                                if dropoff_player_id.0 == *my_id {
                                    map_u.set( y, x, agent );
                                    found = true;
                                    Some(dir)
                                } else { None }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
            })
            .cloned()
            .collect::<Vec<_>>();

        // let choices_non_optimal_filtered = choices_non_optimal
        //     .iter()
        //     .filter_map(|(fr,dir)| {
        //         let y = (fr.0).0 + (dir.0).0;
        //         let x = (fr.0).1 + (dir.0).1;
        //         let agent = map_u.get( (fr.0).0, (fr.0).1 );
        //         if let mapraw::Unit::None = agent {
        //             panic!("ship not found at expected location: {:?}, map_u: {:?}", fr.0, map_u.invmap );
        //             //already processed, skip it
        //             // None
        //         } else {
        //             if let mapraw::Unit::None = map_u.get( y, x ) {
        //                 // map_u.set( (fr.0).0, (fr.0).1, mapraw::Unit::None );
        //                 map_u.set( y, x, agent );
        //                 Some(dir)    
        //             } else {
        //                 None
        //             }
        //         }
        //     })
        //     .cloned()
        //     .collect::<Vec<_>>();
            
        if choices_filtered.is_empty() {
            // if choices_non_optimal.is_empty() {
            //     continue;
            // } else {
            //     let mut rng = rand::thread_rng();
            //     let idx = rng.gen_range(0, choices_non_optimal_filtered.len());
            //     ret.push( ( id, choices_non_optimal_filtered[idx] ) );
            // }
        } else {   
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0, choices_filtered.len());
            // map_u.set( (from.0).0, (from.0).1, mapraw::Unit::None );
            ret.push( ( id, choices_filtered[idx] ) );
        }
    }
    
    ret
}

fn add_movement_cmd( shipid: &usize, dir: &Dir, cmd: & mut Vec<String> ) -> Result< (), & 'static str > {
    if (dir.0).0.abs() + (dir.0).1.abs() > 1 {
        Err( "Dir value invalid" )
    } else {
        match dir {
            &Dir((-1,0)) => {
                cmd.push( format!("m {} n", shipid ) );
            },
            &Dir((1,0)) => {
                cmd.push( format!("m {} s", shipid ) );
            },
            &Dir((0,-1)) => {
                cmd.push( format!("m {} w", shipid ) );
            },
            &Dir((0,1)) => {
                cmd.push( format!("m {} e", shipid ) );
            },
            _ => {},
        }
        Ok( () )
    }
}



#[derive(Clone,Copy,Debug)]
enum AgentStatus {
    MoveToMine,
    MoveToDropoff,
    Mining,
    Idle,
    EndGame,
}

impl Default for AgentStatus {
    fn default() -> AgentStatus {
        AgentStatus::Idle
    }
}

#[derive(Clone,Copy,Debug)]
struct Agent {
    pub assigned_mine: Option<Coord>,
    pub assigned_dropoff: Option<Coord>,
    pub status: AgentStatus,
    pub halite: usize,
    pub pos: Coord,
    pub id: usize,
    pub cooldown_mine: i32,
    pub cooldown_movetomine: i32,
    // pub expected_next_pos: Coord,
}

impl Agent {

    fn reset_cooldown_mine( & mut self ) {
        self.cooldown_mine = 2;
    }
    fn reset_cooldown_movetomine( & mut self ) {
        self.cooldown_movetomine = 15;
    }
    fn tick_cooldown_mine( & mut self ) {
        self.cooldown_mine -= 1;
    }
    fn tick_cooldown_movetomine( & mut self ) {
        self.cooldown_movetomine -= 1;
    }
    fn cooldown_movetomine( & self ) -> i32 {
        self.cooldown_movetomine
    }
    fn cooldown_mine( & self ) -> i32 {
        self.cooldown_mine
    }
    
    fn get_halite_capacity( & self ) -> f32 {
        self.halite as f32 / 1000.
    }
    fn is_idle( & self ) -> bool {
        match self.status {
            AgentStatus::Idle => { true },
            _ => { false },
        }
    }
    fn is_moving_to_mine( & self ) -> bool {
        match self.status {
            AgentStatus::MoveToMine => { true },
            _ => { false },
        }
    }
    fn is_moving_to_dropoff( & self ) -> bool {
        match self.status {
            AgentStatus::MoveToDropoff => { true },
            _ => { false },
        }
    }    
    fn set_task_mine( & mut self, pos: Coord ) {
        self.assigned_mine = Some(pos);
    }
    fn set_task_dropoff( & mut self, pos: Coord ) {
        self.assigned_dropoff = Some(pos);
    }
    //return current pos and desired destination
    fn execute( & mut self, map_r: &mapraw::ResourceMap, log: & mut hlt::log::Log ) -> (usize,Coord,Coord) {
            
        log.log(&format!("agent execute: {:?}", self));

        match self.status {
            AgentStatus::Idle => {},
            AgentStatus::Mining => {
                let mine_resource = map_r.get( (self.pos.0).0, (self.pos.0).1 );
                if self.halite >= 850 && self.cooldown_mine() <= 0 {
                    let mut rng = rand::thread_rng();
                    let num_gen: f32 = rng.gen();
                    if num_gen < 0.5 || self.halite >= 950 {
                        self.status = AgentStatus::MoveToDropoff;
                    }
                } else if self.cooldown_mine() <= 0 && mine_resource < 50 {
                    let mut rng = rand::thread_rng();
                    let num_gen: f32 = rng.gen();
                    if num_gen < 0.90 {
                        self.status = AgentStatus::Idle; //wait to be assign a new mine location by planner
                    } else {
                        self.status = AgentStatus::MoveToDropoff;
                    }
                }
            },
            AgentStatus::MoveToMine => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                if self.pos == mine_pos {
                    log.log(&format!(" ---- !! --- agent move to mine: {:?}", mine_pos));    
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
                // log.log(&format!("agent idle here"));
                ( self.id,self.pos,self.pos )
            },
            _ => { //end game
                let dropoff_pos = self.assigned_dropoff.expect("dropoff pos empty");
                ( self.id,self.pos,dropoff_pos )
            }
        }
    }
}

fn synchronize_player_agents( player_agents: & HashMap<usize,Agent>, update: Vec<(usize,(i32,i32),usize)> ) -> ( HashMap<usize,Agent>, Vec<Agent> ) {
    let mut processed_ids = HashSet::new();
    let mut ret = HashMap::new();

    let update_ids = update.iter().map(|x| x.0).collect::<HashSet<_>>();
    
    for (id,(y,x),halite) in update {
        processed_ids.insert(id);
        match player_agents.get(&id) {
            Some(a) => {
                assert_eq!(id, a.id);
                let mut agent_updated = a.clone();
                agent_updated.halite = halite;
                agent_updated.pos = Coord( (y,x) );
                ret.insert(id, agent_updated);
                // assert_eq!( agent_updated.expected_next_pos, Coord((y,x)) );
            },
            _ => {
                //new agent found
                let a = Agent {
                    assigned_mine: None,
                    assigned_dropoff: None,
                    status: AgentStatus::Idle,
                    halite: halite,
                    pos: Coord( (y,x) ),
                    id: id,
                    cooldown_mine: 0i32,
                    cooldown_movetomine: 0i32,
                };
                ret.insert(id, a);
            },
        }
    }

    let mut removed_agents = vec![];
    
    for i in player_agents.keys().cloned().collect::<HashSet<_>>().difference( &update_ids ) {
        let a = player_agents.get(&i).expect("agent not found");
        removed_agents.push( a.clone() );
    }
        
    ( ret, removed_agents )
}

fn plan_strategy( log: & mut hlt::log::Log, myid: &usize, player_agents: & mut HashMap<usize,Agent>, map_r: &mapraw::ResourceMap, map_d: &mapraw::DropoffMap, map_u: & mapraw::UnitMap ) {
    
    let mut agent_action_change = vec![];

    //find agents with mine resource amount below a threshold
    for (id,a) in player_agents.iter() {
        match a.status {
            AgentStatus::Idle => {
                agent_action_change.push(*id);
            },
            AgentStatus::MoveToDropoff =>{
                match a.assigned_mine {
                    None => { agent_action_change.push(*id); },
                    Some(x) => {
                        // let (y,x) = (a.pos).0;
                        if let Some(Coord((y,x))) = a.assigned_mine {
                            let resource_count = map_r.get( y, x );
                            let mut assign_new_mine = false;
                            if a.cooldown_movetomine() <= 0 && a.cooldown_mine() <= 0 {
                                assign_new_mine = true;
                            }
                            if ( resource_count < 50 && assign_new_mine )//  ||
                            // resource_count <= 50 {
                            {
                                let mut rng = rand::thread_rng();                            
                                let num_gen: f32 = rng.gen();
                                if num_gen < 0.5 {
                                    agent_action_change.push(*id);
                                }   
                            }   
                        }
                    },
                }
            },
            _ => {},
        }
    }

    //log.log(&format!("agent_action_change: {:?}", agent_action_change));

    // //todo: find mining locations and assign to associated agents, update assigned dropoff locations as well
    // for (dropoff_id,dropoff_pos) in map_d.invmap.get(myid).expect("player id not found for dropoff map").iter() {
        
    //     //trace out a square path and find cells that have halite amount above a threshold
        
    //     let mut p = dropoff_pos;
    //     let mut step_stop = 2;
    //     let mut p_n = (p.0+step_stop/2, p.1+step_stop/2);
    //     log.log(&format!("p_n init: {:?}", p_n));
    //     let mut step_count = 0;
    //     #[derive(Clone,Copy)]
    //     enum TraceDir {
    //         L,R,U,D
    //     }
    //     let mut d = TraceDir::U;

    //     let mut cell_processed = 0;
    //     let mut cell_total = 0;
    //     while cell_processed < agent_action_change.len() && cell_total < map_r.dim.0 * map_r.dim.1 {
            
    //         if step_count >= step_stop {
    //             let new_d = { match d {
    //                 TraceDir::L => { TraceDir::D },
    //                 TraceDir::D => { TraceDir::R },
    //                 TraceDir::R => {
    //                     step_stop += 1;
    //                     p_n.0 = p.0 + (step_stop)/2;
    //                     p_n.1 = p.1 + (step_stop)/2;
    //                     TraceDir::U
    //                 },
    //                 TraceDir::U => { TraceDir::L },
    //             } };
    //             d = new_d;
    //             step_count = 0;
    //         }
    //         match d {
    //             TraceDir::L => { p_n.1 -= 1; },
    //             TraceDir::D => { p_n.0 += 1; },
    //             TraceDir::R => { p_n.1 += 1; },
    //             TraceDir::U => { p_n.0 -= 1; },
    //         };

    //         log.log(&format!("p_n: {:?}, step count: {}", p_n, step_count));
            
    //         step_count += 1;
    //         let halite_in_cell = map_r.get( p_n.0, p_n.1 );

    //         let mut rng = rand::thread_rng();
    //         let num_gen_2: f32 = rng.gen();
            
    //         if (halite_in_cell >= 750 && num_gen_2 < 0.75 ) ||
    //             (halite_in_cell >= 500 && halite_in_cell < 750 && num_gen_2 < 0.35) ||
    //             ( halite_in_cell >= 250 && halite_in_cell < 500 && num_gen_2 < 0.015) ||
    //             ( halite_in_cell >= 100 && halite_in_cell < 200 && num_gen_2 < 0.005) ||
    //             ( halite_in_cell >= 50 && halite_in_cell < 100 && num_gen_2 < 0.0005) ||
    //             ( halite_in_cell < 50 && num_gen_2 < 0.00005 ) {
    //             match map_u.get( p_n.0, p_n.1 ) {
    //                 mapraw::Unit::None => {
    //                     // log.log(&format!("agent_action_change assign cell: {:?}", agent_action_change));
    //                     let y = ( p_n.0 % (map_r.dim).0 + (map_r.dim).0 ) % (map_r.dim).0;
    //                     let x = ( p_n.1 % (map_r.dim).1 + (map_r.dim).1 ) % (map_r.dim).1;
    //                     let a_id = agent_action_change.pop().expect("agent_action empty");
    //                     let mut a = player_agents.get_mut(&a_id).expect("agent id not found");
    //                     let mut processed = false;
    //                     if let AgentStatus::Idle = a.status {
    //                         a.status = AgentStatus::MoveToMine;
    //                         a.reset_cooldown_movetomine();
    //                     }

    //                     a.assigned_mine = Some( Coord( (y,x) ) );
    //                     a.assigned_dropoff = Some( Coord( (dropoff_pos.0,dropoff_pos.1) ) );
    //                     processed = true;
                        
    //                     log.log(&format!("agent after action change: {:?}", a));
    //                     if processed {
    //                         cell_processed += 1;
    //                     }
    //                 },
    //                 _ => {},
    //             }
    //         }
    //         cell_total += 1;
    //     }
    // }

    //assign each associated agent with a new location to mine
    for a_id in agent_action_change.iter() {
        let mut a = player_agents.get_mut(a_id).expect("agent id not found");
        //trace out a square path and find cells that have halite amount above a threshold
        
        let mut p = a.pos.0;
        let mut step_stop = 2;
        let mut p_n = (p.0+step_stop/2, p.1+step_stop/2);
        log.log(&format!("p_n init: {:?}", p_n));
        let mut step_count = 0;
        #[derive(Clone,Copy)]
        enum TraceDir {
            L,R,U,D
        }
        let mut d = TraceDir::U;

        let mut cell_processed = 0;
        let mut cell_total = 0;
        while cell_processed < agent_action_change.len() && cell_total < map_r.dim.0 * map_r.dim.1 {
            
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

            log.log(&format!("p_n: {:?}, step count: {}", p_n, step_count));
            
            step_count += 1;
            let halite_in_cell = map_r.get( p_n.0, p_n.1 );

            let mut rng = rand::thread_rng();
            let num_gen_2: f32 = rng.gen();
            
            if (halite_in_cell >= 750 && num_gen_2 < 0.75 ) ||
                (halite_in_cell >= 500 && halite_in_cell < 750 && num_gen_2 < 0.35) ||
                ( halite_in_cell >= 250 && halite_in_cell < 500 && num_gen_2 < 0.015) ||
                ( halite_in_cell >= 100 && halite_in_cell < 200 && num_gen_2 < 0.005) ||
                ( halite_in_cell >= 50 && halite_in_cell < 100 && num_gen_2 < 0.0005) ||
                ( halite_in_cell < 50 && num_gen_2 < 0.00005 ) {
                match map_u.get( p_n.0, p_n.1 ) {
                    mapraw::Unit::None => {
                        let y = ( p_n.0 % (map_r.dim).0 + (map_r.dim).0 ) % (map_r.dim).0;
                        let x = ( p_n.1 % (map_r.dim).1 + (map_r.dim).1 ) % (map_r.dim).1;
                        let mut processed = false;
                        if let AgentStatus::Idle = a.status {
                            a.status = AgentStatus::MoveToMine;
                            a.reset_cooldown_movetomine();
                        }

                        a.assigned_mine = Some( Coord( (y,x) ) );

                        //assign a drop off point for the agent
                        let mut best_dropoff = None;
                        let mut min_dropoff_norm = std::i32::MAX;
                        for (dropoff_id,dropoff_pos) in map_d.invmap.get(myid).expect("player id not found for dropoff map").iter() {
                            let coord_diff = Coord( (dropoff_pos.0, dropoff_pos.1) ) - Coord( (y,x) );
                            let diff_norm = (coord_diff.0).0.abs() + (coord_diff.0).1.abs();
                            if diff_norm < min_dropoff_norm {
                                min_dropoff_norm = diff_norm;
                                best_dropoff = Some( Coord( (dropoff_pos.0, dropoff_pos.1) ) );
                            }
                        }
                        
                        a.assigned_dropoff = Some( best_dropoff.unwrap() );
                        processed = true;
                        
                        log.log(&format!("agent after action change: {:?}", a));
                        if processed {
                            cell_processed += 1;
                        }
                    },
                    _ => {},
                }
            }
            cell_total += 1;
        }
    }
}

fn determine_create_new_agent( player_stats: &HashMap< Player, PlayerStats >, my_id: &usize, map_u: &mapping::mapraw::UnitMap, shipyard_pos: &HashMap<usize,Coord>, turn_num: &usize, max_turn: &usize, is_end_game: &bool ) -> bool {

    let my_shipyard_pos = shipyard_pos.get( my_id ).expect("shipyard position not found");
    
    let pos_empty = if let mapraw::Unit::None = map_u.get( (my_shipyard_pos.0).0, (my_shipyard_pos.0).1 ) {
        true
    } else {
        false
    };
        
    let create = match player_stats.get( &Player(*my_id) ) {
        Some(stats) => {
            if (stats.score > 1000 + turn_num * 5 ) && stats.ships < 17 && pos_empty  && !*is_end_game && *turn_num <= (*max_turn * 7 ) / 10 {
                true
            } else {
                false
            }
        },
        _ => { false },
    };

    create
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rng_seed: u64 = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    };
    let seed_bytes: Vec<u8> = (0..16).map(|x| ((rng_seed >> (x % 8)) & 0xFF) as u8).collect();
    let mut rng: XorShiftRng = SeedableRng::from_seed([
        seed_bytes[0], seed_bytes[1], seed_bytes[2], seed_bytes[3],
        seed_bytes[4], seed_bytes[5], seed_bytes[6], seed_bytes[7],
        seed_bytes[8], seed_bytes[9], seed_bytes[10], seed_bytes[11],
        seed_bytes[12], seed_bytes[13], seed_bytes[14], seed_bytes[15]
    ]);

    let mut log = Rc::new(RefCell::new(hlt::log::Log::new()));
    let mut input = hlt::input::Input::new(&log);

    //inputs:
    //
    //constants
    //number of players
    //my id
    //for each player:
    //  player id, shipyard x, shipyard y
    //map width, map height
    //resource map of halite:
    //  [0][0] [0][1]... (1st row)
    //  [1][0] [1][1]...
    //  ...
        
    let constants = hlt::constants::Constants::new(log.borrow_mut().deref_mut(), &input.read_and_return_line());
    input.read_and_parse_line();
    let num_players : usize = input.next();
    let my_id : usize = input.next();
    let mut players = vec![];
    let mut shipyard_pos = HashMap::new();
    for _ in 0..num_players {
        input.read_and_parse_line();
        let player_id : usize = input.next();
        let shipyard_x : i32 = input.next();
        let shipyard_y : i32 = input.next();        
        players.push( (player_id, shipyard_y, shipyard_x ) );
        shipyard_pos.insert( player_id, Coord((shipyard_y, shipyard_x)) );
    }
    log.borrow_mut().open(my_id);
    input.read_and_parse_line();    
    let map_w : i32 = input.next();
    let map_h : i32 = input.next();
    let mut rawmaps;
    {
        let mut v = vec![ vec![ 0; map_w as usize]; map_h as usize];
        for i in 0..map_h as usize {
            input.read_and_parse_line();
            for j in 0..map_w as usize {
                let amount : usize = input.next();
                v[i][j] = amount;
            }
        }
        let rm = mapping::mapraw::ResourceMap { map: v, dim: (map_h, map_w) };
        rawmaps = mapping::mapraw::RawMaps {
            map_r: rm,
            map_u: Default::default(),
            map_d: Default::default(),
        };
    }

    log.borrow_mut().log(&format!("shipyards: {:?}", players ));
    
    log.borrow_mut().log(&format!("constants: {}", constants));
    log.borrow_mut().log(&format!("max turns: {}", constants.max_turns));
    log.borrow_mut().log(&format!("num players: {}", num_players));
    log.borrow_mut().log(&format!("my id: {}", my_id));
    log.borrow_mut().log(&format!("map width: {}", map_w));
    log.borrow_mut().log(&format!("map height: {}", map_h));

    log.borrow_mut().flush();
    println!("origin");

    log.borrow_mut().log(&format!("Successfully created bot! My Player ID is {}. Bot rng seed is {}.", my_id, rng_seed));

    let mut player_stats : HashMap< Player, PlayerStats > = Default::default();

    let mut agents : HashMap<Player, HashMap<usize,Agent> > = HashMap::new();

    let mut agents_removed : HashMap<Player, Vec<Agent> > = HashMap::new();
    
    loop {

        //input:
        //
        //turn num
        //for each player:
        //  player_id  num_ships num_dropoffs halite_amount
        //  ship_id1 coord_x coord_y halite_value
        //  ship_id2 coord_x coord_y halite_value
        //  ..
        //  (num_ships)
        //  dropoff_id1 coord_x coord_y
        //  dropoff_id2 coord_x coord_y
        //  ..
        //  (num_dropoffs)
        //
        //map_update_count
        //
        let t_start = Instant::now();

        rawmaps.map_d = mapping::mapraw::DropoffMap::from( (map_h,map_w) );
        rawmaps.map_u = mapping::mapraw::UnitMap::from( (map_h,map_w) );
        
        input.read_and_parse_line();
        let turn_num : usize = input.next();

        log.borrow_mut().log(&format!("turn {} -------------------------------------", turn_num ));
        
        for _ in 0..num_players {

            input.read_and_parse_line();
            let player_id : usize = input.next();
            let num_ships : usize = input.next();
            let num_dropoffs : usize = input.next();
            let player_halite : usize = input.next();
            
            player_stats.insert( Player(player_id), PlayerStats{ score: player_halite, ships: num_ships, dropoffs: num_dropoffs } );
            
            for _ in 0..num_ships {
                input.read_and_parse_line();
                let ship_id : usize = input.next();
                let x : i32 = input.next();
                let y : i32 = input.next();
                let ship_halite : usize = input.next();

                if player_id == my_id {
                    log.borrow_mut().log(&format!("ship id: {}, y: {}, x: {}", ship_id, y, x));
                }
                
                rawmaps.map_u.set( y, x, mapping::mapraw::Unit::Ship{ player: player_id, id: ship_id, halite: ship_halite } );
            }

            for _ in 0..num_dropoffs {
                input.read_and_parse_line();
                let dropoff_id : usize = input.next();
                let x : i32 = input.next();
                let y : i32 = input.next();
                rawmaps.map_d.set( dropoff_id as i32, y, x, mapping::mapraw::Player( player_id ) );
            }

            //also count shipyard as a dropoff point
            for i in players.iter() {
                let id = i.0;
                let y = i.1;
                let x = i.2;
                rawmaps.map_d.set( -1i32, y, x, mapping::mapraw::Player( id ) );
            }
            
        }
            
        input.read_and_parse_line();
        let map_update_count : usize = input.next();
        log.borrow_mut().log(&format!("resource update count: {}", map_update_count));        
        for _ in 0..map_update_count {
            input.read_and_parse_line();
            let x : usize = input.next();
            let y : usize = input.next();
            let halite_amount : usize = input.next();
            rawmaps.map_r.map[y][x] = halite_amount;
            log.borrow_mut().log(&format!("resource update [{}][{}]: {}", y,x,halite_amount));
        }

        log.borrow_mut().log(&format!("unit map: {:?}", rawmaps.map_u.invmap ));
        
        //synchronize agent information
        for k in player_stats.keys() {
            
            let player_agents = rawmaps.map_u.get_player_agents( &k.0 );
            if !agents.contains_key( k ) {
                agents.insert( k.clone(), HashMap::new() );
            }
            let (updated_agents, removed) = synchronize_player_agents( agents.get( k ).unwrap(), player_agents );
            log.borrow_mut().log(&format!("player {}: agents updated count: {}", k.0, updated_agents.len() ));
            *agents.get_mut( k ).unwrap() = updated_agents;
                                 
            if !agents_removed.contains_key( k ) {
               agents_removed.insert( k.clone(), Default::default() );
            }
            log.borrow_mut().log(&format!("player {}: agents removed count: {}", k.0, removed.len() ));
            *agents_removed.get_mut( k ).unwrap() = removed;
        }

        //update macro strategy, assign task to each worker
        let mut is_end_game = false;
        if constants.max_turns - turn_num <= (rawmaps.map_r.dim.0 * 6 / 10) as usize {
            is_end_game = true;
            for a in agents.get_mut(&Player(my_id)).expect("player agent") {
                let mut shortest_dist = 99999999;
                let mut dest = None;
                for (dropoff_id,dropoff_pos) in rawmaps.map_d.invmap.get(&my_id).expect("player id not found for dropoff map").iter() {
                    let dist = (((a.1.pos.0).0)-dropoff_pos.0).abs() + (((a.1.pos.0).1)-dropoff_pos.1).abs();
                    if shortest_dist > dist {
                        shortest_dist = dist;
                        dest = Some(*dropoff_pos);
                    }
                }
                a.1.assigned_dropoff = Some(Coord(dest.unwrap()));
                a.1.status = AgentStatus::EndGame;
            }
        } else {
            plan_strategy( & mut log.borrow_mut(), &my_id, agents.get_mut(&Player(my_id)).expect("player agent"), &rawmaps.map_r, &rawmaps.map_d, &rawmaps.map_u );
        }

        log.borrow_mut().log(&format!("agents: {:?}", agents.get_mut(&Player(my_id)).expect("player agent") ) );
        
        //execute agent action
        let mut queued_movements = vec![];
        let mut my_agents = agents.get_mut( &Player(my_id) );
        for a in my_agents.iter_mut() {
            a.iter_mut().for_each(|( agent_id, agent )| {
                queued_movements.push( agent.execute( &rawmaps.map_r, & mut log.borrow_mut() ) );
            });
        }

        log.borrow_mut().log(&format!("queued movement: {:?}", queued_movements ) );
            
        //todo: schedule agent movement
        let movements = schedule( queued_movements, &rawmaps.map_r, &rawmaps.map_d, & mut rawmaps.map_u, &is_end_game, &my_id );

        log.borrow_mut().log(&format!("inspecting scheduled movements:") );
        movements.iter().inspect(|x| log.borrow_mut().log(&format!("{:?}",x)) );
            
        //create new worker if necessary
        let create_new_agent = determine_create_new_agent( &player_stats, &my_id, &rawmaps.map_u, &shipyard_pos, &turn_num, &constants.max_turns, &is_end_game );
        
        //emit commands
        let mut command_queue: Vec<String> = vec![];
        
        for (id,dir) in movements {
            add_movement_cmd( &id, &dir, & mut command_queue ).expect("add movement failed");
        }
        if create_new_agent {
            command_queue.push( format!("g") );
        }

        for i in command_queue.drain(..) {
            log.borrow_mut().log(&format!("turn {}, command: {}", turn_num, i));
            print!("{} ", i);
        }
        println!();
        
        //log time
        let mut t_elapsed = t_start.elapsed();
        let t_elapsed_nanos = t_elapsed.subsec_nanos() as u64;
        let t_elapsed_ms = t_elapsed.as_secs() * 1000 + t_elapsed_nanos / 1000000;
        log.borrow_mut().log(&format!("turn {} elapsed time: {}", turn_num, t_elapsed_ms));
    }
}
