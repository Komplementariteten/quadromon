use vello::Scene;
use vello::peniko::FontData;

pub trait Component {
    fn title(&self) -> String;
    fn component_type(&self) -> ComponentType;

    fn render(&self, surface: &mut Scene, offset: u32, width: u32) -> u32;
    fn order(&self) -> i32;
}

pub enum ComponentType {
    Chart,
}
