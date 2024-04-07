mod port_list;

use nu_plugin::{  PluginCommand};
use crate::port_list::PortList;

pub struct PortListPlugin;

impl nu_plugin::Plugin for PortListPlugin {
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin=Self>>> {
        vec![
            Box::new(PortList::new())
        ]
    }
}
fn main() {
    nu_plugin::serve_plugin(&mut PortListPlugin {}, nu_plugin::MsgPackSerializer {})
}

