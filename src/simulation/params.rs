use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Params {
    #[serde(default = "default_num_people")]
    pub num_people: usize,
    #[serde(default = "default_size")]
    pub size_x: f64,
    #[serde(default = "default_size")]
    pub size_y: f64,
    #[serde(default = "default_speed_stdev")]
    pub speed_stdev: f64,
    #[serde(default = "default_infected")]
    pub init_infected: usize,
    #[serde(default = "default_vaccinated")]
    pub init_vaccinated: usize,
    #[serde(default = "default_inf_to_gen")]
    pub infection_prob_infected_to_general: f64,
    #[serde(default = "default_inf_to_healed")]
    pub infection_prob_infected_to_healed: f64,
    #[serde(default = "default_inf_to_vacc")]
    pub infection_prob_infected_to_vaccinated: f64,
    #[serde(default = "default_vacc_to_gen")]
    pub infection_prob_vaccinated_to_general: f64,
    #[serde(default = "default_vacc_to_healed")]
    pub infection_prob_vaccinated_to_healed: f64,
    #[serde(default = "default_vacc_to_vacc")]
    pub infection_prob_vaccinated_to_vaccinated: f64,
    #[serde(default = "default_duration")]
    pub infection_avg_duration: f64,
    #[serde(default = "default_death_rate")]
    pub death_rate: f64,
}

fn default_num_people() -> usize {
    10000
}

fn default_size() -> f64 {
    300.0
}

fn default_speed_stdev() -> f64 {
    10.0
}

fn default_infected() -> usize {
    1
}

fn default_vaccinated() -> usize {
    0
}

fn default_inf_to_gen() -> f64 {
    0.1
}

fn default_inf_to_healed() -> f64 {
    0.02
}

fn default_inf_to_vacc() -> f64 {
    0.001
}

fn default_vacc_to_gen() -> f64 {
    0.06
}

fn default_vacc_to_healed() -> f64 {
    0.012
}

fn default_vacc_to_vacc() -> f64 {
    0.0006
}

fn default_duration() -> f64 {
    30.0
}

fn default_death_rate() -> f64 {
    0.02
}
