use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use appcore_security::{format_secret_material, new_rotated_secret};

const APPLICATION_MANIFEST: &str = "application.toml";
const DEPLOYMENT_MANIFEST: &str = "deployment.toml";
const RUNTIME_SECRET: &str = "target/runtime/runtime-security.secret";
const EMBEDDED_APPLICATION: &[u8] = include_bytes!("../application.toml");
const EMBEDDED_DEPLOYMENT: &[u8] = include_bytes!("../deployment.local.toml");

struct PackagedLayout {
    application: PathBuf,
    deployment: PathBuf,
    secret: PathBuf,
}

fn packaged_layout(executable: &Path) -> Result<PackagedLayout, String> {
    let directory = executable
        .parent()
        .ok_or_else(|| "release executable path has no parent".to_string())?;
    let application = directory.join(APPLICATION_MANIFEST);
    let deployment = directory.join(DEPLOYMENT_MANIFEST);
    initialize_embedded_file(&application, EMBEDDED_APPLICATION)?;
    initialize_embedded_file(&deployment, EMBEDDED_DEPLOYMENT)?;
    Ok(PackagedLayout {
        application,
        deployment,
        secret: directory.join(RUNTIME_SECRET),
    })
}

fn initialize_embedded_file(path: &Path, contents: &[u8]) -> Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("failed to materialize {}: {error}", path.display()))?;
    file.write_all(contents)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("failed to persist {}: {error}", path.display()))
}

fn secret_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn initialize_secret(path: &Path) -> Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "runtime secret path has no parent".to_string())?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create runtime directory: {error}"))?;

    let material = new_rotated_secret(None)
        .map_err(|error| format!("failed to generate runtime secret: {error}"))?;
    let temporary = parent.join(format!(".runtime-security.{}.tmp", std::process::id()));
    let mut file = secret_options()
        .open(&temporary)
        .map_err(|error| format!("failed to create runtime secret: {error}"))?;
    file.write_all(format_secret_material(&material).as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("failed to persist runtime secret: {error}"))?;
    drop(file);
    fs::rename(&temporary, path)
        .map_err(|error| format!("failed to activate runtime secret: {error}"))?;
    Ok(())
}

pub(crate) fn configure() -> Result<(), String> {
    let application_override = std::env::var_os("APPCORE_APPLICATION_MANIFEST");
    let deployment_override = std::env::var_os("APPCORE_DEPLOYMENT_MANIFEST");
    if application_override.is_some() && deployment_override.is_some() {
        return Ok(());
    }
    let executable = std::env::current_exe()
        .map_err(|error| format!("failed to resolve executable path: {error}"))?;
    let layout = packaged_layout(&executable)?;

    if application_override.is_none() {
        std::env::set_var("APPCORE_APPLICATION_MANIFEST", &layout.application);
    }
    if deployment_override.is_none() {
        std::env::set_var("APPCORE_DEPLOYMENT_MANIFEST", &layout.deployment);
        initialize_secret(&layout.secret)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{initialize_secret, packaged_layout};

    #[test]
    fn packaged_layout_materializes_embedded_manifests_and_initializes_secret() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "proexel-release-layout-{}-{unique}",
            std::process::id(),
        ));
        std::fs::create_dir_all(&directory).unwrap();

        let layout = packaged_layout(&directory.join("proexel-service")).unwrap();
        initialize_secret(&layout.secret).unwrap();

        let application = std::fs::read_to_string(layout.application).unwrap();
        let deployment = std::fs::read_to_string(layout.deployment).unwrap();
        let secret = std::fs::read_to_string(layout.secret).unwrap();
        assert!(application.contains("application_id = \"proexel\""));
        assert!(deployment.contains("installation_id = \"proexel-local\""));
        assert!(secret.contains("status=active"));
        assert!(secret.contains("secret="));
    }
}
