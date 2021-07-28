mod params;
pub mod person;

use std::collections::HashSet;

use rand::{seq::SliceRandom, Rng};

use params::Params;
use person::*;

#[derive(Debug, Clone)]
pub struct Simulation {
    box_size: (f64, f64),
    time: f64,
    people: Vec<Person>,
    params: Params,
}

const MAX_STEP_DURATION: f64 = 0.05;

impl Simulation {
    pub fn new<R: Rng>(rng: &mut R, params: Params) -> Simulation {
        let mut people = vec![];
        let box_size = (params.size_x, params.size_y);
        for _ in 0..params.num_people {
            loop {
                let new_person = Person::random(rng, box_size, params.speed_stdev);
                let can_add = people
                    .iter()
                    .all(|other: &Person| !other.overlaps(&new_person));
                if can_add {
                    people.push(new_person);
                    break;
                }
            }
        }

        Simulation {
            box_size,
            time: 0.0,
            people,
            params,
        }
    }

    pub fn infect<R: Rng>(&mut self, n: usize, rng: &mut R) {
        let mut indices: Vec<_> = (0..self.people.len()).collect();
        indices.shuffle(rng);
        for index in indices.into_iter().take(n) {
            self.people[index].infect(self.time);
        }
    }

    pub fn vaccinate<R: Rng>(&mut self, n: usize, rng: &mut R) {
        let mut indices: Vec<_> = (0..self.people.len()).collect();
        indices.shuffle(rng);
        for index in indices.into_iter().take(n) {
            self.people[index].vaccinate();
        }
    }

    pub fn people(&self) -> &[Person] {
        &self.people
    }

    pub fn step<R: Rng>(&mut self, dt: f64, rng: &mut R) {
        let dt = dt.min(MAX_STEP_DURATION);

        self.move_people(dt);
        let collisions = self.find_collisions();
        self.apply_collisions(collisions, rng);

        self.time += dt;

        let mut dead = vec![];
        for (i, person) in self.people.iter_mut().enumerate() {
            if person.update_status(self.time, self.params, dt, rng) {
                dead.push(i);
            }
        }
        dead.sort();
        for index in dead.into_iter().rev() {
            self.people.remove(index);
        }
    }

    fn move_people(&mut self, dt: f64) {
        for person in &mut self.people {
            person.shift(dt);
        }
    }

    fn find_collisions(&self) -> Vec<Collision> {
        let mut result: Vec<Collision> = vec![];

        let mut sorted_x: Vec<usize> = (0..self.people.len()).collect();
        let mut sorted_y = sorted_x.clone();

        sorted_x.sort_by(|index1, index2| {
            self.people[*index1]
                .pos()
                .x
                .partial_cmp(&self.people[*index2].pos().x)
                .unwrap()
        });
        sorted_y.sort_by(|index1, index2| {
            self.people[*index1]
                .pos()
                .y
                .partial_cmp(&self.people[*index2].pos().y)
                .unwrap()
        });

        // collisions with the left wall
        for index in &sorted_x {
            let person = &self.people[*index];
            if person.pos().x < RADIUS {
                result.push(Collision::Wall(*index, Wall::Left));
            } else {
                break;
            }
        }

        // collisions with the right wall
        for index in sorted_x.iter().rev() {
            let person = &self.people[*index];
            if person.pos().x > self.box_size.0 - RADIUS {
                result.push(Collision::Wall(*index, Wall::Right));
            } else {
                break;
            }
        }

        // collisions with the top wall
        for index in &sorted_y {
            let person = &self.people[*index];
            if person.pos().y < RADIUS {
                result.push(Collision::Wall(*index, Wall::Top));
            } else {
                break;
            }
        }

        // collisions with the right wall
        for index in sorted_y.iter().rev() {
            let person = &self.people[*index];
            if person.pos().y > self.box_size.1 - RADIUS {
                result.push(Collision::Wall(*index, Wall::Bottom));
            } else {
                break;
            }
        }

        let mut pairs = HashSet::new();

        for (i, person_index) in sorted_x.iter().enumerate() {
            for j in i + 1..sorted_x.len() {
                let person1 = &self.people[*person_index];
                let person2 = &self.people[sorted_x[j]];
                if person1.overlaps(person2) {
                    if *person_index < sorted_x[j] {
                        pairs.insert((*person_index, sorted_x[j]));
                    } else {
                        pairs.insert((sorted_x[j], *person_index));
                    }
                } else if person2.pos().x - person1.pos().x > RADIUS {
                    break;
                }
            }
        }

        for (i, person_index) in sorted_y.iter().enumerate() {
            for j in i + 1..sorted_y.len() {
                let person1 = &self.people[*person_index];
                let person2 = &self.people[sorted_y[j]];
                if person1.overlaps(person2) {
                    if *person_index < sorted_y[j] {
                        pairs.insert((*person_index, sorted_y[j]));
                    } else {
                        pairs.insert((sorted_y[j], *person_index));
                    }
                } else if person2.pos().y - person1.pos().y > RADIUS {
                    break;
                }
            }
        }

        for (index1, index2) in pairs {
            result.push(Collision::People(index1, index2));
        }

        result
    }

    fn apply_collisions<R: Rng>(&mut self, collisions: Vec<Collision>, rng: &mut R) {
        for collision in collisions {
            match collision {
                Collision::Wall(index, Wall::Left) => {
                    let person = &mut self.people[index];
                    let mut vel = person.vel();
                    if vel.x < 0.0 {
                        vel.x = -vel.x;
                    }
                    person.set_vel(vel);
                }
                Collision::Wall(index, Wall::Right) => {
                    let person = &mut self.people[index];
                    let mut vel = person.vel();
                    if vel.x > 0.0 {
                        vel.x = -vel.x;
                    }
                    person.set_vel(vel);
                }
                Collision::Wall(index, Wall::Top) => {
                    let person = &mut self.people[index];
                    let mut vel = person.vel();
                    if vel.y < 0.0 {
                        vel.y = -vel.y;
                    }
                    person.set_vel(vel);
                }
                Collision::Wall(index, Wall::Bottom) => {
                    let person = &mut self.people[index];
                    let mut vel = person.vel();
                    if vel.y > 0.0 {
                        vel.y = -vel.y;
                    }
                    person.set_vel(vel);
                }
                Collision::People(index1, index2) => {
                    let (new_vel1, new_vel2) = {
                        let person1 = &self.people[index1];
                        let person2 = &self.people[index2];
                        let normal = (person2.pos() - person1.pos()).normalize();
                        let relative_vel = person1.vel() - person2.vel();
                        let vel_norm = relative_vel.dot(&normal);
                        let vel1 = person1.vel();
                        let vel2 = person2.vel();
                        if vel_norm > 0.0 {
                            (vel1 - vel_norm * normal, vel2 + vel_norm * normal)
                        } else {
                            (vel1, vel2)
                        }
                    };
                    self.people[index1].set_vel(new_vel1);
                    self.people[index2].set_vel(new_vel2);
                    let copy1 = self.people[index1].clone();
                    let copy2 = self.people[index2].clone();
                    self.people[index1].contact(self.time, self.params, copy2, rng);
                    self.people[index2].contact(self.time, self.params, copy1, rng);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Wall {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
enum Collision {
    People(usize, usize),
    Wall(usize, Wall),
}
