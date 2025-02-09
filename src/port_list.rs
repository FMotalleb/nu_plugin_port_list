use crate::helper::ToStr;
use crate::PortListPlugin;
use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{record, Category, LabeledError, PipelineData, Record, Signature, Span, Value};
use std::collections::HashMap;
use std::iter::Iterator;
use std::net::IpAddr;
use sysinfo::{Process, System};

pub struct PortList;

impl PortList {
    pub fn new() -> PortList {
        PortList {}
    }
    fn load_process_info_into(
        rec: &mut Record,
        items: &Vec<u32>,
        skip: bool,
        span: Span,
        process_list: &HashMap<String, &Process>,
    ) -> Record {
        if skip {
            return rec.to_owned();
        }

        for i in items.iter() {
            let pid = i.to_owned();
            let process = process_list.get(&pid.to_string());
            if let Some(process_info) = process {
                rec.push(
                    "process_name",
                    Value::string(process_info.name().to_string(), span),
                );
                rec.push(
                    "cmd",
                    Value::string(
                        process_info
                            .cmd()
                            .into_iter()
                            .map(|f| f.to_string())
                            .collect::<Vec<String>>()
                            .join(" "),
                        span,
                    ),
                );
                rec.push(
                    "exe_path",
                    Value::string(
                        process_info
                            .exe()
                            .map(|f| f.to_str().unwrap_or("-"))
                            .unwrap_or("-")
                            .to_string(),
                        span,
                    ),
                );
                rec.push(
                    "process_status",
                    Value::string(process_info.status().to_string(), span),
                );
                rec.push(
                    "process_user",
                    Value::string(
                        process_info
                            .user_id()
                            .map(|uid| uid.to_string())
                            .unwrap_or("-".to_string()),
                        span,
                    ),
                );
                rec.push(
                    "process_group",
                    Value::string(
                        process_info
                            .group_id()
                            .map(|gid| gid.to_string())
                            .unwrap_or("-".to_string()),
                        span,
                    ),
                );
                rec.push(
                    "process_effective_user",
                    Value::string(
                        process_info
                            .effective_user_id()
                            .map(|uid| uid.to_string())
                            .unwrap_or("-".to_string()),
                        span,
                    ),
                );
                rec.push(
                    "process_effective_group",
                    Value::string(
                        process_info
                            .effective_group_id()
                            .map(|gid| gid.to_string())
                            .unwrap_or("-".to_string()),
                        span,
                    ),
                );
                rec.push(
                    "process_environments",
                    Value::list(
                        Self::map_environments(
                            process_info
                                .environ()
                                .into_iter()
                                .map(|i| i.to_string())
                                .collect::<Vec<String>>(),
                            span,
                        ),
                        span,
                    ),
                )
            }
            break;
        }
        rec.to_owned()
    }

    fn get_ip_version(addr: IpAddr, span: Span) -> Value {
        match addr {
            IpAddr::V4(_) => Value::int(4, span),
            IpAddr::V6(_) => Value::int(6, span),
        }
    }
    fn map_environments(environments: Vec<String>, span: Span) -> Vec<Value> {
        let mut values: Vec<Value> = vec![];
        for i in environments {
            values.push(Value::string(i, span))
        }
        values
    }
    fn load_pid(items: &Vec<u32>, span: Span) -> Value {
        let mut result: Vec<Value> = vec![];
        for i in items.iter() {
            let pid = i.to_owned();

            result.push(Value::int(pid.into(), span));
        }
        match result.len() {
            0 => Value::nothing(span),
            _ => result.first().unwrap().clone(),
        }
    }
}
impl PluginCommand for PortList {
    type Plugin = PortListPlugin;

    fn name(&self) -> &str {
        "port list"
    }

    fn signature(&self) -> Signature {
        Signature::build("port list")
            .switch(
                "disable-ipv4",
                "do not fetch ipv6 connections (ipv6 only)",
                Some('6'),
            )
            .switch(
                "disable-ipv6",
                "do not fetch ipv4 connections (ipv4 only)",
                Some('4'),
            )
            .switch(
                "disable-udp",
                "do not fetch UDP connections (TCP only)",
                Some('t'),
            )
            .switch(
                "disable-tcp",
                "do not fetch TCP connections (UDP only)",
                Some('u'),
            )
            .switch(
                "listeners",
                "only listeners (equivalent to state == \"LISTEN\")",
                Some('l'),
            )
            .switch(
                "process-info",
                "loads process info (name, cmd, binary path)",
                Some('p'),
            )
            .category(Category::Network)
    }

    fn description(&self) -> &str {
        "Like netstat this command will return every open connection on the network interface"
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let mut af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        let mut proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;

        let skip_process_info = match call.has_flag("process-info") {
            Ok(value) => !value,
            Err(_) => false,
        };
        if let Ok(true) = call.has_flag("disable-ipv4") {
            af_flags = af_flags & AddressFamilyFlags::IPV6;
        }
        if let Ok(true) = call.has_flag("disable-ipv6") {
            af_flags = af_flags & AddressFamilyFlags::IPV4;
        }
        if let Ok(true) = call.has_flag("disable-udp") {
            proto_flags = proto_flags & ProtocolFlags::TCP;
        }
        if let Ok(true) = call.has_flag("disable-tcp") {
            proto_flags = proto_flags & ProtocolFlags::UDP;
        }
        let listeners_only = match call.has_flag("listeners") {
            Ok(true) => true,
            _ => false,
        };

        let mut process_list: HashMap<String, &Process> = HashMap::new();
        let sys = System::new_all();
        if skip_process_info != true {
            sys.processes().into_iter().for_each(|(pid, process)| {
                process_list.insert(pid.to_owned().to_string(), process);
            });
        }

        let sockets_info = get_sockets_info(af_flags, proto_flags);
        let mut other: Vec<Value> = vec![];
        match sockets_info {
            Ok(sockets_info) => {
                for si in sockets_info {
                    if listeners_only {
                        if let ProtocolSocketInfo::Tcp(tcp_si) = &si.protocol_socket_info {
                            if tcp_si.state != TcpState::Listen {
                                continue;
                            }
                        }
                    }

                    match si.protocol_socket_info {
                        ProtocolSocketInfo::Tcp(tcp_si) =>{
                            other.push(Value::record(
                                Self::load_process_info_into(  &mut record!{
                                    "type" => Value::string("tcp".to_string(),call.head),
                                    "ip_version" => Self::get_ip_version(tcp_si.local_addr,call.head),
                                    "local_address" => Value::string(tcp_si.local_addr.to_string(),call.head),
                                    "local_port" => Value::int(tcp_si.local_port.into(),call.head),
                                    "remote_address" => Value::string(tcp_si.remote_addr.to_string(),call.head),
                                    "remote_port" => Value::int(tcp_si.remote_port.into(),call.head),
                                    "state" => Value::string(tcp_si.state.to_string(),call.head),
                                    "pid"=>Self::load_pid(&si.associated_pids,call.head),

                                },&si.associated_pids,skip_process_info,call.head,&process_list),
                                call.head)
                            )
                        }
                        ProtocolSocketInfo::Udp(udp_si) => {
                            other.push(Value::record(
                                Self::load_process_info_into(  &mut record!{
                                    "type" => Value::string("udp".to_string(),call.head),
                                    "ip_version" => Self::get_ip_version(udp_si.local_addr,call.head),
                                    "local_address" => Value::string(udp_si.local_addr.to_string(),call.head),
                                    "local_port" => Value::int(udp_si.local_port.into(),call.head),
                                    "remote_address" => Value::string("".to_string(),call.head),
                                    "remote_port" => Value::int(-1,call.head),
                                    "state" => Value::string("LISTEN".to_string(),call.head),
                                    "pid"=>Self::load_pid(&si.associated_pids,call.head),
                                },&si.associated_pids,skip_process_info,call.head,&process_list),
                                call.head)
                            )
                        },
                    }
                }
            }
            Err(err) => {
                return Err(LabeledError::new(err.to_string()).with_code("sockets_info::fetch"))
            }
        };
        return Ok(PipelineData::Value(Value::list(other, call.head), None));
    }
}
