use nalgebra::Vector2;
use rand::{
    distributions::{Distribution, Normal, Uniform},
    Rng,
};

use super::params::{INFECTION_DURATION, INFECTION_PROBABILITY};

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

    pub fn overlaps(&self, other: &Person) -> bool {
        let pos_diff = self.position - other.position;
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

    pub fn shift(&mut self, dt: f64) {
        self.position += self.velocity * dt;
    }

    pub fn set_vel(&mut self, vel: Vector2<f64>) {
        self.velocity = vel;
    }

    pub fn contact<R: Rng>(&mut self, time: f64, other: Person, rng: &mut R) {
        if other.status.infected.is_some() && rng.gen::<f64>() < INFECTION_PROBABILITY {
            self.status.infected = Some(time);
        }
    }

    pub fn update_status<R: Rng>(&mut self, time: f64, _rng: &mut R) {
        match self.status.infected {
            Some(infected) if infected < time - INFECTION_DURATION => {
                self.status.infected = None;
                self.status.past_infected = true;
            }
            _ => (),
        }
    }
}
