use rand::Rng;
use rand::distributions::{Distribution,Uniform};
use rand::SeedableRng;
use std::collections::{HashMap,HashSet};
use std::borrow::BorrowMut;

use hlt;
use mapping::mapraw::{ResourceMap, RawMaps, UnitMap, DropoffMap, Unit};
use common::agent::{Agent,AgentStatus};
use common::coord::{Coord,Dir};

pub fn add_and_flush_cmds( turn_num: &usize,
                           log: & mut hlt::log::Log, // cmd: & mut Vec<String>,
                           movements: &[(i32,Dir)],
                           new_dropoffs: &[i32],
                           create_agent: &bool,
                           ship_id_create_dropoff: Option<i32> ) {

    let mut cmd = vec![];
    
    for (id,dir) in movements.iter() {
        add_movement_cmd( &id, &dir, & mut cmd, ship_id_create_dropoff ).expect("add movement failed");
    }

    for id in new_dropoffs.iter() {
        log.borrow_mut().log(&format!("creating new dropoff") );
        add_dropoff_cmd( *id, & mut cmd );
    }
    
    if *create_agent {
        cmd.push( format!("g") );
    }

    for i in cmd.drain(..) {
        log.borrow_mut().log(&format!("turn {}, command: {}", turn_num, i));
        print!("{} ", i);
    }
    println!("");
}

fn add_movement_cmd( shipid: &i32,
                     dir: &Dir,
                     cmd: & mut Vec<String>,
                     ship_id_create_dropoff: Option<i32> )
                     -> Result< (), & 'static str > {
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
            &Dir((0,0)) => {
                if let Some(x) = ship_id_create_dropoff {
                    if *shipid == x {
                        cmd.push( format!("c {}", shipid ) );
                    }
                }
            }
            _ => {},
        }
        Ok( () )
    }
}


fn add_dropoff_cmd( shipid: i32,
                    cmd: & mut Vec<String> )
                    -> Result< (), & 'static str > {
    cmd.push( format!("c {}", shipid ) );
    Ok( () )
}
