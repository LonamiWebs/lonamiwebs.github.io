use std::path::PathBuf;

pub enum Config {
    Build(BuildConfig),
    Deploy(DeployConfig),
    Serve(ServeConfig),
}

pub struct BuildConfig {
    pub write: bool,
    pub force: bool,
    pub ignore_errors: bool,
    pub output_folder: PathBuf,
}

pub struct DeployConfig {
    pub token: Option<String>,
}

pub struct ServeConfig {
    pub watch: bool,
}
