extern crate rand;

mod hlt;
mod mapping;
mod planning;
mod metric;
mod common;
mod cmd;

use cmd::cmd::add_and_flush_cmds;
use mapping::{mapraw};
use planning::{plan::{schedule,
                      plan_strategy_around_dropoff,
                      plan_strategy,
                      plan_strategy_new,
                      determine_create_dropoff,
                      determine_create_new_agent,
                      synthesize_agent_actions},
               sync::synchronize_player_agents};

use common::{coord::{Coord, Vector, Dir},
             agent::{Agent,AgentStatus},
             player::{Player, PlayerStats}};

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

    let mut created_agent_last_turn = false;
    
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

            if let None = player_stats.get( &Player(player_id) ) {
                player_stats.insert( Player(player_id), PlayerStats{ score: player_halite, ships: num_ships, dropoffs: num_dropoffs, score_accum_rate: 0f32, score_accum_window: 50i32 } );
            } else {
                match player_stats.get_mut( &Player(player_id) ){
                    Some(cur_stat) => {
                        cur_stat.score = player_halite;
                        cur_stat.ships = num_ships;
                        cur_stat.dropoffs = num_dropoffs;
                        let diff_halite = if created_agent_last_turn {
                            // player_halite - cur_stat.score + 1000
                            player_halite - cur_stat.score
                        } else {
                            player_halite - cur_stat.score
                        };
                        cur_stat.score_accum_rate = ( (cur_stat.score_accum_window - 1) as f32 * cur_stat.score_accum_rate + diff_halite as f32 ) / cur_stat.score_accum_window as f32;
                    },
                    _ => {},
                }
            }

            
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

        //agent implementation starts here -------------------------------------------------------------------
        
        //update macro strategy, assign task to each worker
        let is_end_game = plan_strategy_new( & mut log.borrow_mut(),
                                               turn_num,
                                               constants.max_turns,
                                               & my_id,
                                               & mut agents,
                                               & rawmaps );
        
        //crate agent actions
        let ( queued_movements, queued_new_dropoffs ) = synthesize_agent_actions( & mut log.borrow_mut(),
                                                                                    turn_num,
                                                                                    constants.max_turns,
                                                                                    & my_id,
                                                                                    & mut agents,
                                                                                    &rawmaps );                
        // log.borrow_mut().log(&format!("queued movement: {:?}", queued_movements ) );

        let movements = schedule( queued_movements,
                                  & rawmaps.map_r,
                                  & rawmaps.map_d,
                                  & mut rawmaps.map_u,
                                  & is_end_game, &my_id );
        // log.borrow_mut().log(&format!("inspecting scheduled movements:") );
        // movements.iter().inspect(|x| log.borrow_mut().log(&format!("{:?}",x)) );
            
        //create new worker if necessary
        let create_new_agent = determine_create_new_agent( & player_stats,
                                                             & my_id,
                                                             & rawmaps.map_u,
                                                             & shipyard_pos,
                                                             & turn_num,
                                                             & constants.max_turns,
                                                             & is_end_game );

        created_agent_last_turn = create_new_agent;
        
        //emit commands
        add_and_flush_cmds( & turn_num,
                             & mut log.borrow_mut(),
                             movements.as_slice(),
                             queued_new_dropoffs.as_slice(),
                             & create_new_agent );
        
        //log time
        let mut t_elapsed = t_start.elapsed();
        let t_elapsed_nanos = t_elapsed.subsec_nanos() as u64;
        let t_elapsed_ms = t_elapsed.as_secs() * 1000 + t_elapsed_nanos / 1000000;
        log.borrow_mut().log(&format!("turn {} elapsed time: {}", turn_num, t_elapsed_ms));
    }
}
