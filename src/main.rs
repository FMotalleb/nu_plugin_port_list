
use netstat::{AddressFamilyFlags, ProtocolFlags, get_sockets_info, ProtocolSocketInfo};
use nu_plugin::{self, EvaluatedCall, LabeledError};
use nu_protocol::{record, Category, PluginSignature, Span, Value};

pub struct Plugin;

impl nu_plugin::Plugin for Plugin {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("port list")
            .usage("list all active connections (tcp+udp)")
            .category(Category::Experimental)]
    }

    fn run(
        &mut self,
        _name: &str,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
        let sockets_info = get_sockets_info(af_flags, proto_flags);
        let mut other: Vec<Value> = vec![];
        match sockets_info {
            Ok(sockets_info) => {
                for si in sockets_info {
                    match si.protocol_socket_info {
                        ProtocolSocketInfo::Tcp(tcp_si) =>{
                            other.push(Value::record(
                                record!{
                                    "type" => Value::string("tcp".to_string(),call.head),
                                    "local_address" => Value::string(tcp_si.local_addr.to_string(),call.head),
                                    "local_port" => Value::int(tcp_si.local_port.into(),call.head),
                                    "remote_address" => Value::string(tcp_si.remote_addr.to_string(),call.head),
                                    "remote_port" => Value::int(tcp_si.remote_port.into(),call.head),
                                    "state" => Value::string(tcp_si.state.to_string(),call.head),
                                    "pids"=>map_to_values(si.associated_pids,call.head)
                                }, 
                                call.head)
                            )
                        }
                        ProtocolSocketInfo::Udp(udp_si) => {
                        other.push(Value::record(
                                record!{
                                    "type" => Value::string("udp".to_string(),call.head),
                                    "local_address" => Value::string(udp_si.local_addr.to_string(),call.head),
                                    "local_port" => Value::int(udp_si.local_port.into(),call.head),
                                    "remote_address" => Value::string("Unknown".to_string(),call.head),
                                    "remote_port" => Value::string("Unknown".to_string(),call.head),
                                    "state" => Value::string("Unknown".to_string(),call.head),
                                    "pids"=>map_to_values(si.associated_pids,call.head)
                                },
                                call.head)
                            )
                        },
                    }
                }
            }
            Err(err) => {
                return Err(LabeledError {
                    label: "cannot list active sockets".to_string(),
                    msg: err.to_string(),
                    span: Some(call.head),
                })
            }
        }

        return Ok(Value::list(other, call.head));
    }
}
fn main() {
    nu_plugin::serve_plugin(&mut Plugin {}, nu_plugin::MsgPackSerializer {})
}
fn map_to_values(items: Vec<u32>, span: Span) -> Value {
    let mut result: Vec<Value> = vec![];
    for i in items.iter() {
        result.push(Value::int(i.to_owned().into(), span))
    }
    if result.len() == 1 {
        result.first().unwrap().clone()
    } else {
        Value::list(result, span)
    }
}
