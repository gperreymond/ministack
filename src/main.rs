use clap::{Arg, ArgGroup, Command};
use include_dir::{include_dir, Dir, File};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command as ShellCommand;
use tera::{Context, Tera};

// Embed the "templates" directory into the binary
static TEMPLATES_DIR: Dir = include_dir!("src/templates");

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
        .group(
            ArgGroup::new("action")
                .args(["start", "restart", "stop", "purge", "env"])
                .multiple(false),
        )
        .get_matches();

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

    // Load the YAML configuration file and render all templates
    let config_data = load_yaml(config_path);
    render_all_templates(&config_data, &output_dir);

    // Print the environment variables, no docker needed
    if matches.get_flag("env") {
        let env_file = output_dir.join("env.sh");
        print!("{}", fs::read_to_string(&env_file).unwrap_or_default());
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
    eprintln!(
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
        .unwrap_or_else(|e| panic!("Failed to execute 'docker compose {}': {}", args.join(" "), e));

    if !status.success() {
        eprintln!("Command 'docker compose {}' failed with status: {}", args.join(" "), status);
    }
    status.success()
}
