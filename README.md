# nu_plugin_port_list

A [nushell](https://www.nushell.sh/) plugin to display all active network connections.
similar to `netstat -ntp`

**Important**: to list pid correctly it needs to run as a privileged user (root)

* flags

```bash
  -6, --disable-ipv4 - do not fetch ivp6 connections (ipv6 only)
  -4, --disable-ipv6 - do not fetch ivp6 connections (ipv4 only)
  -t, --disable-udp - do not fetch UDP connections (TCP only)
  -u, --disable-tcp - do not fetch TCP connections (UDP only)
  -p, --process-info - loads process info (name, cmd, binary path)
```

# Examples

* list all open ports
```bash
~>port list
╭───┬──────┬───────────────┬────────────┬────────────────┬─────────────┬────────┬────────────────╮
│ # │ type │ local_address │ local_port │ remote_address │ remote_port │ state  │      pids      │
├───┼──────┼───────────────┼────────────┼────────────────┼─────────────┼────────┼────────────────┤
│ 0 │ tcp  │ 127.0.0.1     │        631 │ 0.0.0.0        │           0 │ LISTEN │ [list 0 items] │
│ 1 │ tcp  │ 0.0.0.0       │       7070 │ 0.0.0.0        │           0 │ LISTEN │            973 │
│ 2 │ tcp  │ 0.0.0.0       │         22 │ 0.0.0.0        │           0 │ LISTEN │           1010 │
│ 3 │ tcp  │ 127.0.0.1     │       6463 │ 0.0.0.0        │           0 │ LISTEN │          46595 │
│ 4 │ tcp  │ 0.0.0.0       │       9000 │ 0.0.0.0        │           0 │ LISTEN │           1537 │
╰───┴──────┴───────────────┴────────────┴────────────────┴─────────────┴────────┴────────────────╯
```

 * list all open tcp port that are in LISTEN state and using local address 0.0.0.0

 ```bash
~> port list | where state == LISTEN and local_address == 0.0.0.0
╭───┬──────┬───────────────┬────────────┬────────────────┬─────────────┬────────┬──────╮
│ # │ type │ local_address │ local_port │ remote_address │ remote_port │ state  │ pids │
├───┼──────┼───────────────┼────────────┼────────────────┼─────────────┼────────┼──────┤
│ 0 │ tcp  │ 0.0.0.0       │       7070 │ 0.0.0.0        │           0 │ LISTEN │  973 │
│ 1 │ tcp  │ 0.0.0.0       │         22 │ 0.0.0.0        │           0 │ LISTEN │ 1010 │
│ 2 │ tcp  │ 0.0.0.0       │       9000 │ 0.0.0.0        │           0 │ LISTEN │ 1537 │
╰───┴──────┴───────────────┴────────────┴────────────────┴─────────────┴────────┴──────╯
```

* get process that is listening on a port

```bash
~> let pid = (port list | where local_port == 7890 | get pids | first)
~> ps | where pid == $pid
```

# Installing

* via git

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
