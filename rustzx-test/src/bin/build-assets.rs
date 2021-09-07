use std::{
    path::{Path, PathBuf},
    process::Command,
};

const HEX_ALPHABET: [char; 16] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
];

// Running docker container OS may differ from the host, therefore
// we can't use Path/PathBuf
struct ForeignOsPath(String);

impl ForeignOsPath {
    fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }
}

struct DockerMountPoint {
    dir: PathBuf,
    mount: ForeignOsPath,
}

enum DockerAction {
    ShellScript { path: ForeignOsPath },
}

struct DockerContainer {
    image: String,
    name: Option<String>,
    mount_points: Vec<DockerMountPoint>,
    remove_after_run: bool,
    action: Option<DockerAction>,
}

impl DockerContainer {
    pub fn from_image(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            remove_after_run: false,
            mount_points: vec![],
            name: None,
            action: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name.replace(name.into());
        self
    }

    pub fn mount(mut self, mount_point: DockerMountPoint) -> Self {
        self.mount_points.push(mount_point);
        self
    }

    pub fn remove_after_run(mut self) -> Self {
        self.remove_after_run = true;
        self
    }

    pub fn startup_script(mut self, path: ForeignOsPath) -> Self {
        self.action.replace(DockerAction::ShellScript { path });
        self
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        let mut cmd = Command::new("docker");
        cmd.arg("run");
        if let Some(name) = self.name {
            cmd.args(&["--name", &name]);
        }
        for mount_point in self.mount_points {
            cmd.args(&[
                "-v",
                &format!("{}:{}", mount_point.dir.display(), mount_point.mount.0),
            ]);
        }
        if self.remove_after_run {
            cmd.arg("--rm");
        }
        if matches!(&self.action, Some(DockerAction::ShellScript { .. })) {
            cmd.arg("-it");
        }
        cmd.arg(self.image);
        if let Some(action) = self.action {
            match action {
                DockerAction::ShellScript { path } => {
                    cmd.args(&["sh", &path.0]);
                }
            }
        }

        println!("Running docker with args: {:#?}", cmd);

        let status = cmd.spawn()?.wait()?;

        if !status.success() {
            anyhow::anyhow!(
                "Docker execution failed with code {}",
                status.code().unwrap_or(1)
            );
        }

        Ok(())
    }
}

fn to_container_name(prefix: &str) -> String {
    format!("{}-{}", prefix, nanoid::nanoid!(8, &HEX_ALPHABET))
}

fn main() {
    let mut assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assets_dir.push(Path::new("test_data/asset"));

    DockerContainer::from_image("z88dk/z88dk")
        .name(to_container_name("rustzx-assets"))
        .mount(DockerMountPoint {
            dir: assets_dir,
            mount: ForeignOsPath::new("/src/"),
        })
        .remove_after_run()
        .startup_script(ForeignOsPath::new("/src/make.sh"))
        .run()
        .expect("Failed to build assets");
}
