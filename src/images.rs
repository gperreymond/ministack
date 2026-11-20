// Copy docker images of the host into the docker engine of every nomad client,
// so the jobs can use an image built locally, without a registry

use std::process::{Command, Stdio};

// true when the command succeeds, its output is discarded
fn succeeds(command: &mut Command) -> bool {
    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn is_running(container: &str) -> bool {
    Command::new("docker")
        .args(["inspect", "--format", "{{.State.Running}}", container])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "true")
        .unwrap_or(false)
}

// docker save <image> | docker exec -i <client> docker load
fn copy_image(image: &str, client: &str) -> Result<(), String> {
    let mut save = Command::new("docker")
        .args(["save", image])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not run 'docker save': {}", e))?;
    let archive = save.stdout.take().expect("docker save stdout is piped");
    let load = Command::new("docker")
        .args(["exec", "-i", client, "docker", "load", "--quiet"])
        .stdin(Stdio::from(archive))
        .stdout(Stdio::null())
        .status()
        .map_err(|e| format!("could not run 'docker exec': {}", e))?;
    let saved = save
        .wait()
        .map_err(|e| format!("'docker save' failed: {}", e))?;
    if !saved.success() {
        return Err(format!("'docker save {}' failed", image));
    }
    if !load.success() {
        return Err(format!("'docker load' failed in '{}'", client));
    }
    Ok(())
}

// Copy every image into every client, false (and an error message) on the first problem
pub fn copy_images(images: &[String], clients: &[String]) -> bool {
    if images.is_empty() {
        return true;
    }
    if clients.is_empty() {
        eprintln!("Error: the cluster has no nomad clients to copy the images to.");
        return false;
    }
    for image in images {
        if !succeeds(Command::new("docker").args(["image", "inspect", image])) {
            eprintln!(
                "Error: the image '{}' is not on the host, build or pull it first.",
                image
            );
            return false;
        }
    }
    for client in clients {
        if !is_running(client) {
            eprintln!(
                "Error: the nomad client '{}' is not running, start the cluster first.",
                client
            );
            return false;
        }
    }
    for image in images {
        for client in clients {
            if let Err(error) = copy_image(image, client) {
                eprintln!("Error: {}", error);
                return false;
            }
            println!("[INFO] image '{}' copied to '{}'", image, client);
        }
    }
    true
}
