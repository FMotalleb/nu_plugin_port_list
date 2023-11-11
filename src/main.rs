
use std::{net::IpAddr, collections::HashMap};
use netstat2::{AddressFamilyFlags, ProtocolFlags, get_sockets_info, ProtocolSocketInfo};
use nu_plugin::{self, EvaluatedCall, LabeledError};
use nu_protocol::{record, Category, PluginSignature, Span, Value, Record};

use sysinfo::{
   System, SystemExt, ProcessExt, Process,
};

pub struct Plugin;

impl nu_plugin::Plugin for Plugin {
    fn signature(&self) -> Vec<PluginSignature> {
        vec![PluginSignature::build("port list")
            .usage("list all active connections (tcp+udp)")
            .switch("disable-ipv4","do not fetch ivp6 connections (ipv6 only)",Some('6'))
            .switch("disable-ipv6","do not fetch ivp6 connections (ipv4 only)",Some('4'))
            .switch("disable-udp","do not fetch udp connections (tcp only)",Some('t'))
            .switch("disable-tcp","do not fetch tcp connections (udp only)",Some('u'))
            .switch("process-info","loads process info (name,cmd, binary path)",Some('p'))
            .category(Category::Experimental)]
    }

    fn run(
        &mut self,
        _name: &str,
        call: &EvaluatedCall,
        _input: &Value,
    ) -> Result<Value, LabeledError> {
        let mut af_flags= AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6 ;
        let mut proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;
        let skip_process_info=!call.has_flag("process-info");
        if call.has_flag("disable-ipv4") {
            af_flags=af_flags & AddressFamilyFlags::IPV6;
        }
        if call.has_flag("disable-ipv6") {
            af_flags=af_flags & AddressFamilyFlags::IPV4;
        } 
        if call.has_flag("disable-udp") {
            proto_flags=proto_flags & ProtocolFlags::TCP;
        }
        if call.has_flag("disable-tcp") {
            proto_flags=proto_flags & ProtocolFlags::UDP;
        }
        let mut process_list: HashMap<String, &Process>=HashMap::new();
        let sys=System::new_all();
        if skip_process_info!=true {
            sys.processes().into_iter().for_each(|(pid, process)| {
                process_list.insert(pid.to_owned().to_string(),process);
            });
        }
        
        let sockets_info = get_sockets_info(af_flags, proto_flags);
        let mut other: Vec<Value> = vec![];
        match sockets_info {
            Ok(sockets_info) => {
                for si in sockets_info {
                    match si.protocol_socket_info {
                        ProtocolSocketInfo::Tcp(tcp_si) =>{
                            other.push(Value::record(
                                load_process_info_into(  &mut record!{
                                    "type" => Value::string("tcp".to_string(),call.head),
                                    "ip_version" => get_ip_version(tcp_si.local_addr,call.head),
                                    "local_address" => Value::string(tcp_si.local_addr.to_string(),call.head),
                                    "local_port" => Value::int(tcp_si.local_port.into(),call.head),
                                    "remote_address" => Value::string(tcp_si.remote_addr.to_string(),call.head),
                                    "remote_port" => Value::int(tcp_si.remote_port.into(),call.head),
                                    "state" => Value::string(tcp_si.state.to_string(),call.head),
                                    "pid"=>load_pid(&si.associated_pids,call.head),
                                    
                                },&si.associated_pids,skip_process_info,call.head,&process_list), 
                                call.head)
                            )
                        }
                        ProtocolSocketInfo::Udp(udp_si) => {
                        other.push(Value::record(
                            load_process_info_into(  &mut record!{
                                    "type" => Value::string("udp".to_string(),call.head),
                                    "ip_version" => get_ip_version(udp_si.local_addr,call.head),
                                    "local_address" => Value::string(udp_si.local_addr.to_string(),call.head),
                                    "local_port" => Value::int(udp_si.local_port.into(),call.head),
                                    "remote_address" => Value::string("Unknown".to_string(),call.head),
                                    "remote_port" => Value::string("Unknown".to_string(),call.head),
                                    "state" => Value::string("Unknown".to_string(),call.head),
                                    "pid"=>load_pid(&si.associated_pids,call.head),
                                },&si.associated_pids,skip_process_info,call.head,&process_list),
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
fn load_pid(items: &Vec<u32>, span: Span) -> Value {
    let mut result: Vec<Value> = vec![];
    for i in items.iter() {
        let pid=i.to_owned();
       
        result.push( Value::int(pid.into(),span));
    }
    match result.len() {
        0 => Value::nothing(span),
        _ => result.first().unwrap().clone(),
    }
}

fn load_process_info_into(rec : &mut Record ,items: &Vec<u32>,skip: bool, span: Span,process_list: &HashMap<String, &Process>) -> Record {
    if skip {
        return rec.to_owned();
    }
    
    for i in items.iter() {
        let pid=i.to_owned();
        let process = process_list.get(&pid.to_string());
        if let Some(process_info) =  process {
            rec.push("name" , Value::string(process_info.name().to_string(), span));
            rec.push("cmd" , Value::string(process_info.cmd().join(" ").to_string(), span));
            rec.push("exe_path" , Value::string(process_info.exe().to_owned().to_str().unwrap_or("-").to_string(), span));
        }
        break;
        
    } 
    rec.to_owned()
}

fn get_ip_version(addr: IpAddr,span: Span) -> Value{
    match addr {
        IpAddr::V4(_) => Value::int(4, span),
        IpAddr::V6(_) => Value::int(6, span),
    }
}