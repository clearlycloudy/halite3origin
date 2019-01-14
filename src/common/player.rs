#[derive(Hash,Eq,PartialEq,Clone,Copy)]
pub struct Player(pub usize);

#[derive(Debug)]
pub struct PlayerStats {
    pub score: usize,
    pub ships: usize,
    pub dropoffs: usize,
    pub score_accum_rate: f32,
    pub score_accum_window: i32,
}
