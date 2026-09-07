use std::time::{Duration, Instant};
use vello::Scene;
use fluxo::chart_component::render_chart;
use fluxo::components::{Component, ComponentType};
use quadrosrv::client::Client;
use quadrosrv::client::sensor_dto::SensorDto;
use crate::ui::EventDrivenPlugin;

pub(crate) struct SensorHandle {
    last_tick_call: Instant,
    update_duration: Duration,
    c: Client,
    dtos: Vec<SensorDto>,
}


impl SensorHandle {
    pub fn new() -> Self {
        SensorHandle {
            last_tick_call: Instant::now(),
            update_duration: Duration::from_secs(2),
            c: Client::default(),
            dtos: vec![],
        }
    }

    fn track(&mut self, dto: SensorDto) {
        if let Some(index) = self.dtos.iter().position(|x| x.name == dto.name && x.module == dto.module) {
            self.dtos[index] = dto;
        } else {
            self.dtos.push(dto);
        }
    }

    fn render_sh(&self, surface: &mut Scene, mut offset: u32, width: u32) -> u32 {
        for dto in &self.dtos {
            offset += render_chart(surface, dto.module.trim(), dto.values.iter().map(|x| *x as f64).collect(), offset, width);
        }
        offset
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
            if let Some(dto) = self.c.read() {
                self.last_tick_call = now;
                self.track(dto);
            }
        }
        Ok(())
    }

    fn get_component(&self) -> Option<&Self> {
        Some(self)
    }
}