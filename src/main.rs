use clap::{Arg, Command};
use include_dir::{include_dir, Dir, File};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command as ShellCommand, Stdio};
use tera::{Context, Tera};

// Embed the "templates" directory into the binary
static TEMPLATES_DIR: Dir = include_dir!("src/templates");

fn main() {
    // Define CLI arguments
    let matches = Command::new("Manage hashistack with only one binary.")
        .version("1.2.2")
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
                .help("Run 'docker compose up -d' for all generated files"),
        )
        .arg(
            Arg::new("restart")
                .long("restart")
                .action(clap::ArgAction::SetTrue)
                .help("Run 'docker compose up -d --force-recreate' for all generated files"),
        )
        .arg(
            Arg::new("stop")
                .long("stop")
                .action(clap::ArgAction::SetTrue)
                .help("Run 'docker compose down' for all generated files"),
        )
        .get_matches();

    // Verify that Docker is installed
    if !is_docker_available() {
        eprintln!("Error: 'docker' is not installed or not accessible in the PATH.");
        std::process::exit(1);
    }

    // Retrieve the config path and validate its existence
    let config_path = matches.get_one::<String>("config").expect("Config path is required");
    if !Path::new(config_path).exists() {
        eprintln!("Error: Configuration file '{}' does not exist.", config_path);
        std::process::exit(1);
    }

    // Define the output directory dynamically based on the config file
    let output_dir = Path::new(config_path)
        .parent()
        .expect("Failed to determine parent directory of the config file")
        .join(".ministack");

    // Ensure the output directory exists
    fs::create_dir_all(&output_dir).unwrap_or_else(|_| {
        panic!(
            "Could not create output directory: {}",
            output_dir.display()
        )
    });

    let start_services = matches.get_flag("start");
    let restart_services = matches.get_flag("restart");
    let stop_services = matches.get_flag("stop");

    // Restart Docker Compose services if requested
    if restart_services {
        start_docker_compose(true, &output_dir);
    } else {
        // Load and parse the YAML configuration file
        let config_data = load_yaml(config_path);
        // Render all templates
        let _ = render_all_templates(&config_data, &output_dir);
        // Start Docker Compose services if requested
        if start_services {
            start_docker_compose(false, &output_dir);
        }
        // Stop Docker Compose services if requested
        if stop_services {
            stop_docker_compose(&output_dir);
        }
    }
}

// Check if Docker is available
fn is_docker_available() -> bool {
    let status = ShellCommand::new("docker")
        .arg("--version")
        .status();

    match status {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

// Load and parse the YAML file
fn load_yaml(file_path: &str) -> HashMap<String, Value> {
    let content = fs::read_to_string(file_path)
        .unwrap_or_else(|_| panic!("Could not read the file at path: {}", file_path));
    serde_yaml::from_str(&content)
        .unwrap_or_else(|_| panic!("Failed to parse YAML file at path: {}", file_path))
}

// Render all templates in the embedded directory, including subdirectories
fn render_all_templates(config_data: &HashMap<String, Value>, output_dir: &Path) -> Vec<String> {
    let mut generated_files = Vec::new();

    fs::remove_dir_all(output_dir).ok();
    fs::create_dir_all(output_dir).unwrap_or_else(|_| panic!("Could not create directory: {}", output_dir.display()));

    // Traverse and render files recursively
    traverse_and_render(&TEMPLATES_DIR, output_dir, config_data, &mut generated_files);

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
    let template_content = file.contents_utf8().expect("Template file is not valid UTF-8");

    // Compute the relative path from the root of TEMPLATES_DIR
    let relative_path = template_path
        .strip_prefix(TEMPLATES_DIR.path())
        .expect("Failed to strip TEMPLATES_DIR prefix");

    // Combine the base output directory with the relative path
    let output_path = base_output_dir.join(relative_path);

    // Log the paths for debugging
    println!(
        "Rendering template: '{}', saving to '{}'",
        relative_path.display(),
        output_path.display()
    );

    // Render the template content
    let output = render_template(template_content, config_data);

    // Write the rendered content to the output file
    write_to_file(output_path.to_str().unwrap(), &output);

    // Add the generated file to the list
    generated_files.push(output_path.to_str().unwrap().to_string());
}

// Render a single template using Tera
fn render_template(template_content: &str, data: &HashMap<String, Value>) -> String {
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

// Write the rendered output to a file
fn write_to_file(output_path: &str, content: &str) {
    let path = Path::new(output_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|_| panic!("Could not create directory: {}", parent.display()));
    }
    fs::write(output_path, content)
        .unwrap_or_else(|_| panic!("Could not write to file at path: {}", output_path));
}

// Run `docker compose up -d` for all generated files
fn start_docker_compose(force: bool, output_dir: &Path) {
    let mut recreate: &str = "";
    let compose_file = output_dir.join("cluster.yaml");
    if !compose_file.exists() {
        eprintln!(
            "Error: Docker Compose file '{}' does not exist.",
            compose_file.display()
        );
        std::process::exit(1);
    }

    if force == true {
        recreate = "--force-recreate";
        println!("Restarting Docker Compose services...");
    } else {
        println!("Starting Docker Compose services...");
    }

    let mut binding = ShellCommand::new("docker");
    let mut command = binding
        .arg("compose")
        .arg("--file")
        .arg(compose_file)
        .arg("up")
        .arg("-d");
    if force {
        command = command.arg(recreate);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute 'docker compose up -d'");

    let stdout = child.stdout.take().expect("Could not capture stdout");
    let stderr = child.stderr.take().expect("Could not capture stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    for line in stdout_reader.lines() {
        println!("{}", line.unwrap());
    }

    for line in stderr_reader.lines() {
        eprintln!("{}", line.unwrap());
    }

    let status = child.wait().expect("Failed to wait for command");
    if !status.success() {
        eprintln!("Command failed with status: {}", status);
    }
}

// Run `docker compose down` for all generated files
fn stop_docker_compose(output_dir: &Path) {
    let compose_file = output_dir.join("cluster.yaml");
    if !compose_file.exists() {
        eprintln!(
            "Error: Docker Compose file '{}' does not exist.",
            compose_file.display()
        );
        std::process::exit(1);
    }

    let mut command = ShellCommand::new("docker")
        .arg("compose")
        .arg("--file")
        .arg(compose_file)
        .arg("down")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute 'docker compose down'");

    let stdout = command.stdout.take().expect("Could not capture stdout");
    let stderr = command.stderr.take().expect("Could not capture stderr");

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    for line in stdout_reader.lines() {
        println!("{}", line.unwrap());
    }

    for line in stderr_reader.lines() {
        eprintln!("{}", line.unwrap());
    }

    let status = command.wait().expect("Failed to wait for command");
    if !status.success() {
        eprintln!("Command failed with status: {}", status);
    }
}
