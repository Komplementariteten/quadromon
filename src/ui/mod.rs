use crate::ui::app::QuadromonApp;
use anyhow::Result;
use winit::event_loop::EventLoop;

mod app;
mod components;

pub(crate) trait EventDrivenPlugin {
    fn event_tick(&mut self) -> anyhow::Result<()>;
}

pub fn run<Plugin: EventDrivenPlugin>(plugins: Vec<Plugin>) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        run_linux(plugins)
    }
    #[cfg(not(target_os = "linux"))]
    {
        panic!("Not implemented for this platform")
    }
}

fn run_linux<Plugin: EventDrivenPlugin>(plugins: Vec<Plugin>) -> Result<()> {
    let mut app = QuadromonApp::new(plugins);
    let event_loop = EventLoop::new()?;
    event_loop
        .run_app(&mut app)
        .expect("Could not run event loop");
    Ok(())
}
