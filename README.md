# nu_plugin_port_list

A [nushell](https://www.nushell.sh/) plugin to display all active network connections.
similar to `netstat -ntp`

**Important**: to list pid correctly it needs to run as a privileged user (root)

* flags

```bash
  -6, --disable-ipv4 - do not fetch ipv4 connections (ipv6 only)
  -4, --disable-ipv6 - do not fetch ipv6 connections (ipv4 only)
  -t, --disable-udp - do not fetch UDP connections (TCP only)
  -u, --disable-tcp - do not fetch TCP connections (UDP only)
  -p, --process-info - loads process info (process_name, cmd, binary path, ...)
```

# Examples

* list all open ports

```bash
~> port list
```

|type|ip_version|local_address|local_port|remote_address|remote_port|state|pid|
|-|-|-|-|-|-|-|-|
|tcp|4|0.0.0.0|22|0.0.0.0|0|LISTEN|1000|
|tcp|4|192.168.100.8|42352|...|780|ESTABLISHED|9343|
|tcp|4|192.168.100.8|60564|...|443|ESTABLISHED|2899|
|tcp|4|127.0.0.1|38946|127.0.0.1|7890|ESTABLISHED|3376|
|tcp|4|127.0.0.1|50180|127.0.0.1|37921|ESTABLISHED|7620|

* list all open tcp port that are in LISTEN state and using local address 0.0.0.0

 ```bash
~> port list | where state == LISTEN and local_address == 0.0.0.0
```

|type|ip_version|local_address|local_port|remote_address|remote_port|state|pid|
|-|-|-|-|-|-|-|-|
|tcp|4|0.0.0.0|7070|0.0.0.0|0|LISTEN|993|
|tcp|4|0.0.0.0|3306|0.0.0.0|0|LISTEN|9953|
|tcp|4|0.0.0.0|9000|0.0.0.0|0|LISTEN|1525|
|tcp|4|0.0.0.0|8585|0.0.0.0|0|LISTEN|10693|
|tcp|4|0.0.0.0|22|0.0.0.0|0|LISTEN|1000|

* get process that is listening on a port

```bash
~> port list -t4p
```

|type|ip_version|local_address|local_port|remote_address|remote_port|state|pid|process_name|cmd|exe_path|process_status|process_user|process_group|process_effective_user|process_effective_group|process_environments|
|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|
|tcp|4|127.0.0.1|631|0.0.0.0|0|LISTEN|986|cupsd|/usr/sbin/cupsd -l|/usr/sbin/cupsd|Sleeping|0|0|0|0|[LANG=en_US.UTF-8,...]|

# Installing

* using [nupm](https://github.com/nushell/nupm)

```bash
git clone https://github.com/FMotalleb/nu_plugin_port_list.git
nupm install --path nu_plugin_port_list -f
```

* or compile manually

```bash
git clone https://github.com/FMotalleb/nu_plugin_port_list.git
cd nu_plugin_port_list
cargo build -r
register target/release/nu_plugin_port_list
```

* or using cargo

```bash
cargo install nu_plugin_port_list
register ~/.cargo/bin/nu_plugin_port_list
```
