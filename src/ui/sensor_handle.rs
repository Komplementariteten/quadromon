use std::time::{Duration, Instant};
use crate::proc::{Processing, Value};
use crate::sensors::{Config, ResultWrapper};
use crate::ui::components::Component;
use crate::ui::EventDrivenPlugin;

pub(crate) struct SensorHandle {
    cfg: Config,
    last_tick_call: Instant,
    update_duration: Duration,
    processing: Option<Processing>,
    history: Vec<Value>,
    current: Vec<Value>
}


impl SensorHandle {
    pub fn new() -> Self {
        let cfg = Config::default();
        SensorHandle{
            cfg,
            last_tick_call: Instant::now(),
            update_duration: Duration::from_secs(2),
            processing: None,
            history: vec![],
            current: vec![]
        }
    }

    fn process(&mut self, results: Vec<ResultWrapper>) {
        if self.processing.is_none() {
            self.processing = Some(Processing::init(results, None))
        }
        let p = self.processing.as_mut().unwrap();
        p.process();
        self.history = p.hist.clone();
        self.current = p.res.clone();
    }
}

impl EventDrivenPlugin for SensorHandle {
    fn event_tick(&mut self) -> anyhow::Result<()>
    {
        let now = Instant::now();
        let elapsed = now - self.last_tick_call;
        if elapsed > self.update_duration {
            self.last_tick_call = now;
            let results = crate::sensors::sensor_read::read(&self.cfg.clone());
            self.process(results);
        }
        Ok(())
    }

    fn get_component(&self) -> Option<Component> {
        todo!()
    }
}
