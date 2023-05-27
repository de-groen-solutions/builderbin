use std::collections::HashMap;

#[derive(serde::Deserialize, Debug)]
struct Document {
    images: HashMap<String, Image>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum Image {
    Based {
        base: String,
        steps: Vec<Step>,
        #[serde(rename = "after-all")]
        after_all: Option<Vec<Step>>,
    },
    Root {
        image: String,
        steps: Vec<Step>,
        #[serde(rename = "after-all")]
        after_all: Option<Vec<Step>>,
    },
}
#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum Step {
    Run { run: String },
    Sources { sources: String },
    Install { install: Vec<String> },
    Env { env: HashMap<String, String> },
}

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);
    if !path.is_file() {
        panic!("path is not a file");
    }

    let model: Document = serde_yaml::from_reader(std::fs::File::open(path).unwrap()).unwrap();

    for (name, image) in model.images {
        let output_path = path
            .parent()
            .unwrap()
            .join("dockerfiles")
            .join(format!("{}.Dockerfile", normalize_name(&name)));

        let mut writer = std::fs::File::create(&output_path).unwrap();
        image.print(&mut writer);
    }
}

fn normalize_name(name: &str) -> String {
    name.replace(['/', ':', '.'], "-")
}

trait Printable {
    fn print(&self, writer: &mut dyn std::io::Write);
}

impl Printable for Image {
    fn print(&self, writer: &mut dyn std::io::Write) {
        match self {
            Image::Based {
                base,
                steps,
                after_all,
            } => {
                writeln!(writer, "FROM --platform=linux/amd64 {}", base).unwrap();
                writeln!(writer).unwrap();
                for step in steps {
                    step.print(writer);
                    writeln!(writer, ).unwrap();
                }
                if let Some(after_all) = after_all {
                    for step in after_all {
                        step.print(writer);
                        writeln!(writer, ).unwrap();
                    }
                }
            }
            Image::Root {
                image,
                steps,
                after_all,
            } => {
                writeln!(writer, "FROM --platform=linux/amd64 {}", image).unwrap();
                for step in steps {
                    step.print(writer);
                }
                if let Some(after_all) = after_all {
                    for step in after_all {
                        step.print(writer);
                    }
                }
            }
        }
    }
}

impl Printable for Step {
    fn print(&self, writer: &mut dyn std::io::Write) {
        match self {
            Step::Run { run } => {
                writeln!(writer, 
                    "RUN {} || exit 99",
                    run.split('\n')
                        .filter(|x| !x.is_empty())
                        .collect::<Vec<_>>()
                        .join(" \\\n    && ")
                ).unwrap();
            }
            Step::Sources { sources } => {
                writeln!(writer, 
                    "RUN echo {:?} > /etc/apt/sources.list",
                    sources
                ).unwrap();
            }
            Step::Install { install } => {
                writeln!(writer, 
                    "RUN wajig update \\\n    && wajig install -y \\\n      {} \\\n    && wajig clean || exit 99",
                    install.join(" \\\n      ")
                ).unwrap();
            }
            Step::Env { env } => {
                writeln!(writer, 
                    "ENV {}",
                    env.iter()
                        .map(|(k, v)| format!("{}={:?}", k, v))
                        .collect::<Vec<_>>()
                        .join(" \\\n    ")
                ).unwrap();
            }
        }
    }
}
