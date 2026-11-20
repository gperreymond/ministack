// Schema and validation of the cluster configuration file.
//
// The templates still receive the raw yaml values: this module only checks the file
// before rendering, so a typo (an unknown key) or an inconsistent cluster is reported
// with a clear message instead of a silently ignored setting or a cluster that never starts.

// the fields describe the schema, most of them are only read by the templates
#![allow(dead_code)]

use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::path::Path;

// defaults, kept in one place
const DEFAULT_HASHIBASE_REPOSITORY: &str = "ghcr.io/gperreymond/hashibase";
const DEFAULT_HASHIBASE_TAG: &str = "base-1.3.0";
const DEFAULT_SUBNET: &str = "10.1.0.0";
const DEFAULT_NOMAD_TOKEN: &str = "11111111-1111-1111-1111-111111111111";
const DEFAULT_CONSUL_TOKEN: &str = "22222222-2222-2222-2222-222222222222";
const DEFAULT_VAULT_ROOT_TOKEN: &str = "root";
const DEFAULT_VAULT_VERSION: &str = "2.2.2";
const DEFAULT_VAULT_MODE: &str = "dev";
const DEFAULT_INGRESS_PORT: u16 = 8080;
const DEFAULT_INGRESS_DOMAIN: &str = "localhost";
const DEFAULT_INGRESS_VERSION: &str = "v3.7.19";

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
    pub ingress: Option<Ingress>,
    #[serde(default)]
    pub services: Option<Services>,
}

// a traefik in front of the services of the jobs: http://<service>.<domain>:<port>
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ingress {
    #[serde(default)]
    pub enabled: bool,
    pub port: Option<u16>,
    pub domain: Option<String>,
    pub version: Option<String>,
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
    // images of the host copied into every client after each start
    pub copy_images: Option<Vec<String>>,
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

// ------------------------------------------------------------------ computed values

// a non empty string, as a tera condition would see it
fn non_empty(value: Option<&String>) -> Option<&str> {
    value.map(|v| v.as_str()).filter(|v| !v.is_empty())
}

// the host port as the templates use it: the configured value (a number or false), or the default
fn host_port(port: Option<&HostPort>, default: u16) -> Value {
    match port {
        Some(HostPort::Port(port)) => Value::from(*port),
        Some(HostPort::Flag(flag)) => Value::from(*flag),
        None => Value::from(default),
    }
}

// a container of the cluster
#[derive(Debug, PartialEq)]
pub struct Node {
    pub container: String,
    pub role: &'static str,
    pub ip: Option<String>,
}

impl Node {
    fn new(container: String, role: &'static str, ip: Option<String>) -> Node {
        Node {
            container,
            role,
            ip,
        }
    }
}

// ip of a node: nomad servers .11+, consul servers .21+, vault .30, nomad clients .(100 + loop_index)
const NOMAD_SERVER_BASE: u64 = 10;
const CONSUL_SERVER_BASE: u64 = 20;
const VAULT_NUMBER: u64 = 30;
const INGRESS_NUMBER: u64 = 40;
const NOMAD_CLIENT_BASE: u64 = 100;

impl Config {
    fn network_prefix(&self) -> String {
        let subnet = non_empty(self.network.as_ref().and_then(|n| n.subnet.as_ref()))
            .unwrap_or(DEFAULT_SUBNET);
        subnet.split('.').take(3).collect::<Vec<_>>().join(".")
    }

    // ips of the nodes, in the order of the configuration
    fn nomad_server_ips(&self) -> Vec<String> {
        let prefix = self.network_prefix();
        let servers = self
            .services
            .as_ref()
            .and_then(|s| s.nomad.as_ref())
            .and_then(|n| n.servers.as_ref());
        (1..=servers.map(|s| s.len()).unwrap_or(0) as u64)
            .map(|index| format!("{}.{}", prefix, NOMAD_SERVER_BASE + index))
            .collect()
    }

    fn consul_server_ips(&self) -> Vec<String> {
        let prefix = self.network_prefix();
        let servers = self
            .services
            .as_ref()
            .and_then(|s| s.consul.as_ref())
            .and_then(|c| c.servers.as_ref());
        (1..=servers.map(|s| s.len()).unwrap_or(0) as u64)
            .map(|index| format!("{}.{}", prefix, CONSUL_SERVER_BASE + index))
            .collect()
    }

    fn nomad_client_ips(&self) -> Vec<String> {
        let prefix = self.network_prefix();
        let clients = self
            .services
            .as_ref()
            .and_then(|s| s.nomad.as_ref())
            .and_then(|n| n.clients.as_ref());
        clients
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(position, client)| {
                let index = client.loop_index.unwrap_or(position as u64 + 1);
                format!("{}.{}", prefix, NOMAD_CLIENT_BASE + index)
            })
            .collect()
    }

    // Every container of the cluster, in the start order, for the status command
    pub fn nodes(&self) -> Vec<Node> {
        let computed: HashMap<&str, Value> = self.computed().into_iter().collect();
        let flag = |key: &str| computed.get(key).and_then(|v| v.as_bool()).unwrap_or(false);
        let services = self.services.as_ref();
        let consul = services.and_then(|s| s.consul.as_ref());
        let vault = services.and_then(|s| s.vault.as_ref());
        let nomad = services.and_then(|s| s.nomad.as_ref());
        let container = |name: &str| format!("{}-{}", self.datacenter, name);
        let mut nodes = Vec::new();

        if consul.map(|c| c.enabled).unwrap_or(false) {
            let servers = consul
                .and_then(|c| c.servers.as_ref())
                .into_iter()
                .flatten();
            for (server, ip) in servers.zip(self.consul_server_ips()) {
                nodes.push(Node::new(
                    container(&server.name),
                    "consul server",
                    Some(ip),
                ));
            }
        }
        if vault.map(|v| v.enabled).unwrap_or(false) {
            let role = if flag("vault_raft") {
                "vault (raft)"
            } else {
                "vault (dev)"
            };
            let ip = format!("{}.{}", self.network_prefix(), VAULT_NUMBER);
            nodes.push(Node::new(container("vault"), role, Some(ip)));
        }
        // the nomad nodes are started when they are defined
        let servers = nomad.and_then(|n| n.servers.as_ref()).into_iter().flatten();
        for (server, ip) in servers.zip(self.nomad_server_ips()) {
            nodes.push(Node::new(container(&server.name), "nomad server", Some(ip)));
        }
        let clients = nomad.and_then(|n| n.clients.as_ref()).into_iter().flatten();
        for (client, ip) in clients.zip(self.nomad_client_ips()) {
            nodes.push(Node::new(container(&client.name), "nomad client", Some(ip)));
        }
        if flag("needs_bootstrap") {
            nodes.push(Node::new(container("bootstrap"), "bootstrap", None));
        }
        if self.ingress_enabled() {
            let ip = format!("{}.{}", self.network_prefix(), INGRESS_NUMBER);
            nodes.push(Node::new(container("ingress"), "ingress", Some(ip)));
        }
        nodes
    }

    pub fn ingress_enabled(&self) -> bool {
        self.ingress.as_ref().map(|i| i.enabled).unwrap_or(false)
    }

    // http://<service>.<domain>:<port>, the address of a service behind the ingress
    pub fn ingress_url(&self, service: &str) -> Option<String> {
        let ingress = self.ingress.as_ref().filter(|i| i.enabled)?;
        let domain = non_empty(ingress.domain.as_ref()).unwrap_or(DEFAULT_INGRESS_DOMAIN);
        let port = ingress.port.unwrap_or(DEFAULT_INGRESS_PORT);
        Some(format!("http://{}.{}:{}", service, domain, port))
    }

    // Images of the host to copy into the clients after each start
    pub fn copy_images(&self) -> Vec<String> {
        self.services
            .as_ref()
            .and_then(|s| s.nomad.as_ref())
            .and_then(|n| n.config.as_ref())
            .and_then(|c| c.client.as_ref())
            .and_then(|c| c.copy_images.clone())
            .unwrap_or_default()
    }

    // Containers of the nomad clients
    pub fn client_containers(&self) -> Vec<String> {
        self.nodes()
            .into_iter()
            .filter(|node| node.role == "nomad client")
            .map(|node| node.container)
            .collect()
    }

    // Add an 'ip' to every node of the raw configuration, for the loops of the templates
    pub fn add_node_ips(&self, data: &mut HashMap<String, Value>) {
        let nodes = [
            ("nomad", "servers", self.nomad_server_ips()),
            ("nomad", "clients", self.nomad_client_ips()),
            ("consul", "servers", self.consul_server_ips()),
        ];
        for (service, kind, ips) in nodes {
            let list = data
                .get_mut("services")
                .and_then(|s| s.get_mut(service))
                .and_then(|s| s.get_mut(kind))
                .and_then(|l| l.as_sequence_mut());
            for (node, ip) in list.into_iter().flatten().zip(ips) {
                if let Some(node) = node.as_mapping_mut() {
                    node.insert(Value::from("ip"), Value::from(ip));
                }
            }
        }
    }

    // Values derived from the configuration, given to every template as global variables
    pub fn computed(&self) -> Vec<(&'static str, Value)> {
        let services = self.services.as_ref();
        let consul = services.and_then(|s| s.consul.as_ref());
        let vault = services.and_then(|s| s.vault.as_ref());
        let nomad = services.and_then(|s| s.nomad.as_ref());
        let consul_config = consul.and_then(|c| c.config.as_ref());
        let vault_config = vault.and_then(|v| v.config.as_ref());
        let nomad_config = nomad.and_then(|n| n.config.as_ref());

        let consul_enabled = consul.map(|c| c.enabled).unwrap_or(false);
        let vault_enabled = vault.map(|v| v.enabled).unwrap_or(false);
        let nomad_enabled = nomad.map(|n| n.enabled).unwrap_or(false);

        // tls of a disabled service is ignored
        let nomad_tls = nomad_enabled && nomad_config.map(|c| c.tls).unwrap_or(false);
        let consul_tls = consul_enabled && consul_config.map(|c| c.tls).unwrap_or(false);
        let vault_tls = vault_enabled && vault_config.map(|c| c.tls).unwrap_or(false);
        let scheme = |tls: bool| if tls { "https" } else { "http" };

        let nomad_acl_config = nomad_config.and_then(|c| c.acl.as_ref());
        let consul_acl_config = consul_config.and_then(|c| c.acl.as_ref());
        let nomad_acl = nomad_acl_config.map(|a| a.enabled).unwrap_or(false);
        let consul_acl = consul_enabled && consul_acl_config.map(|a| a.enabled).unwrap_or(false);

        let vault_mode =
            non_empty(vault.and_then(|v| v.mode.as_ref())).unwrap_or(DEFAULT_VAULT_MODE);
        let vault_raft = vault_enabled && vault_mode == "raft";
        let with_nomad = nomad_enabled
            && nomad
                .and_then(|n| n.servers.as_ref())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
        let needs_bootstrap =
            vault_raft || (with_nomad && (nomad_acl || consul_acl || vault_enabled));

        let network_subnet = non_empty(self.network.as_ref().and_then(|n| n.subnet.as_ref()))
            .unwrap_or(DEFAULT_SUBNET);
        let network_prefix = self.network_prefix();
        // first node of each kind: the address used by the bootstrap and env.sh
        let first = |ips: Vec<String>| Value::from(ips.into_iter().next().unwrap_or_default());

        let hashibase = self.hashibase.as_ref();
        vec![
            ("nomad_tls", Value::from(nomad_tls)),
            ("consul_tls", Value::from(consul_tls)),
            ("vault_tls", Value::from(vault_tls)),
            ("any_tls", Value::from(nomad_tls || consul_tls || vault_tls)),
            ("nomad_scheme", Value::from(scheme(nomad_tls))),
            ("consul_scheme", Value::from(scheme(consul_tls))),
            ("vault_scheme", Value::from(scheme(vault_tls))),
            // consul http api port inside the cluster
            (
                "consul_port",
                Value::from(if consul_tls { 8501 } else { 8500 }),
            ),
            ("nomad_acl", Value::from(nomad_acl)),
            ("consul_acl", Value::from(consul_acl)),
            (
                "nomad_token",
                Value::from(
                    non_empty(nomad_acl_config.and_then(|a| a.token.as_ref()))
                        .unwrap_or(DEFAULT_NOMAD_TOKEN),
                ),
            ),
            (
                "consul_token",
                Value::from(
                    non_empty(consul_acl_config.and_then(|a| a.token.as_ref()))
                        .unwrap_or(DEFAULT_CONSUL_TOKEN),
                ),
            ),
            ("vault_mode", Value::from(vault_mode)),
            ("vault_raft", Value::from(vault_raft)),
            (
                "vault_root_token",
                Value::from(
                    non_empty(vault_config.and_then(|c| c.root_token.as_ref()))
                        .unwrap_or(DEFAULT_VAULT_ROOT_TOKEN),
                ),
            ),
            (
                "vault_version",
                Value::from(
                    non_empty(vault.and_then(|v| v.version.as_ref()))
                        .unwrap_or(DEFAULT_VAULT_VERSION),
                ),
            ),
            ("with_nomad", Value::from(with_nomad)),
            ("ingress_enabled", Value::from(self.ingress_enabled())),
            (
                "ingress_port",
                Value::from(
                    self.ingress
                        .as_ref()
                        .and_then(|i| i.port)
                        .unwrap_or(DEFAULT_INGRESS_PORT),
                ),
            ),
            (
                "ingress_domain",
                Value::from(
                    non_empty(self.ingress.as_ref().and_then(|i| i.domain.as_ref()))
                        .unwrap_or(DEFAULT_INGRESS_DOMAIN),
                ),
            ),
            (
                "ingress_version",
                Value::from(
                    non_empty(self.ingress.as_ref().and_then(|i| i.version.as_ref()))
                        .unwrap_or(DEFAULT_INGRESS_VERSION),
                ),
            ),
            (
                "ingress_ip",
                Value::from(format!("{}.{}", self.network_prefix(), INGRESS_NUMBER)),
            ),
            ("needs_bootstrap", Value::from(needs_bootstrap)),
            ("network_subnet", Value::from(network_subnet)),
            (
                "vault_ip",
                Value::from(format!("{}.{}", network_prefix, VAULT_NUMBER)),
            ),
            ("nomad_server_ip", first(self.nomad_server_ips())),
            ("nomad_client_ip", first(self.nomad_client_ips())),
            ("consul_server_ip", first(self.consul_server_ips())),
            (
                "hashibase_repository",
                Value::from(
                    non_empty(hashibase.and_then(|h| h.repository.as_ref()))
                        .unwrap_or(DEFAULT_HASHIBASE_REPOSITORY),
                ),
            ),
            (
                "hashibase_tag",
                Value::from(
                    non_empty(hashibase.and_then(|h| h.tag.as_ref()))
                        .unwrap_or(DEFAULT_HASHIBASE_TAG),
                ),
            ),
            // published on the host: a port, or false
            (
                "nomad_host_port",
                host_port(
                    nomad_config
                        .and_then(|c| c.server.as_ref())
                        .and_then(|s| s.host_port.as_ref()),
                    4646,
                ),
            ),
            (
                "consul_host_port",
                host_port(
                    consul_config
                        .and_then(|c| c.server.as_ref())
                        .and_then(|s| s.host_port.as_ref()),
                    8500,
                ),
            ),
            (
                "vault_host_port",
                host_port(vault_config.and_then(|c| c.host_port.as_ref()), 8200),
            ),
        ]
    }
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
        && groups.iter().zip(lengths).all(|(group, length)| {
            group.len() == length && group.chars().all(|c| c.is_ascii_hexdigit())
        })
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

    let consul_servers: Vec<&ConsulServer> = consul
        .and_then(|c| c.servers.as_ref())
        .into_iter()
        .flatten()
        .collect();
    let nomad_servers: Vec<&NomadServer> = nomad
        .and_then(|n| n.servers.as_ref())
        .into_iter()
        .flatten()
        .collect();
    let nomad_clients: Vec<&NomadClient> = nomad
        .and_then(|n| n.clients.as_ref())
        .into_iter()
        .flatten()
        .collect();

    let mut names: HashMap<&str, &str> = HashMap::new();
    if vault_enabled {
        names.insert("vault", "services.vault");
    }
    names.insert("bootstrap", "the bootstrap container");
    if config.ingress_enabled() {
        names.insert("ingress", "the ingress container");
    }
    let mut nodes: Vec<(&str, &str)> = Vec::new();
    if consul_enabled {
        nodes.extend(
            consul_servers
                .iter()
                .map(|s| ("services.consul.servers", s.name.as_str())),
        );
    }
    nodes.extend(
        nomad_servers
            .iter()
            .map(|s| ("services.nomad.servers", s.name.as_str())),
    );
    nodes.extend(
        nomad_clients
            .iter()
            .map(|c| ("services.nomad.clients", c.name.as_str())),
    );
    for (path, name) in nodes {
        if !is_valid_name(name, true) {
            errors.push(format!(
                "{}: name '{}' must use letters, digits, '-' or '_'",
                path, name
            ));
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
        check_acl(
            config.and_then(|c| c.acl.as_ref()),
            "services.consul.config.acl",
            &mut errors,
        );
        let expect = config
            .and_then(|c| c.server.as_ref())
            .and_then(|s| s.bootstrap_expect);
        check_bootstrap_expect(
            expect,
            consul_servers.len(),
            "services.consul.config.server",
            &mut errors,
        );
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
            errors.push(
                "services.nomad: enabled, but no servers and no clients are defined".to_string(),
            );
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
        let copy_images = config
            .and_then(|c| c.client.as_ref())
            .and_then(|c| c.copy_images.as_ref());
        for image in copy_images.into_iter().flatten() {
            if image.trim().is_empty() || image.contains(char::is_whitespace) {
                errors.push(format!(
                    "services.nomad.config.client.copy_images: '{}' is not an image name",
                    image
                ));
            }
        }
        if copy_images.map(|i| !i.is_empty()).unwrap_or(false) && nomad_clients.is_empty() {
            errors.push(
                "services.nomad.config.client.copy_images: there are no clients to copy the images to"
                    .to_string(),
            );
        }
        check_acl(
            config.and_then(|c| c.acl.as_ref()),
            "services.nomad.config.acl",
            &mut errors,
        );
        let expect = config
            .and_then(|c| c.server.as_ref())
            .and_then(|s| s.bootstrap_expect);
        if !nomad_servers.is_empty() {
            check_bootstrap_expect(
                expect,
                nomad_servers.len(),
                "services.nomad.config.server",
                &mut errors,
            );
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

    // ---- ingress: a traefik reading the services of the nomad servers of this cluster
    let ingress_port = config
        .ingress
        .as_ref()
        .map(|i| HostPort::Port(i.port.unwrap_or(DEFAULT_INGRESS_PORT)));
    if let Some(ingress) = config.ingress.as_ref().filter(|i| i.enabled) {
        if !nomad_enabled || nomad_servers.is_empty() {
            errors.push(
                "ingress: needs nomad servers in this cluster (it reads the services from their api)"
                    .to_string(),
            );
        }
        if ingress.port == Some(0) {
            errors.push("ingress.port: must be a port number".to_string());
        }
        if let Some(domain) = &ingress.domain {
            let valid =
                !domain.is_empty() && domain.split('.').all(|label| is_valid_name(label, false));
            if !valid {
                errors.push(format!(
                    "ingress.domain: '{}' is not a domain name, e.g. 'localhost'",
                    domain
                ));
            }
        }
    }

    // ---- ports published on the host
    let mut host_ports: HashMap<u16, &str> = HashMap::new();
    let mut published = Vec::new();
    if config.ingress_enabled() {
        published.push(("ingress.port", ingress_port.as_ref(), None));
    }
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
        let port = vault
            .and_then(|v| v.config.as_ref())
            .and_then(|c| c.host_port.as_ref());
        published.push(("services.vault.config.host_port", port, Some(8200)));
    }
    for (path, port, default) in published {
        if let Some(HostPort::Flag(true)) = port {
            errors.push(format!(
                "{}: must be a port number, or false to not publish it",
                path
            ));
            continue;
        }
        let port = match port {
            Some(port) => port.port(),
            None => default,
        };
        if let Some(port) = port {
            if let Some(other) = host_ports.insert(port, path) {
                errors.push(format!(
                    "{}: the port {} is already published by {}",
                    path, port, other
                ));
            }
        }
    }

    // ---- nomad nodes are started even when nomad is not enabled: a forgotten 'enabled: true'
    if !nomad_enabled && (!nomad_servers.is_empty() || !nomad_clients.is_empty()) {
        errors.push("services.nomad: nodes are defined but 'enabled: true' is missing".to_string());
    }

    if external_network
        && config
            .network
            .as_ref()
            .and_then(|n| n.subnet.as_ref())
            .is_some()
    {
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

fn check_bootstrap_expect(
    expect: Option<u64>,
    servers: usize,
    path: &str,
    errors: &mut Vec<String>,
) {
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
        let yaml = MINIMUM.replace(
            "    enabled: true\n",
            "    enabled: true\n    config:\n      tsl: true\n",
        );
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
        for expected in [
            "must be a uuid",
            "bootstrap_expect",
            "mode: 'file'",
            "no servers",
            "network.subnet",
        ] {
            assert!(
                errors.iter().any(|e| e.contains(expected)),
                "{} not in {:?}",
                expected,
                errors
            );
        }
    }

    #[test]
    fn same_host_port_is_rejected() {
        let yaml = format!(
            "{}  vault:\n    enabled: true\n    config:\n      host_port: 4646\n",
            MINIMUM
        );
        assert!(errors(&yaml)
            .iter()
            .any(|e| e.contains("already published")));
    }

    #[test]
    fn disabled_host_port_is_valid() {
        let yaml = format!(
            "{}  vault:\n    enabled: true\n    config:\n      host_port: false\n",
            MINIMUM
        );
        assert!(errors(&yaml).is_empty());
    }

    #[test]
    fn clients_need_servers_or_retry_join() {
        let yaml = MINIMUM.replace("    servers:\n      - name: 'server'\n", "");
        assert!(errors(&yaml).iter().any(|e| e.contains("retry_join")));
    }

    #[test]
    fn same_client_ip_is_rejected() {
        let yaml = MINIMUM.replace(
            "      - name: 'worker'\n",
            "      - name: 'worker'\n      - name: 'other'\n        loop_index: 1\n",
        );
        assert!(errors(&yaml).iter().any(|e| e.contains("same ip")));
    }

    fn computed(yaml: &str) -> HashMap<&'static str, Value> {
        validate(yaml, Path::new("examples"))
            .unwrap()
            .computed()
            .into_iter()
            .collect()
    }

    #[test]
    fn computed_defaults() {
        let values = computed(MINIMUM);
        assert_eq!(values["nomad_scheme"], Value::from("http"));
        assert_eq!(values["nomad_acl"], Value::from(false));
        assert_eq!(values["nomad_token"], Value::from(DEFAULT_NOMAD_TOKEN));
        assert_eq!(values["nomad_server_ip"], Value::from("10.1.0.11"));
        assert_eq!(values["nomad_client_ip"], Value::from("10.1.0.101"));
        assert_eq!(values["vault_ip"], Value::from("10.1.0.30"));
        assert_eq!(values["nomad_host_port"], Value::from(4646));
        assert_eq!(values["hashibase_tag"], Value::from(DEFAULT_HASHIBASE_TAG));
        assert_eq!(values["needs_bootstrap"], Value::from(false));
    }

    #[test]
    fn computed_secure_cluster() {
        let yaml = format!(
            "{}  consul:\n    config:\n      tls: true\nnetwork:\n  subnet: '10.1.20.0'\n",
            MINIMUM.replace(
                "    enabled: true\n",
                "    enabled: true\n    config:\n      tls: true\n      acl:\n        enabled: true\n      server:\n        host_port: false\n",
            )
        );
        let values = computed(&yaml);
        assert_eq!(values["nomad_scheme"], Value::from("https"));
        assert_eq!(values["nomad_server_ip"], Value::from("10.1.20.11"));
        assert_eq!(values["nomad_host_port"], Value::from(false));
        assert_eq!(values["needs_bootstrap"], Value::from(true));
        // the tls of a disabled service is ignored
        assert_eq!(values["consul_tls"], Value::from(false));
        assert_eq!(values["consul_port"], Value::from(8500));
    }

    #[test]
    fn node_ips_are_added() {
        let yaml = MINIMUM.replace(
            "      - name: 'worker'\n",
            "      - name: 'worker'\n      - name: 'other'\n        loop_index: 10\n",
        );
        let config = validate(&yaml, Path::new("examples")).unwrap();
        let mut data: HashMap<String, Value> = serde_yaml::from_str(&yaml).unwrap();
        config.add_node_ips(&mut data);
        let nomad = &data["services"]["nomad"];
        assert_eq!(nomad["servers"][0]["ip"], Value::from("10.1.0.11"));
        assert_eq!(nomad["clients"][0]["ip"], Value::from("10.1.0.101"));
        assert_eq!(nomad["clients"][1]["ip"], Value::from("10.1.0.110"));
    }

    #[test]
    fn nodes_of_a_hashistack() {
        let yaml = format!(
            "{}  consul:\n    enabled: true\n    servers:\n      - name: 'consul'\n  vault:\n    enabled: true\n    mode: 'raft'\n",
            MINIMUM
        );
        let nodes = validate(&yaml, Path::new("examples")).unwrap().nodes();
        let summary: Vec<(&str, &str, Option<&str>)> = nodes
            .iter()
            .map(|n| (n.container.as_str(), n.role, n.ip.as_deref()))
            .collect();
        assert_eq!(
            summary,
            vec![
                ("localhost-consul", "consul server", Some("10.1.0.21")),
                ("localhost-vault", "vault (raft)", Some("10.1.0.30")),
                ("localhost-server", "nomad server", Some("10.1.0.11")),
                ("localhost-worker", "nomad client", Some("10.1.0.101")),
                ("localhost-bootstrap", "bootstrap", None),
            ]
        );
    }

    #[test]
    fn copy_images_are_read_and_checked() {
        let yaml = MINIMUM.replace(
            "    enabled: true\n",
            "    enabled: true\n    config:\n      client:\n        copy_images:\n          - 'myapp:dev'\n",
        );
        let config = validate(&yaml, Path::new("examples")).unwrap();
        assert_eq!(config.copy_images(), vec!["myapp:dev".to_string()]);
        assert_eq!(
            config.client_containers(),
            vec!["localhost-worker".to_string()]
        );

        let bad = yaml.replace("'myapp:dev'", "'my app'");
        assert!(errors(&bad).iter().any(|e| e.contains("not an image name")));
    }

    #[test]
    fn ingress_is_computed_and_checked() {
        let yaml = format!("{}ingress:\n  enabled: true\n  port: 4646\n", MINIMUM);
        assert!(errors(&yaml)
            .iter()
            .any(|e| e.contains("already published")));

        let yaml = format!("{}ingress:\n  enabled: true\n", MINIMUM);
        let config = validate(&yaml, Path::new("examples")).unwrap();
        assert_eq!(
            config.ingress_url("whoami").as_deref(),
            Some("http://whoami.localhost:8080")
        );
        let last = config.nodes().pop().unwrap();
        assert_eq!(
            (last.container.as_str(), last.ip.as_deref()),
            ("localhost-ingress", Some("10.1.0.40"))
        );

        let bad = format!(
            "{}ingress:\n  enabled: true\n  domain: 'my domain'\n",
            MINIMUM
        );
        assert!(errors(&bad).iter().any(|e| e.contains("not a domain name")));
        let without_servers = yaml.replace("    servers:\n      - name: 'server'\n", "");
        assert!(errors(&without_servers)
            .iter()
            .any(|e| e.contains("needs nomad servers")));
    }

    #[test]
    fn missing_plugin_is_rejected() {
        let yaml = format!("{}plugins:\n  - 'plugins/nothing.yaml'\n", MINIMUM);
        assert!(errors(&yaml).iter().any(|e| e.contains("does not exist")));
    }
}
