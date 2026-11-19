// ministack --status: the containers of the cluster, their state, and the addresses to reach it

use crate::config::Config;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::Command;

// a line of the output, written at once at the end (a closed pipe, e.g. '| head', is not an error)
macro_rules! outln {
    ($out:expr, $($arg:tt)*) => {
        let _ = writeln!($out, $($arg)*);
    };
}

// a container of the compose project, as docker sees it
struct Container {
    state: String,
    status: String,
}

// running, and healthy when it has a healthcheck
fn is_ready(container: &Container) -> bool {
    container.state == "running"
        && !container.status.contains("unhealthy")
        && !container.status.contains("health: starting")
}

// every container of the compose project (plugins included), by name
fn project_containers(project: &str) -> Result<Vec<(String, Container)>, String> {
    let output = Command::new("docker")
        .args(["ps", "--all", "--filter"])
        .arg(format!("label=com.docker.compose.project={}", project))
        .args(["--format", "{{.Names}}\t{{.State}}\t{{.Status}}"])
        .output()
        .map_err(|e| format!("could not run 'docker ps': {}", e))?;
    if !output.status.success() {
        return Err(format!(
            "'docker ps' failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let name = fields.next()?.to_string();
            let state = fields.next()?.to_string();
            let status = fields.next().unwrap_or_default().to_string();
            Some((name, Container { state, status }))
        })
        .collect())
}

// the addresses exported by the generated env.sh, e.g. NOMAD_ADDR
fn addresses(env_file: &Path) -> Vec<(&'static str, String)> {
    let content = fs::read_to_string(env_file).unwrap_or_default();
    let exported: HashMap<&str, &str> = content
        .lines()
        .filter_map(|line| line.strip_prefix("export "))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key, value.trim_matches('\'')))
        .collect();
    [
        ("nomad", "NOMAD_ADDR"),
        ("consul", "CONSUL_HTTP_ADDR"),
        ("vault", "VAULT_ADDR"),
    ]
    .into_iter()
    .filter_map(|(service, key)| exported.get(key).map(|value| (service, value.to_string())))
    .collect()
}

// Print the status of the cluster, true when every node is running and healthy
pub fn print_status(config: &Config, config_path: &str, output_dir: &Path) -> bool {
    let containers = match project_containers(&config.name) {
        Ok(containers) => containers,
        Err(error) => {
            eprintln!("Error: {}", error);
            return false;
        }
    };
    let mut by_name: HashMap<String, Container> = containers.into_iter().collect();

    // the nodes of the configuration, then the other containers of the project (plugins)
    let mut out = String::new();
    let mut rows: Vec<[String; 4]> = Vec::new();
    let mut ready = 0;
    let nodes = config.nodes();
    for node in &nodes {
        let state = match by_name.remove(&node.container) {
            Some(container) => {
                if is_ready(&container) {
                    ready += 1;
                }
                container.status
            }
            None => "not created".to_string(),
        };
        let ip = node.ip.clone().unwrap_or_else(|| "-".to_string());
        rows.push([node.container.clone(), node.role.to_string(), ip, state]);
    }
    let mut others: Vec<(String, Container)> = by_name.into_iter().collect();
    others.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, container) in others {
        rows.push([
            name,
            "plugin".to_string(),
            "-".to_string(),
            container.status,
        ]);
    }

    outln!(
        out,
        "cluster '{}', datacenter '{}': {}/{} nodes ready",
        config.name,
        config.datacenter,
        ready,
        nodes.len()
    );
    out.push('\n');
    let header = ["NODE", "ROLE", "IP", "STATUS"].map(String::from);
    let widths: Vec<usize> = (0..3)
        .map(|i| {
            rows.iter()
                .chain(std::iter::once(&header))
                .map(|row| row[i].len())
                .max()
                .unwrap_or(0)
        })
        .collect();
    for row in std::iter::once(&header).chain(rows.iter()) {
        outln!(
            out,
            "{:w0$}  {:w1$}  {:w2$}  {}",
            row[0],
            row[1],
            row[2],
            row[3],
            w0 = widths[0],
            w1 = widths[1],
            w2 = widths[2]
        );
    }

    let started = ready > 0 || rows.iter().any(|row| row[3] != "not created");
    let addresses = addresses(&output_dir.join("env.sh"));
    if started && !addresses.is_empty() {
        out.push('\n');
        for (service, address) in addresses {
            outln!(out, "{:7} {}", service, address);
        }
        out.push('\n');
        outln!(out, "eval \"$(ministack --config {} --env)\"", config_path);
    } else if !started {
        out.push('\n');
        outln!(
            out,
            "the cluster is not started: ministack --config {} --start",
            config_path
        );
    }
    let _ = std::io::stdout().write_all(out.as_bytes());
    ready == nodes.len()
}
