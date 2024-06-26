mod port_list;

use crate::port_list::PortList;
use nu_plugin::PluginCommand;

pub struct PortListPlugin;

impl nu_plugin::Plugin for PortListPlugin {
    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![Box::new(PortList::new())]
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }
}
fn main() {
    nu_plugin::serve_plugin(&mut PortListPlugin {}, nu_plugin::MsgPackSerializer {})
}
