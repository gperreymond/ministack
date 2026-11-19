// Schema and validation of the cluster configuration file.
//
// The templates still receive the raw yaml values: this module only checks the file
// before rendering, so a typo (an unknown key) or an inconsistent cluster is reported
// with a clear message instead of a silently ignored setting or a cluster that never starts.

// the fields describe the schema, most of them are only read by the templates
#![allow(dead_code)]

use serde::Deserialize;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::Path;

// nomad servers are .11+, consul servers .21+, vault .30, nomad clients .101+
const MAX_NOMAD_SERVERS: usize = 9;
const MAX_CONSUL_SERVERS: usize = 9;
const MAX_CLIENT_INDEX: u64 = 154;
const VAULT_MODES: [&str; 2] = ["dev", "raft"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub name: String,
    pub datacenter: String,
    #[serde(default)]
    pub hashibase: Option<Hashibase>,
    #[serde(default)]
    pub network: Option<Network>,
    #[serde(default)]
    pub plugins: Option<Vec<String>>,
    #[serde(default)]
    pub services: Option<Services>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hashibase {
    pub repository: Option<String>,
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub subnet: Option<String>,
    pub external: Option<Enabled>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Enabled {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Services {
    pub consul: Option<Consul>,
    pub vault: Option<Vault>,
    pub nomad: Option<Nomad>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acl {
    #[serde(default)]
    pub enabled: bool,
    pub token: Option<String>,
}

// a published port on the host: a number, or false to not publish it
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum HostPort {
    Port(u16),
    Flag(bool),
}

impl HostPort {
    fn port(&self) -> Option<u16> {
        match self {
            HostPort::Port(0) | HostPort::Flag(_) => None,
            HostPort::Port(port) => Some(*port),
        }
    }
}

// ------------------------------------------------------------------ consul

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consul {
    #[serde(default)]
    pub enabled: bool,
    pub version: Option<String>,
    pub config: Option<ConsulConfig>,
    pub servers: Option<Vec<ConsulServer>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsulConfig {
    pub acl: Option<Acl>,
    #[serde(default)]
    pub tls: bool,
    pub log_level: Option<String>,
    pub server: Option<ConsulServerConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsulServerConfig {
    pub bootstrap_expect: Option<u64>,
    pub host_port: Option<HostPort>,
    pub labels: Option<Vec<String>>,
    pub local_volumes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConsulServer {
    pub name: String,
    pub ports: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub local_volumes: Option<Vec<String>>,
}

// ------------------------------------------------------------------ vault

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vault {
    #[serde(default)]
    pub enabled: bool,
    pub version: Option<String>,
    pub mode: Option<String>,
    pub config: Option<VaultConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultConfig {
    #[serde(default)]
    pub tls: bool,
    pub root_token: Option<String>,
    pub host_port: Option<HostPort>,
    pub labels: Option<Vec<String>>,
}

// ------------------------------------------------------------------ nomad

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Nomad {
    #[serde(default)]
    pub enabled: bool,
    pub version: Option<String>,
    pub config: Option<NomadConfig>,
    pub servers: Option<Vec<NomadServer>>,
    pub clients: Option<Vec<NomadClient>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NomadConfig {
    pub acl: Option<Acl>,
    #[serde(default)]
    pub tls: bool,
    pub bind_addr: Option<String>,
    pub log_level: Option<String>,
    pub server: Option<NomadServerConfig>,
    pub client: Option<NomadClientConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NomadServerConfig {
    pub bootstrap_expect: Option<u64>,
    pub host_port: Option<HostPort>,
    pub retry_join: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub local_volumes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NomadClientConfig {
    pub retry_join: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub local_volumes: Option<Vec<String>>,
    pub docker_volumes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NomadServer {
    pub name: String,
    pub ports: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub local_volumes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NomadClient {
    pub name: String,
    pub loop_index: Option<u64>,
    pub ports: Option<Vec<String>>,
    pub labels: Option<Vec<String>>,
    pub local_volumes: Option<Vec<String>>,
    pub docker_volumes: Option<Vec<String>>,
}

// ------------------------------------------------------------------ validation

// Parse and check the configuration, every problem found is returned
pub fn validate(content: &str, config_dir: &Path) -> Result<Config, Vec<String>> {
    let config: Config = serde_yaml::from_str(content).map_err(|e| vec![e.to_string()])?;
    let errors = check(&config, config_dir);
    if errors.is_empty() {
        Ok(config)
    } else {
        Err(errors)
    }
}

// letters, digits and '-' (and '_' when allowed), starting with a letter or a digit:
// used in container names, hostnames, volume names and certificates
fn is_valid_name(value: &str, allow_underscore: bool) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || (allow_underscore && c == '_'))
}

fn is_uuid(value: &str) -> bool {
    let groups: Vec<&str> = value.split('-').collect();
    let lengths = [8, 4, 4, 4, 12];
    groups.len() == lengths.len()
        && groups
            .iter()
            .zip(lengths)
            .all(|(group, length)| group.len() == length && group.chars().all(|c| c.is_ascii_hexdigit()))
}

fn check(config: &Config, config_dir: &Path) -> Vec<String> {
    let mut errors = Vec::new();

    // ---- cluster
    if !is_valid_name(&config.name, true) || config.name.chars().any(|c| c.is_ascii_uppercase()) {
        errors.push(format!(
            "name: '{}' must use lowercase letters, digits, '-' or '_' (it is the docker compose project name)",
            config.name
        ));
    }
    if !is_valid_name(&config.datacenter, false) {
        errors.push(format!(
            "datacenter: '{}' must use letters, digits or '-' (it prefixes the container names)",
            config.datacenter
        ));
    }

    // ---- network
    let external_network = config
        .network
        .as_ref()
        .and_then(|n| n.external.as_ref())
        .map(|e| e.enabled)
        .unwrap_or(false);
    if let Some(subnet) = config.network.as_ref().and_then(|n| n.subnet.as_ref()) {
        match subnet.parse::<Ipv4Addr>() {
            Ok(ip) if ip.octets()[3] == 0 => {}
            _ => errors.push(format!(
                "network.subnet: '{}' must be an ipv4 network address ending with .0, e.g. '10.1.0.0' (a /24 is created)",
                subnet
            )),
        }
    }

    // ---- plugins: docker compose files, relative to the configuration file
    for plugin in config.plugins.iter().flatten() {
        if !config_dir.join(plugin).is_file() {
            errors.push(format!(
                "plugins: '{}' does not exist (the path is relative to the configuration file)",
                plugin
            ));
        }
    }

    // ---- nodes: names are unique container names '<datacenter>-<name>'
    let services = config.services.as_ref();
    let consul = services.and_then(|s| s.consul.as_ref());
    let vault = services.and_then(|s| s.vault.as_ref());
    let nomad = services.and_then(|s| s.nomad.as_ref());

    let consul_enabled = consul.map(|c| c.enabled).unwrap_or(false);
    let vault_enabled = vault.map(|v| v.enabled).unwrap_or(false);
    let nomad_enabled = nomad.map(|n| n.enabled).unwrap_or(false);

    let consul_servers: Vec<&ConsulServer> = consul.and_then(|c| c.servers.as_ref()).into_iter().flatten().collect();
    let nomad_servers: Vec<&NomadServer> = nomad.and_then(|n| n.servers.as_ref()).into_iter().flatten().collect();
    let nomad_clients: Vec<&NomadClient> = nomad.and_then(|n| n.clients.as_ref()).into_iter().flatten().collect();

    let mut names: HashMap<&str, &str> = HashMap::new();
    if vault_enabled {
        names.insert("vault", "services.vault");
    }
    names.insert("bootstrap", "the bootstrap container");
    let mut nodes: Vec<(&str, &str)> = Vec::new();
    if consul_enabled {
        nodes.extend(consul_servers.iter().map(|s| ("services.consul.servers", s.name.as_str())));
    }
    nodes.extend(nomad_servers.iter().map(|s| ("services.nomad.servers", s.name.as_str())));
    nodes.extend(nomad_clients.iter().map(|c| ("services.nomad.clients", c.name.as_str())));
    for (path, name) in nodes {
        if !is_valid_name(name, true) {
            errors.push(format!("{}: name '{}' must use letters, digits, '-' or '_'", path, name));
        }
        if let Some(other) = names.insert(name, path) {
            errors.push(format!(
                "{}: name '{}' is already used by {} (container '{}-{}')",
                path, name, other, config.datacenter, name
            ));
        }
    }

    // ---- consul
    if let Some(consul) = consul.filter(|c| c.enabled) {
        let config = consul.config.as_ref();
        if consul_servers.is_empty() {
            errors.push("services.consul: enabled, but no servers are defined".to_string());
        }
        if consul_servers.len() > MAX_CONSUL_SERVERS {
            errors.push(format!(
                "services.consul.servers: {} servers, the maximum is {}",
                consul_servers.len(),
                MAX_CONSUL_SERVERS
            ));
        }
        check_acl(config.and_then(|c| c.acl.as_ref()), "services.consul.config.acl", &mut errors);
        let expect = config.and_then(|c| c.server.as_ref()).and_then(|s| s.bootstrap_expect);
        check_bootstrap_expect(expect, consul_servers.len(), "services.consul.config.server", &mut errors);
    }

    // ---- vault
    if let Some(vault) = vault.filter(|v| v.enabled) {
        if let Some(mode) = &vault.mode {
            if !VAULT_MODES.contains(&mode.as_str()) {
                errors.push(format!(
                    "services.vault.mode: '{}' is unknown, use one of: {}",
                    mode,
                    VAULT_MODES.join(", ")
                ));
            }
        }
        if let Some(token) = vault.config.as_ref().and_then(|c| c.root_token.as_ref()) {
            if token.trim().is_empty() {
                errors.push("services.vault.config.root_token: must not be empty".to_string());
            }
        }
    }

    // ---- nomad
    if let Some(nomad) = nomad.filter(|n| n.enabled) {
        let config = nomad.config.as_ref();
        let client_retry_join = config
            .and_then(|c| c.client.as_ref())
            .and_then(|c| c.retry_join.as_ref())
            .map(|r| !r.is_empty())
            .unwrap_or(false);
        if nomad_servers.is_empty() && nomad_clients.is_empty() {
            errors.push("services.nomad: enabled, but no servers and no clients are defined".to_string());
        }
        if nomad_servers.is_empty() && !nomad_clients.is_empty() && !client_retry_join {
            errors.push(
                "services.nomad: clients without servers need services.nomad.config.client.retry_join (servers of another cluster)"
                    .to_string(),
            );
        }
        if nomad_servers.len() > MAX_NOMAD_SERVERS {
            errors.push(format!(
                "services.nomad.servers: {} servers, the maximum is {}",
                nomad_servers.len(),
                MAX_NOMAD_SERVERS
            ));
        }
        check_acl(config.and_then(|c| c.acl.as_ref()), "services.nomad.config.acl", &mut errors);
        let expect = config.and_then(|c| c.server.as_ref()).and_then(|s| s.bootstrap_expect);
        if !nomad_servers.is_empty() {
            check_bootstrap_expect(expect, nomad_servers.len(), "services.nomad.config.server", &mut errors);
        }

        // clients ip: .(100 + loop_index), loop_index defaults to the position in the list
        let mut ips: HashMap<u64, &str> = HashMap::new();
        for (position, client) in nomad_clients.iter().enumerate() {
            let index = client.loop_index.unwrap_or(position as u64 + 1);
            if index == 0 || index > MAX_CLIENT_INDEX {
                errors.push(format!(
                    "services.nomad.clients: '{}' has the index {}, it must be between 1 and {} (ip .{})",
                    client.name,
                    index,
                    MAX_CLIENT_INDEX,
                    100 + index
                ));
            } else if let Some(other) = ips.insert(index, &client.name) {
                errors.push(format!(
                    "services.nomad.clients: '{}' and '{}' get the same ip .{}, change a loop_index",
                    other,
                    client.name,
                    100 + index
                ));
            }
        }
    }

    // ---- ports published on the host
    let mut host_ports: HashMap<u16, &str> = HashMap::new();
    let mut published = Vec::new();
    if nomad_enabled && !nomad_servers.is_empty() {
        let port = nomad
            .and_then(|n| n.config.as_ref())
            .and_then(|c| c.server.as_ref())
            .and_then(|s| s.host_port.as_ref());
        published.push(("services.nomad.config.server.host_port", port, Some(4646)));
    }
    if consul_enabled && !consul_servers.is_empty() {
        let port = consul
            .and_then(|c| c.config.as_ref())
            .and_then(|c| c.server.as_ref())
            .and_then(|s| s.host_port.as_ref());
        published.push(("services.consul.config.server.host_port", port, Some(8500)));
    }
    if vault_enabled {
        let port = vault.and_then(|v| v.config.as_ref()).and_then(|c| c.host_port.as_ref());
        published.push(("services.vault.config.host_port", port, Some(8200)));
    }
    for (path, port, default) in published {
        if let Some(HostPort::Flag(true)) = port {
            errors.push(format!("{}: must be a port number, or false to not publish it", path));
            continue;
        }
        let port = match port {
            Some(port) => port.port(),
            None => default,
        };
        if let Some(port) = port {
            if let Some(other) = host_ports.insert(port, path) {
                errors.push(format!("{}: the port {} is already published by {}", path, port, other));
            }
        }
    }

    // ---- nomad nodes are started even when nomad is not enabled: a forgotten 'enabled: true'
    if !nomad_enabled && (!nomad_servers.is_empty() || !nomad_clients.is_empty()) {
        errors.push("services.nomad: nodes are defined but 'enabled: true' is missing".to_string());
    }

    if external_network && config.network.as_ref().and_then(|n| n.subnet.as_ref()).is_some() {
        errors.push(
            "network: 'subnet' and 'external' can not be used together (an external network is created by another cluster)"
                .to_string(),
        );
    }

    errors
}

fn check_acl(acl: Option<&Acl>, path: &str, errors: &mut Vec<String>) {
    if let Some(token) = acl.and_then(|a| a.token.as_ref()) {
        if !is_uuid(token) {
            errors.push(format!(
                "{}.token: '{}' must be a uuid, e.g. 11111111-1111-1111-1111-111111111111",
                path, token
            ));
        }
    }
}

fn check_bootstrap_expect(expect: Option<u64>, servers: usize, path: &str, errors: &mut Vec<String>) {
    if let Some(expect) = expect {
        if expect == 0 || expect as usize > servers {
            errors.push(format!(
                "{}.bootstrap_expect: {} but {} server(s) are defined, the cluster would never elect a leader",
                path, expect, servers
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errors(yaml: &str) -> Vec<String> {
        match validate(yaml, Path::new("examples")) {
            Ok(_) => Vec::new(),
            Err(errors) => errors,
        }
    }

    const MINIMUM: &str = "
name: 'test'
datacenter: 'localhost'
services:
  nomad:
    enabled: true
    servers:
      - name: 'server'
    clients:
      - name: 'worker'
";

    #[test]
    fn every_example_is_valid() {
        let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
        let mut checked = 0;
        let mut dirs = vec![examples];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() && !path.ends_with(".ministack") {
                    dirs.push(path);
                } else if path.file_name().unwrap() == "cluster.yaml" {
                    let content = std::fs::read_to_string(&path).unwrap();
                    let result = validate(&content, path.parent().unwrap());
                    assert!(result.is_ok(), "{}: {:?}", path.display(), result.err());
                    checked += 1;
                }
            }
        }
        assert!(checked >= 10, "only {} examples found", checked);
    }

    #[test]
    fn minimum_is_valid() {
        assert!(errors(MINIMUM).is_empty());
    }

    #[test]
    fn unknown_key_is_rejected() {
        let yaml = MINIMUM.replace("    enabled: true\n", "    enabled: true\n    config:\n      tsl: true\n");
        let errors = errors(&yaml);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("unknown field `tsl`"), "{}", errors[0]);
    }

    #[test]
    fn missing_name_is_rejected() {
        let errors = errors("datacenter: 'localhost'\n");
        assert!(errors[0].contains("missing field `name`"), "{}", errors[0]);
    }

    #[test]
    fn duplicated_node_names_are_rejected() {
        let yaml = MINIMUM.replace("'worker'", "'server'");
        assert!(errors(&yaml).iter().any(|e| e.contains("already used")));
    }

    #[test]
    fn bad_values_are_rejected() {
        let yaml = format!(
            "{}{}",
            MINIMUM.replace("    enabled: true\n", "    enabled: true\n    config:\n      acl:\n        enabled: true\n        token: 'secret'\n      server:\n        bootstrap_expect: 3\n"),
            "  vault:\n    enabled: true\n    mode: 'file'\n  consul:\n    enabled: true\nnetwork:\n  subnet: '10.1.300.0'\n"
        );
        let errors = errors(&yaml);
        for expected in ["must be a uuid", "bootstrap_expect", "mode: 'file'", "no servers", "network.subnet"] {
            assert!(errors.iter().any(|e| e.contains(expected)), "{} not in {:?}", expected, errors);
        }
    }

    #[test]
    fn same_host_port_is_rejected() {
        let yaml = format!("{}  vault:\n    enabled: true\n    config:\n      host_port: 4646\n", MINIMUM);
        assert!(errors(&yaml).iter().any(|e| e.contains("already published")));
    }

    #[test]
    fn disabled_host_port_is_valid() {
        let yaml = format!("{}  vault:\n    enabled: true\n    config:\n      host_port: false\n", MINIMUM);
        assert!(errors(&yaml).is_empty());
    }

    #[test]
    fn clients_need_servers_or_retry_join() {
        let yaml = MINIMUM.replace("    servers:\n      - name: 'server'\n", "");
        assert!(errors(&yaml).iter().any(|e| e.contains("retry_join")));
    }

    #[test]
    fn same_client_ip_is_rejected() {
        let yaml = MINIMUM.replace("      - name: 'worker'\n", "      - name: 'worker'\n      - name: 'other'\n        loop_index: 1\n");
        assert!(errors(&yaml).iter().any(|e| e.contains("same ip")));
    }

    #[test]
    fn missing_plugin_is_rejected() {
        let yaml = format!("{}plugins:\n  - 'plugins/nothing.yaml'\n", MINIMUM);
        assert!(errors(&yaml).iter().any(|e| e.contains("does not exist")));
    }
}
