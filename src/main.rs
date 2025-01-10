use clap::{Arg, Command};
use include_dir::{include_dir, Dir};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::process::Command as ShellCommand;
use tera::{Context, Tera};

// Embed the "templates" directory into the binary
static TEMPLATES_DIR: Dir = include_dir!("templates");

// Default output directory and file
const OUTPUT_FILE: &str = "/tmp/ministack/docker-compose.yml";

fn main() {
    // Define the CLI arguments
    let matches = Command::new("Docker Compose Templating System")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Generates a Docker Compose file from a predefined template and optionally starts or stops services")
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
                .help("Run 'docker-compose up -d' after generating the file")
                .takes_value(false),
        )
        .arg(
            Arg::new("stop")
                .long("stop")
                .help("Run 'docker-compose down' to stop and remove services")
                .takes_value(false),
        )
        .get_matches();

    // Verify that `docker-compose` is installed
    if !is_docker_compose_available() {
        eprintln!("Error: 'docker-compose' is not installed or not accessible in the PATH.");
        std::process::exit(1);
    }

    // Retrieve the arguments
    let config_path = matches.get_one::<String>("config").expect("Config path is required");
    let start_services = matches.contains_id("start");
    let stop_services = matches.contains_id("stop");

    // Load and parse the YAML configuration file
    let config_data = load_yaml(config_path);

    // Load the template content from the embedded directory
    let template_content = load_template("docker-compose.yml");

    // Render the template with the YAML data
    let output = render_template(&template_content, config_data);

    // Write the rendered output to the default output file
    write_to_file(OUTPUT_FILE, &output);

    println!("Rendered Docker Compose file has been saved to '{}'", OUTPUT_FILE);

    // Start Docker Compose services if requested
    if start_services {
        start_docker_compose(OUTPUT_FILE);
    }

    // Stop Docker Compose services if requested
    if stop_services {
        stop_docker_compose(OUTPUT_FILE);
    }
}

/// Check if `docker-compose` is available
fn is_docker_compose_available() -> bool {
    let status = ShellCommand::new("docker-compose")
        .arg("--version")
        .status();

    match status {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Load and parse the YAML file
fn load_yaml(file_path: &str) -> HashMap<String, Value> {
    let content = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Could not read the file at path: {}", file_path));
    serde_yaml::from_str(&content)
        .unwrap_or_else(|_| panic!("Failed to parse YAML file at path: {}", file_path))
}

/// Load a template from the embedded directory
fn load_template(template_name: &str) -> String {
    let file = TEMPLATES_DIR
        .get_file(template_name)
        .unwrap_or_else(|| panic!("Template '{}' not found in embedded templates.", template_name));

    file.contents_utf8()
        .expect("Template file is not valid UTF-8")
        .to_string()
}

/// Render the template using Tera
fn render_template(template_content: &str, data: HashMap<String, Value>) -> String {
    let mut tera = Tera::default();
    tera.add_raw_template("template", template_content)
        .expect("Failed to load the template");

    let mut context = Context::new();
    for (key, value) in data {
        context.insert(key, &value);
    }

    tera.render("template", &context)
        .unwrap_or_else(|e| panic!("Failed to render template: {}", e))
}

/// Write the rendered output to a file
fn write_to_file(output_path: &str, content: &str) {
    fs::create_dir_all("/tmp/ministack").unwrap_or_else(|_| panic!("Could not create directory: /tmp/ministack"));
    fs::write(output_path, content)
        .unwrap_or_else(|_| panic!("Could not write to file at path: {}", output_path));
}

/// Run `docker-compose up -d` on the generated file
fn start_docker_compose(output_path: &str) {
    println!("Starting Docker Compose services...");
    let status = ShellCommand::new("docker-compose")
        .arg("-f")
        .arg(output_path)
        .arg("up")
        .arg("-d")
        .status()
        .expect("Failed to execute 'docker-compose up -d'");

    if status.success() {
        println!("Docker Compose services started successfully.");
    } else {
        eprintln!("Failed to start Docker Compose services.");
    }
}

/// Run `docker-compose down` on the generated file
fn stop_docker_compose(output_path: &str) {
    println!("Stopping Docker Compose services...");
    let status = ShellCommand::new("docker-compose")
        .arg("-f")
        .arg(output_path)
        .arg("down")
        .status()
        .expect("Failed to execute 'docker-compose down'");

    if status.success() {
        println!("Docker Compose services stopped successfully.");
    } else {
        eprintln!("Failed to stop Docker Compose services.");
    }
}
