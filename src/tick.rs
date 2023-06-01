use std::collections::HashMap;
use std::time::Instant;

struct TickHistoryData {
    start: Instant,
    end: Instant,
    duration: f64,
}

struct TickHistory {
    datas: Vec<TickHistoryData>,
    // archived: Vec<TickHistoryData>,
    acc: f64,
    average: f64,
    min: f64,
    max: f64,
}

impl TickHistory {
    pub fn new() -> Self {
        Self {
            datas: vec![],
            // archived: vec![],
            acc: 0.0,
            average: 0.0,
            min: 100.0,
            max: 0.0,
        }
    }
    pub fn add(&mut self, start: Instant, end: Instant) {
        let duration = end.duration_since(start.clone()).as_secs_f64();
        if self.min > duration {
            self.min = duration;
        }
        if self.max < duration {
            self.max = duration;
        }
        self.acc += duration;
        self.datas.push(TickHistoryData {
            start,
            end,
            duration,
        });
        self.average = self.acc / self.datas.len() as f64;
    }
    pub fn reset(&mut self) {
        self.acc = 0.0;
        self.average = 0.0;
        self.min = 100.0;
        self.max = 0.0;
        self.datas = vec![];
    }
}

enum TickState {
    Initialized,
    Running(Instant),
    Done,
}

pub type TickID = &'static str;

pub const TICK_FRAME_ID: &str = "Frame";
pub const TICK_DRAW_ID: &str = "Draw";
pub const TICK_RENDER_ID: &str = "Render";
pub const TICK_RENDER_EGUI_ID: &str = "EguiRender";

pub struct TickSystem {
    tick_listeners: Vec<TickID>,
    running_tick: HashMap<TickID, TickState>,
    tick_history: HashMap<TickID, TickHistory>,
    remaining_time: f64,
}

impl TickSystem {
    pub fn new() -> Self {
        Self {
            tick_listeners: vec![],
            running_tick: Default::default(),
            tick_history: Default::default(),
            remaining_time: 1.0,
        }
    }
    pub fn register_listener(&mut self, id: TickID) {
        if !self.tick_listeners.contains(&id) {
            self.tick_listeners.push(id);
            self.running_tick.insert(id, TickState::Initialized);
            self.tick_history.insert(id, TickHistory::new());
        }
    }
    pub fn start_tick(&mut self, id: TickID) {
        if let Some(state) = self.running_tick.get_mut(&id) {
            *state = TickState::Running(Instant::now());
        }
    }
    pub fn end_tick(&mut self, id: TickID) {
        if let Some(state) = self.running_tick.get_mut(&id) {
            *state = match state {
                TickState::Running(start) => {
                    let history = self.tick_history.get_mut(&id).unwrap();
                    history.add(*start, Instant::now());

                    TickState::Done
                }
                TickState::Initialized => TickState::Initialized,
                TickState::Done => TickState::Done,
            };
        }
    }

    pub fn duration_since_frame_start(&self) -> Option<f64> {
        if let Some(state) = self.running_tick.get(&TICK_FRAME_ID) {
            return match state {
                TickState::Running(since) => Some(Instant::now().duration_since(since.clone()).as_secs_f64()),
                _ => None
            };
        }
        None
    }

    pub fn debug_tick(&self, id: TickID) {
        if let Some(history) = self.tick_history.get(&id) {
            println!("({:2}) {:7} lasted {:5.3} ms, avg ± {:5.3} (-{:5.3}, +{:5.3})",
                     history.datas.len(),
                     id,
                     history.datas.last().unwrap().duration * 1000.,
                     history.average * 1000.,
                     (history.average - history.min) * 1000.,
                     (history.max - history.average) * 1000.,
            );
        }
    }
    pub fn debug_tick_iteration(&self) {
        if let Some(history) = self.tick_history.get(&TICK_FRAME_ID) {
            println!("{:3} FPS with, avg ± {:5.3} ms (-{:5.3}, +{:5.3})",
                     history.datas.len(),
                     history.average * 1000.,
                     (history.average - history.min) * 1000.,
                     (history.max - history.average) * 1000.,
            );
        }
    }

    pub fn reset(&mut self) {
        self.tick_history.values_mut().for_each(|history| history.reset());
        self.remaining_time = 1.0;
    }

    pub fn should_reset(&self) -> bool {
        self.remaining_time < 0.0
    }

    pub fn update_time(&mut self) {
        if let Some(history) = self.tick_history.get(&TICK_FRAME_ID) {
            self.remaining_time -= history.datas.last().unwrap().duration;
        }
    }
}
