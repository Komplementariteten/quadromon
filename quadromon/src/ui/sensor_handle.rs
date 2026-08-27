use std::time::{Duration, Instant};
use vello::Scene;
use vello::wgpu::naga::Module;
use fluxo::chart_component::render_chart;
use fluxo::components::{Component, ComponentType};
use crate::ui::EventDrivenPlugin;

/* 
pub(crate) struct SensorHandle {
    cfg: Module,
    last_tick_call: Instant,
    update_duration: Duration,
    processing: Option<Processing>,
    history: Vec<Value>,
    current: Vec<Value>
}


impl SensorHandle {
    pub fn new() -> Self {
        let cfg = Module::default();
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

    fn render_sh(&self, surface: &mut Scene, offset: u32, width: u32) -> u32 {
        let v = vec![10., 12., 14.1234567, 8., 13.];
        // let v = self.current.iter().map(|v| v.into()).collect::<Vec<_>>();
        render_chart(surface, self.cfg.module_name.trim(), v, offset, width)
    }
}

impl Component for SensorHandle {
    fn title(&self) -> String {
        todo!()
    }

    fn component_type(&self) -> ComponentType {
        todo!()
    }

    fn render(&self, surface: &mut Scene, offset: u32, width: u32) -> u32 {
        self.render_sh(surface, offset, width)
    }

    fn order(&self) -> i32 {
        todo!()
    }
}

impl EventDrivenPlugin for SensorHandle {

    type Component = Self;

    fn event_tick(&mut self) -> anyhow::Result<()>
    {
        let now = Instant::now();
        let elapsed = now - self.last_tick_call;
        if elapsed > self.update_duration {
            self.last_tick_call = now;
            let results = sensor_read::read(&self.cfg.clone());
            self.process(results);
        }
        Ok(())
    }

    fn get_component(&self) -> Option<&Self> {
        Some(self)
    }
}

*/