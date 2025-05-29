use crate::conf;

use super::{BuildConfig, Config, DeployConfig, ServeConfig};
use std::env;
use std::fmt;
use std::mem;
use std::path::PathBuf;
use std::process;

#[derive(Clone, Copy)]
enum Subcommand {
    Build,
    Deploy,
    Serve,
}

#[derive(Debug, PartialEq)]
enum Arg {
    Short(char),
    Long(String),
    Value(String),
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arg::Short(x) => write!(f, "-{x}"),
            Arg::Long(x) => write!(f, "--{x}"),
            Arg::Value(x) => f.write_str(x),
        }
    }
}

struct ArgIter {
    arg: String,
    yielding_short: bool,
}

impl ArgIter {
    fn yield_short(&mut self) -> Option<Arg> {
        if let Some(c) = self.arg.chars().next() {
            self.arg.replace_range(..c.len_utf8(), "");
            Some(Arg::Short(c))
        } else {
            None
        }
    }
}

impl Iterator for ArgIter {
    type Item = Arg;

    fn next(&mut self) -> Option<Self::Item> {
        if self.arg.is_empty() {
            None
        } else if self.yielding_short {
            self.yield_short()
        } else if self.arg == "--" || self.arg.starts_with("---") {
            Some(Arg::Value(mem::take(&mut self.arg)))
        } else if self.arg.starts_with("--") {
            self.arg.replace_range(..2, "");
            Some(Arg::Long(mem::take(&mut self.arg)))
        } else if self.arg.starts_with("-") {
            self.yielding_short = true;
            self.arg.replace_range(..1, "");
            self.yield_short()
        } else {
            Some(Arg::Value(mem::take(&mut self.arg)))
        }
    }
}

fn parse_arg(arg: String) -> ArgIter {
    ArgIter {
        arg,
        yielding_short: false,
    }
}

fn print_usage(subcommand: Option<Subcommand>) {
    match subcommand {
        None => println!("usage: site [-h] {{build,deploy,serve}} ..."),
        Some(Subcommand::Build) => println!("usage: site build [-h] [-w] [-f] [--ignore-errors]"),
        Some(Subcommand::Deploy) => println!("usage: site deploy [-h] [TOKEN]"),
        Some(Subcommand::Serve) => println!("usage: site serve [-h] [-w]"),
    }
}

fn print_help(subcommand: Option<Subcommand>) {
    print_usage(subcommand);
    println!();
    match subcommand {
        None => {
            println!("positional arguments:");
            println!("  {{build,deploy,serve}}");
            println!("    build        build  the site");
            println!("    deploy       deploy the site");
            println!("    serve        serve  the site");
            println!();
            println!("options:");
            println!("  -h, --help     show this help message and exit");
        }
        Some(Subcommand::Build) => {
            println!("options:");
            println!("  -h, --help       show this help message and exit");
            println!("  -w, --write      write output instead of simply performing a dry run");
            println!("  -f, --force      delete output folder before writing");
            println!("  --ignore-errors  ignore errors during input processing");
        }
        Some(Subcommand::Deploy) => {
            println!("positional arguments:");
            println!("  TOKEN       when executing from the temporary directory and the parent");
            println!("              folder name matches this value, the deploy process continues");
            println!("options:");
            println!("  -h, --help  show this help message and exit");
        }
        Some(Subcommand::Serve) => {
            println!("options:");
            println!("  -h, --help   show this help message and exit");
            println!("  -w, --watch  watch for changes and automatically regenerate files");
        }
    }
}

pub fn parse() -> Config {
    let mut subcommand = Option::<Subcommand>::None;

    let mut write = false;
    let mut force = false;
    let mut ignore_errors = false;
    let mut watch = false;
    let mut token = None;

    for argument in env::args().skip(1).flat_map(parse_arg) {
        let shortcircuit_help = match &argument {
            Arg::Short('h') => true,
            Arg::Long(x) if x == "help" => true,
            _ => false,
        };
        if shortcircuit_help {
            print_help(subcommand);
            process::exit(0);
        }

        match subcommand {
            None => match argument {
                Arg::Value(x) if x == "build" => {
                    subcommand = Some(Subcommand::Build);
                }
                Arg::Value(x) if x == "deploy" => {
                    subcommand = Some(Subcommand::Deploy);
                }
                Arg::Value(x) if x == "serve" => {
                    subcommand = Some(Subcommand::Serve);
                }
                Arg::Value(x) => {
                    print_usage(subcommand);
                    println!(
                        "site: error: argument {{build,deploy,serve}}: invalid choice '{x}' (choose from build, deploy, serve)"
                    );
                    process::exit(1);
                }
                _ => {}
            },
            Some(Subcommand::Build) => match argument {
                Arg::Short('w') => write = true,
                Arg::Long(x) if x == "write" => write = true,
                Arg::Short('f') => force = true,
                Arg::Long(x) if x == "force" => force = true,
                Arg::Long(x) if x == "ignore-errors" => ignore_errors = true,
                arg => {
                    print_usage(subcommand);
                    println!("site: error: unrecognized arguments: {arg}");
                    process::exit(1);
                }
            },
            Some(Subcommand::Deploy) => match argument {
                Arg::Value(value) if token.is_none() => token = Some(value),
                arg => {
                    print_usage(subcommand);
                    println!("site: error: unrecognized arguments: {arg}");
                    process::exit(1);
                }
            },
            Some(Subcommand::Serve) => match argument {
                Arg::Short('w') => watch = true,
                Arg::Long(x) if x == "watch" => watch = true,
                arg => {
                    print_usage(subcommand);
                    println!("site: error: unrecognized arguments: {arg}");
                    process::exit(1);
                }
            },
        }
    }

    match subcommand {
        None => {
            print_usage(subcommand);
            println!("site: error: the following arguments are required: {{build,deploy,serve}}");
            process::exit(1);
        }
        Some(Subcommand::Build) => Config::Build(BuildConfig {
            write,
            force,
            ignore_errors,
            output_folder: PathBuf::from(conf::OUTPUT_FOLDER),
        }),
        Some(Subcommand::Deploy) => Config::Deploy(DeployConfig { token }),
        Some(Subcommand::Serve) => Config::Serve(ServeConfig { watch }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_arg() {
        assert_eq!(
            parse_arg("-wf".to_owned()).collect::<Vec<_>>(),
            vec![Arg::Short('w'), Arg::Short('f')]
        );
    }
}
