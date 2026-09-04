//! Difficulty ranks, ported from `src/balance.lua`.
//!
//! Higher `fire_rate_mul` / `bullet_spd_mul` / `hp_mul` is harder. The agent
//! knobs run the other way round: a high `agent_rate_mul` means the AI shoots
//! *less* often, and a high `agent_eat_mul` means it blocks more.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Rank {
    Easy,
    Normal,
    Hard,
}

pub const RANKS: [Rank; 3] = [Rank::Easy, Rank::Normal, Rank::Hard];
pub const DEFAULT: Rank = Rank::Normal;

#[derive(Clone, Copy, Debug)]
pub struct Kit {
    pub hp_mul: f32,
    pub spd_mul: f32,
    pub fire_rate_mul: f32,
    pub bullet_spd_mul: f32,
    pub boss_hp_mul: f32,
    pub lives: i32,
    pub bombs: i32,
    pub power: i32,
    pub agent_dmg_mul: f32,
    pub agent_rate_mul: f32,
    pub agent_eat_mul: f32,
    pub agent_auto_fire: bool,
    pub token_rain_empty: f32,
    pub token_rain_busy: f32,
}

impl Rank {
    pub fn title(self) -> &'static str {
        match self {
            Rank::Easy => "EASY",
            Rank::Normal => "NORMAL",
            Rank::Hard => "HARD",
        }
    }

    pub fn sub(self) -> &'static str {
        match self {
            Rank::Easy => "TRAINING BUILD",
            Rank::Normal => "RELEASE BUILD",
            Rank::Hard => "DEBUG HELL",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Rank::Easy => 0,
            Rank::Normal => 1,
            Rank::Hard => 2,
        }
    }

    /// Anything out of range lands on the intended fight rather than failing.
    pub fn from_index(i: i32) -> Rank {
        match i {
            0 => Rank::Easy,
            2 => Rank::Hard,
            _ => DEFAULT,
        }
    }

    pub fn kit(self) -> Kit {
        match self {
            Rank::Easy => Kit {
                hp_mul: 0.80,
                spd_mul: 0.82,
                fire_rate_mul: 0.78,
                bullet_spd_mul: 0.82,
                boss_hp_mul: 0.62,
                lives: 5,
                bombs: 5,
                power: 2,
                agent_dmg_mul: 0.90,
                agent_rate_mul: 0.22,
                agent_eat_mul: 1.05,
                agent_auto_fire: true,
                token_rain_empty: 4.2,
                token_rain_busy: 7.2,
            },
            Rank::Normal => Kit {
                hp_mul: 1.20,
                spd_mul: 1.08,
                fire_rate_mul: 1.28,
                bullet_spd_mul: 1.18,
                boss_hp_mul: 1.00,
                lives: 3,
                bombs: 3,
                power: 1,
                agent_dmg_mul: 0.38,
                agent_rate_mul: 1.00,
                agent_eat_mul: 1.25,
                agent_auto_fire: false,
                token_rain_empty: 6.4,
                token_rain_busy: 10.5,
            },
            Rank::Hard => Kit {
                hp_mul: 1.65,
                spd_mul: 1.26,
                fire_rate_mul: 1.65,
                bullet_spd_mul: 1.42,
                boss_hp_mul: 1.55,
                lives: 2,
                bombs: 2,
                power: 1,
                agent_dmg_mul: 0.24,
                agent_rate_mul: 1.30,
                agent_eat_mul: 1.40,
                agent_auto_fire: false,
                token_rain_empty: 8.5,
                token_rain_busy: 14.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_range_ranks_fall_back_to_normal() {
        assert_eq!(Rank::from_index(-1), Rank::Normal);
        assert_eq!(Rank::from_index(9), Rank::Normal);
        assert_eq!(Rank::from_index(1), Rank::Normal);
    }

    #[test]
    fn index_round_trips() {
        for r in RANKS {
            assert_eq!(Rank::from_index(r.index() as i32), r);
        }
    }

    #[test]
    fn harder_ranks_are_harder_in_every_direction() {
        let e = Rank::Easy.kit();
        let n = Rank::Normal.kit();
        let h = Rank::Hard.kit();
        assert!(e.hp_mul < n.hp_mul && n.hp_mul < h.hp_mul);
        assert!(e.spd_mul < n.spd_mul && n.spd_mul < h.spd_mul);
        assert!(e.fire_rate_mul < n.fire_rate_mul && n.fire_rate_mul < h.fire_rate_mul);
        assert!(e.bullet_spd_mul < n.bullet_spd_mul && n.bullet_spd_mul < h.bullet_spd_mul);
        assert!(e.lives > n.lives && n.lives > h.lives);
        assert!(e.bombs > n.bombs && n.bombs > h.bombs);
    }

    #[test]
    fn only_easy_lets_the_agents_fire_at_will() {
        assert!(Rank::Easy.kit().agent_auto_fire);
        assert!(!Rank::Normal.kit().agent_auto_fire);
        assert!(!Rank::Hard.kit().agent_auto_fire);
    }
}
