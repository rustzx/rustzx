use std::{
    path::{Path, PathBuf},
    process::Command,
};
use anyhow::Context;

const DOCKER_IMAGE_REPO: &str = "https://github.com/z88dk/z88dk.git";
const DOCKER_IMAGE_COMMIT: &str = "d61f6bb46ec15775cccf543f5941b6a2d6864ecf";
const DOCKER_IMAGE_NAME: &str = "rustzx/z88dk";
const DOCKER_IMAGE_FILE: &str = "z88dk.Dockerfile";

const HEX_ALPHABET: [char; 16] = [
    '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f',
];
const SHORTENED_COMMIT_LENGTH: usize = 7;

fn image_tagged_name(name: impl AsRef<str>, tag: impl Into<String>) -> String {
    let mut tag = tag.into();
    tag.truncate(SHORTENED_COMMIT_LENGTH);

    format!("{}:{}", name.as_ref(), tag)
}

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

        execute_command_transparent(cmd)?;
        Ok(())
    }
}

struct DockerImage {
    url: String,
    name: Option<String>,
    dockerfile: Option<String>,
}

impl DockerImage {
    pub fn present_on_machine(image_name: &str) -> Result<bool, anyhow::Error> {
        let mut cmd = Command::new("docker");
        cmd.args(["images", "-q", image_name]);

        let output = cmd
            .output()
            .with_context(|| "Failed to query docker images")
            .and_then(|out| {
                String::from_utf8(out.stdout)
                    .with_context(|| "Failed to parse docker stdout")
            })?;

        return Ok(!output.is_empty())
    }

    pub fn from_git(repo: impl AsRef<str>, commit: impl AsRef<str>) -> Self {
        let url = format!("{}#{}", repo.as_ref(), commit.as_ref());

        Self {
            url,
            name: None,
            dockerfile: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name.replace(name.into());
        self
    }

    pub fn with_dockerfile(mut self, dockerfile: impl Into<String>) -> Self {
        self.dockerfile.replace(dockerfile.into());
        self
    }

    pub fn build(self) -> anyhow::Result<()> {
        let mut cmd = Command::new("docker");
        cmd.arg("build");
        if let Some(dockerfile) = self.dockerfile {
            cmd.args(["-f", &dockerfile]);
        }
        if let Some(name) = self.name {
            cmd.args(["-t", &name]);
        }
        cmd.arg(self.url);
        cmd.spawn()?.wait()?;
        execute_command_transparent(cmd)?;
        Ok(())
    }
}

fn to_container_name(prefix: &str) -> String {
    format!("{}-{}", prefix, nanoid::nanoid!(8, &HEX_ALPHABET))
}

fn execute_command_transparent(mut cmd: Command) -> anyhow::Result<()> {
    println!("Running command with args: {:#?}", cmd);

    let status = cmd.spawn()?.wait()?;

    if !status.success() {
        anyhow::anyhow!(
            "Command execution failed with code {}",
            status.code().unwrap_or(1)
        );
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    if !DockerImage::present_on_machine(DOCKER_IMAGE_NAME)
        .with_context(|| "Failed to check for docker image presence")?
    {
        DockerImage::from_git(DOCKER_IMAGE_REPO, DOCKER_IMAGE_COMMIT)
            .with_name(image_tagged_name(DOCKER_IMAGE_NAME, DOCKER_IMAGE_COMMIT))
            .with_dockerfile(DOCKER_IMAGE_FILE)
            .build()
            .with_context(|| "Failed to build docker image")?;
    }

    let mut assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assets_dir.push(Path::new("test_data"));

    DockerContainer::from_image(image_tagged_name(DOCKER_IMAGE_NAME, DOCKER_IMAGE_COMMIT))
        .name(to_container_name("rustzx-assets"))
        .mount(DockerMountPoint {
            dir: assets_dir,
            mount: ForeignOsPath::new("/src/"),
        })
        .remove_after_run()
        .startup_script(ForeignOsPath::new("/src/make.sh"))
        .run()
        .expect("Failed to build assets");

    Ok(())
}
