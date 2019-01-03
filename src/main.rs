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

fn schedule( queued: Vec<(usize,Coord,Coord)> ) -> Vec<(usize,Dir)> {
    unimplemented!();
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
    pub expected_next_pos: Coord,
}

impl Agent {

    fn get_halite_capacity( & self ) -> f32 {
        self.halite as f32 / 1000.
    }
    fn is_idle( & self ) -> bool {
        match self.status {
            Idle => { true },
            _ => { false },
        }
    }
    fn is_moving_to_mine( & self ) -> bool {
        match self.status {
            MoveToMine => { true },
            _ => { false },
        }
    }
    fn is_moving_to_dropoff( & self ) -> bool {
        match self.status {
            MoveToDropoff => { true },
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
    fn execute( & mut self ) -> (usize,Coord,Coord) {
        match self.status {
            Idle => {},
            Mining => {
                if self.halite >= 750 {
                    let mut rng = rand::thread_rng();
                    let num_gen: f32 = rng.gen();
                    if num_gen < 0.5 || self.halite >= 925 {
                        self.status = AgentStatus::MoveToDropoff;
                    }
                }
            },
            MoveToMine => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                if self.pos == mine_pos {
                    self.status = AgentStatus::Mining;
                }
            },
            MoveToDropoff => {
                let dropoff_pos = self.assigned_dropoff.expect("dropoff pos empty");
                if self.pos == dropoff_pos {
                    self.status = AgentStatus::MoveToMine;
                }
            },
        }
        match self.status {
            Idle => {
                ( self.id,self.pos,self.pos )
            },
            Mining => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                assert_eq!( mine_pos, self.pos );
                ( self.id,self.pos,mine_pos )
            },
            MoveToMine => {
                let mine_pos = self.assigned_mine.expect("mine pos empty");
                ( self.id,self.pos,mine_pos )
            },
            MoveToDropoff => {
                let dropoff_pos = self.assigned_dropoff.expect("dropoff pos empty");
                ( self.id,self.pos,dropoff_pos )
            },
        }
    }
}

fn synchronize_player_agents( player_agents: & HashMap<usize,Agent>, update: Vec<(usize,(i32,i32),usize)> ) -> ( HashMap<usize,Agent>, Vec<Agent> ) {
    let mut processed_ids = HashSet::new();
    let mut ret = HashMap::new();

    let update_ids = update.iter().map(|x| x.0).collect::<HashSet<_>>();
    
    for (id,(y,x),halite) in update {
        processed_ids.insert(id);
        let a = player_agents.get(&id).expect("agent not found");
        assert_eq!(id, a.id);
        let mut agent_updated = a.clone();
        agent_updated.halite = halite;
        assert_eq!( agent_updated.expected_next_pos, Coord((y,x)) );
        ret.insert(id, agent_updated);
    }

    let mut removed_agents = vec![];
    
    for i in player_agents.keys().cloned().collect::<HashSet<_>>().difference( &update_ids ) {
        let a = player_agents.get(&i).expect("agent not found");
        removed_agents.push( a.clone() );
    }
        
    ( ret, removed_agents )
}

fn plan_strategy( player_agents: & mut HashMap<usize,Agent>, map_r: &mapraw::ResourceMap, map_d: &mapraw::DropoffMap ) {
    let mut agent_action_change = HashSet::new();
    
    for (id,a) in player_agents.iter() {
        match a.status {
            Idle => {
                agent_action_change.insert(id);
            },
            _ =>{
                match a.assigned_mine {
                    None => { agent_action_change.insert(id); },
                    Some(x) => {
                        let (y,x) = (a.pos).0;
                        let resource_count = map_r.get( y, x );
                        if resource_count <= 100 {
                            let mut rng = rand::thread_rng();
                            let num_gen: f32 = rng.gen();
                            if num_gen < 0.75 {
                                agent_action_change.insert(id);
                            }
                        }
                    },
                }
            },
        }
    }

    //todo: find mining locations and assign to associated agents, update assigned dropoff locations as well
    
    
    unimplemented!();
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
    for _ in 0..num_players {
        input.read_and_parse_line();
        let player_id : usize = input.next();
        let shipyard_x : i32 = input.next();
        let shipyard_y : i32 = input.next();
        players.push( (player_id, shipyard_y, shipyard_x ) );
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

    log.borrow_mut().log(&format!("constants: {}", constants));
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

        //synchronize agent information
        for k in player_stats.keys() {
            let player_agents = rawmaps.map_u.get_player_agents( &k.0 );
            if !agents.contains_key( k ) {
                agents.insert( k.clone(), HashMap::new() );
            }
            let (updated_agents, removed) = synchronize_player_agents( agents.get( k ).unwrap(), player_agents );
            *agents.get_mut( k ).unwrap() = updated_agents;

            if !agents_removed.contains_key( k ) {
               agents_removed.insert( k.clone(), Default::default() );
            }
            *agents_removed.get_mut( k ).unwrap() = removed;
        }

        //update macro strategy, assign task to each worker
        plan_strategy( agents.get_mut(&Player(my_id)).expect("player agent"), &rawmaps.map_r, &rawmaps.map_d );
        
        //execute agent action
        let mut queued_movements = vec![];
        let mut my_agents = agents.get_mut( &Player(my_id) );
        for a in my_agents.iter_mut() {
            a.iter_mut().for_each(|( agent_id, agent )| {
                queued_movements.push( agent.execute() );
            });
        }

        //schedule agent movement
        let movements = schedule( queued_movements );

        //create new worker if necessary
        let create_new_agent = {
            //todo
            false
        };
        
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
