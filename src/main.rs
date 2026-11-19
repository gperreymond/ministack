use clap::{Arg, ArgGroup, Command};
use include_dir::{include_dir, Dir, File};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::Command as ShellCommand;
use tera::{Context, Tera};

mod config;
mod status;

// Embed the "templates" directory into the binary
static TEMPLATES_DIR: Dir = include_dir!("src/templates");

// tls certificates, inside the output directory, kept between two renderings
const TLS_DIR: &str = "tls";

fn main() {
    // Define CLI arguments
    let matches = Command::new("ministack")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Gilles Perreymond <gperreymond@gmail.com>")
        .about("Manage hashistack with only one binary.")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Path to the YAML configuration file")
                .required(true),
        )
        .arg(
            Arg::new("start")
                .long("start")
                .action(clap::ArgAction::SetTrue)
                .help("Run 'docker compose up -d --wait' and wait for the cluster to be healthy"),
        )
        .arg(
            Arg::new("restart")
                .long("restart")
                .action(clap::ArgAction::SetTrue)
                .help("Run 'docker compose up -d --wait --force-recreate'"),
        )
        .arg(
            Arg::new("stop")
                .long("stop")
                .action(clap::ArgAction::SetTrue)
                .help("Run 'docker compose down' (volumes are kept)"),
        )
        .arg(
            Arg::new("purge")
                .long("purge")
                .action(clap::ArgAction::SetTrue)
                .help("Run 'docker compose down --volumes' (all cluster data is deleted)"),
        )
        .arg(
            Arg::new("env")
                .long("env")
                .action(clap::ArgAction::SetTrue)
                .help("Print the environment variables to reach the cluster, usage: eval \"$(ministack -c <file> --env)\""),
        )
        .arg(
            Arg::new("status")
                .long("status")
                .action(clap::ArgAction::SetTrue)
                .help("Show the nodes of the cluster, their state and the addresses (exit code 1 when a node is not ready)"),
        )
        .group(
            ArgGroup::new("action")
                .args(["start", "restart", "stop", "purge", "env", "status"])
                .multiple(false),
        )
        .get_matches();

    // Retrieve the config path and validate its existence
    let config_path = matches
        .get_one::<String>("config")
        .expect("Config path is required");
    if !Path::new(config_path).exists() {
        eprintln!(
            "Error: Configuration file '{}' does not exist.",
            config_path
        );
        std::process::exit(1);
    }

    // Define the output directory dynamically based on the config file
    let output_dir = Path::new(config_path)
        .parent()
        .expect("Failed to determine parent directory of the config file")
        .join(".ministack");

    // Load and validate the YAML configuration file, then render all templates
    let (config, mut config_data) = load_config(config_path);
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|_| panic!("Could not create directory: {}", output_dir.display()));
    let ministack_dir =
        fs::canonicalize(&output_dir).expect("Failed to resolve the output directory");
    // absolute path of the output directory, for the templates (ca file paths in env.sh)
    config_data.insert(
        "ministack_dir".to_string(),
        Value::String(ministack_dir.to_string_lossy().to_string()),
    );
    let generated_files = render_all_templates(&config_data, &output_dir);
    // the read only commands keep their output clean
    if !(matches.get_flag("env") || matches.get_flag("status")) {
        for file in generated_files {
            eprintln!("Rendered: '{}'", file);
        }
    }

    // Delete the certificates with the data of the cluster
    if matches.get_flag("purge") {
        fs::remove_dir_all(output_dir.join(TLS_DIR)).ok();
    }

    // Create the tls certificates before starting the cluster (or for a manual docker compose)
    let starting = matches.get_flag("start") || matches.get_flag("restart");
    let only_rendering = !(starting
        || matches.get_flag("stop")
        || matches.get_flag("purge")
        || matches.get_flag("env")
        || matches.get_flag("status"));
    if starting || only_rendering {
        if !create_tls_certificates(&config_data, &ministack_dir) {
            std::process::exit(1);
        }
    }

    // Print the environment variables, no docker needed
    if matches.get_flag("env") {
        let env_file = output_dir.join("env.sh");
        print!("{}", fs::read_to_string(&env_file).unwrap_or_default());
        return;
    }

    // State of the containers, and the addresses to reach the cluster
    if matches.get_flag("status") {
        if !is_docker_available() {
            eprintln!("Error: 'docker' is not installed or not accessible in the PATH.");
            std::process::exit(1);
        }
        if !status::print_status(&config, config_path, &output_dir) {
            std::process::exit(1);
        }
        return;
    }

    let compose_args: Option<&[&str]> = if matches.get_flag("start") {
        Some(&["up", "-d", "--wait"])
    } else if matches.get_flag("restart") {
        Some(&["up", "-d", "--wait", "--force-recreate"])
    } else if matches.get_flag("stop") {
        Some(&["down"])
    } else if matches.get_flag("purge") {
        Some(&["down", "--volumes"])
    } else {
        None
    };

    if let Some(args) = compose_args {
        // Verify that Docker is installed
        if !is_docker_available() {
            eprintln!("Error: 'docker' is not installed or not accessible in the PATH.");
            std::process::exit(1);
        }
        let success = run_docker_compose(&output_dir, args);
        if !success {
            std::process::exit(1);
        }
        if args[0] == "up" {
            println!();
            println!("[INFO] cluster is ready, to use it from your shell:");
            println!("  eval \"$(ministack --config {} --env)\"", config_path);
        }
    }
}

// Check if Docker is available
fn is_docker_available() -> bool {
    let status = ShellCommand::new("docker")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

// Load, validate and parse the configuration file, exit with every problem found
fn load_config(file_path: &str) -> (config::Config, HashMap<String, Value>) {
    let content = fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!(
            "Error: could not read the configuration file '{}': {}",
            file_path, e
        );
        std::process::exit(1);
    });
    let config_dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
    let config = config::validate(&content, config_dir).unwrap_or_else(|errors| {
        eprintln!("Error: the configuration file '{}' is invalid:", file_path);
        for error in errors {
            eprintln!("  - {}", error);
        }
        std::process::exit(1);
    });
    // the templates use the raw values, and the values computed from them (tls, acl, tokens, network...)
    let mut data: HashMap<String, Value> = serde_yaml::from_str(&content).unwrap_or_else(|e| {
        eprintln!(
            "Error: the configuration file '{}' is invalid: {}",
            file_path, e
        );
        std::process::exit(1);
    });
    config.add_node_ips(&mut data);
    for (key, value) in config.computed() {
        data.insert(key.to_string(), value);
    }
    (config, data)
}

// Run the generated scripts/tls.sh (empty when tls is disabled) in a hashibase container,
// the certificates are written into <output_dir>/tls, owned by the current user
fn create_tls_certificates(config_data: &HashMap<String, Value>, ministack_dir: &Path) -> bool {
    let script = ministack_dir.join("scripts").join("tls.sh");
    let content = fs::read_to_string(&script).unwrap_or_default();
    if content.trim().is_empty() {
        return true;
    }
    if !is_docker_available() {
        eprintln!("Error: 'docker' is needed to create the tls certificates.");
        return false;
    }

    let tls_dir = ministack_dir.join(TLS_DIR);
    fs::create_dir_all(&tls_dir)
        .unwrap_or_else(|_| panic!("Could not create directory: {}", tls_dir.display()));

    // same image as the nodes
    let computed = |key: &str| {
        config_data
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };
    let repository = computed("hashibase_repository");
    let tag = computed("hashibase_tag");

    // run as the owner of the output directory, so the files stay removable by the user
    let metadata =
        fs::metadata(ministack_dir).expect("Failed to read the output directory metadata");
    let user = format!("{}:{}", metadata.uid(), metadata.gid());

    let status = ShellCommand::new("docker")
        .args(["run", "--rm", "--user", &user, "--entrypoint", "bash"])
        .arg("--volume")
        .arg(format!("{}:/tls", tls_dir.display()))
        .arg("--volume")
        .arg(format!("{}:/ministack/tls.sh:ro", script.display()))
        .arg(format!("{}:{}", repository, tag))
        .arg("/ministack/tls.sh")
        .status()
        .unwrap_or_else(|e| panic!("Failed to execute 'docker run': {}", e));

    if !status.success() {
        eprintln!("Error: the tls certificates could not be created.");
    }
    status.success()
}

// Render all templates in the embedded directory, including subdirectories
fn render_all_templates(config_data: &HashMap<String, Value>, output_dir: &Path) -> Vec<String> {
    let mut generated_files = Vec::new();

    // clean the previous rendering, but keep the tls certificates (and their ca)
    fs::create_dir_all(output_dir)
        .unwrap_or_else(|_| panic!("Could not create directory: {}", output_dir.display()));
    for entry in fs::read_dir(output_dir)
        .expect("Failed to read the output directory")
        .flatten()
    {
        if entry.file_name() == TLS_DIR {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).ok();
        } else {
            fs::remove_file(&path).ok();
        }
    }

    // Traverse and render files recursively
    traverse_and_render(
        &TEMPLATES_DIR,
        output_dir,
        config_data,
        &mut generated_files,
    );

    generated_files
}

// Recursively traverse the template directory and render templates
fn traverse_and_render(
    dir: &Dir,
    base_output_dir: &Path, // The base output directory for all templates
    config_data: &HashMap<String, Value>,
    generated_files: &mut Vec<String>,
) {
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(subdir) => {
                // Get the relative path of the subdirectory
                let relative_path = subdir.path().strip_prefix(TEMPLATES_DIR.path()).unwrap();
                // Build the full output directory path
                let sub_output_dir = base_output_dir.join(relative_path);

                // Ensure the output directory exists
                fs::create_dir_all(&sub_output_dir).unwrap_or_else(|_| {
                    panic!("Could not create directory: {}", sub_output_dir.display())
                });

                // Recursively process the subdirectory
                traverse_and_render(subdir, base_output_dir, config_data, generated_files);
            }
            include_dir::DirEntry::File(file) => {
                // Render and save the templates
                render_and_save_template(file, config_data, generated_files, base_output_dir);
            }
        }
    }
}

// Render a single template and save it to the output directory
fn render_and_save_template(
    file: &File,
    config_data: &HashMap<String, Value>,
    generated_files: &mut Vec<String>,
    base_output_dir: &Path,
) {
    let template_path = file.path();
    let template_content = file
        .contents_utf8()
        .expect("Template file is not valid UTF-8");

    // Compute the relative path from the root of TEMPLATES_DIR
    let relative_path = template_path
        .strip_prefix(TEMPLATES_DIR.path())
        .expect("Failed to strip TEMPLATES_DIR prefix");

    // Combine the base output directory with the relative path
    let output_path = base_output_dir.join(relative_path);

    // Render the template content
    let output = render_template(template_content, config_data);

    // Write the rendered content to the output file
    write_to_file(output_path.to_str().unwrap(), &output);

    // Add the generated file to the list
    generated_files.push(output_path.to_str().unwrap().to_string());
}

// Render a single template using Tera
fn render_template(template_content: &str, data: &HashMap<String, Value>) -> String {
    let mut context = Context::new();
    for (key, value) in data {
        context.insert(key, &value);
    }
    let mut tera = Tera::default();
    tera.add_raw_template("template", template_content)
        .and_then(|_| tera.render("template", &context))
        .unwrap_or_else(|e| {
            // the cause of a tera error is in its sources
            let mut message = e.to_string();
            let mut source = std::error::Error::source(&e);
            while let Some(cause) = source {
                message = format!("{}: {}", message, cause);
                source = cause.source();
            }
            eprintln!("Error: {}", message);
            std::process::exit(1);
        })
}

// Write the rendered output to a file
fn write_to_file(output_path: &str, content: &str) {
    let path = Path::new(output_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|_| panic!("Could not create directory: {}", parent.display()));
    }
    fs::write(output_path, content)
        .unwrap_or_else(|_| panic!("Could not write to file at path: {}", output_path));
}

// Run 'docker compose --file <output_dir>/cluster.yaml <args>', output is streamed to the terminal
fn run_docker_compose(output_dir: &Path, args: &[&str]) -> bool {
    let compose_file = output_dir.join("cluster.yaml");
    if !compose_file.exists() {
        eprintln!(
            "Error: Docker Compose file '{}' does not exist.",
            compose_file.display()
        );
        std::process::exit(1);
    }

    let status = ShellCommand::new("docker")
        .arg("compose")
        .arg("--file")
        .arg(&compose_file)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to execute 'docker compose {}': {}",
                args.join(" "),
                e
            )
        });

    if !status.success() {
        eprintln!(
            "Command 'docker compose {}' failed with status: {}",
            args.join(" "),
            status
        );
    }
    status.success()
}
