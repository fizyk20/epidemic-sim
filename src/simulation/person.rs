use nalgebra::Vector2;
use rand::{
    distributions::{Distribution, Normal, Uniform},
    Rng,
};

use super::{clamp_f64, clamp_vec2, params::Params};

pub const RADIUS: f64 = 0.5;

#[derive(Debug, Clone, Copy, Default)]
pub struct Status {
    infected: Option<f64>, // simulation time when infected
    past_infected: bool,
    vaccinated: bool,
}

impl Status {
    pub fn infected(&self) -> Option<f64> {
        self.infected
    }

    pub fn past_infected(&self) -> bool {
        self.past_infected
    }

    pub fn vaccinated(&self) -> bool {
        self.vaccinated
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Person {
    position: Vector2<f64>,
    velocity: Vector2<f64>,
    status: Status,
}

impl Person {
    pub fn random<R: Rng>(rng: &mut R, space_size: (f64, f64), speed_stdev: f64) -> Person {
        let (size_x, size_y) = space_size;
        let position = Vector2::new(
            Uniform::new(RADIUS, size_x - RADIUS).sample(rng),
            Uniform::new(RADIUS, size_y - RADIUS).sample(rng),
        );
        let velocity = Vector2::new(
            Normal::new(0.0, speed_stdev).sample(rng),
            Normal::new(0.0, speed_stdev).sample(rng),
        );

        Person {
            position,
            velocity,
            status: Default::default(),
        }
    }

    pub fn overlaps(&self, other: &Person, box_size: (f64, f64)) -> bool {
        let pos_diff = clamp_vec2(self.position - other.position, box_size);
        pos_diff.dot(&pos_diff).sqrt() < RADIUS * 2.0
    }

    pub fn pos(&self) -> Vector2<f64> {
        self.position
    }

    pub fn vel(&self) -> Vector2<f64> {
        self.velocity
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn infect(&mut self, time: f64) {
        self.status.infected = Some(time);
    }

    pub fn vaccinate(&mut self) {
        self.status.vaccinated = true;
    }

    pub fn shift(&mut self, dt: f64, box_size: (f64, f64)) {
        self.position += self.velocity * dt;
        self.position.x = clamp_f64(self.position.x, box_size.0);
        self.position.y = clamp_f64(self.position.y, box_size.1);
    }

    pub fn set_vel(&mut self, vel: Vector2<f64>) {
        self.velocity = vel;
    }

    pub fn contact<R: Rng>(&mut self, time: f64, params: Params, other: Person, rng: &mut R) {
        if other.status.infected.is_some() {
            let draw = rng.gen::<f64>();
            let threshold = match (
                self.status.past_infected,
                self.status.vaccinated,
                other.status.vaccinated,
            ) {
                (false, false, false) => params.infection_prob_infected_to_general,
                (true, false, false) => params.infection_prob_infected_to_healed,
                (_, true, false) => params.infection_prob_infected_to_vaccinated,
                (false, false, true) => params.infection_prob_vaccinated_to_general,
                (true, false, true) => params.infection_prob_vaccinated_to_healed,
                (_, true, true) => params.infection_prob_vaccinated_to_vaccinated,
            };
            if draw < threshold {
                self.status.infected = Some(time);
            }
        }
    }

    pub fn update_status<R: Rng>(
        &mut self,
        time: f64,
        params: Params,
        dt: f64,
        rng: &mut R,
    ) -> bool {
        match self.status.infected {
            Some(infected) => {
                if rng.gen::<f64>() < params.death_rate * dt / params.infection_avg_duration {
                    return true;
                }
                let heal_prob = (time - infected) / params.infection_avg_duration - 0.7;
                if rng.gen::<f64>() < heal_prob {
                    self.status.infected = None;
                    self.status.past_infected = true;
                }
            }
            _ => (),
        }
        false
    }
}
