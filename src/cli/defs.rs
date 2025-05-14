pub enum Config {
    Build(BuildConfig),
    Serve(ServeConfig),
}

pub struct BuildConfig {
    pub write: bool,
    pub force: bool,
    pub ignore_errors: bool,
}

pub struct ServeConfig {
    pub watch: bool,
}
