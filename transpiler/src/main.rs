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
       
    },
    Root {
        image: String,
        steps: Vec<Step>,
    },
}
#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum Step {
    Run { run: String },
    Sources { sources: String },
    Install { install: Vec<String> },
    Env { env: HashMap<String, String> },
//    Mount { kind: String, mode: Option<String>, host: Option<String>, container:String },
}

#[test]
fn parse_images() {
    let model = r#"
images:
  test:
    base: "ghcr.io/de-groen-solutions/builderbin-armhf-rust:18.04"
    steps:
      - run: echo Hello World
  another_image: # Another image key
    image: "another_image_here"
    steps:
      - run: echo Hello World

"#;

    let model: Document = serde_yaml::from_str(model).map_err(|e| {
        println!("Error: {:#?}", e);
        e
    }).unwrap();
    assert_eq!(model.images.len(), 2);
    assert!(model.images.contains_key("test"));
    assert!(model.images.contains_key("another_image"));
}

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let path = std::path::Path::new(&path);
    if !path.is_file() {
        panic!("path is not a file");
    }

    let model = std::fs::read_to_string(path)
        .map_err(|e| {
            println!("Error: {:#?}", e);
            std::process::exit(1);
        })
        .unwrap();

    let model = match serde_yaml::from_str::<Document>(&model){
        Ok(model) => model,
        Err(e) => {
            eprintln!("Error Type: {:#?}", e);
            eprintln!("Error: {}", e);

            std::process::exit(1); 
        }
    };

    let mut dependency_graph = HashMap::new();
    for (name, image) in &model.images {
        match image {
            Image::Based { base, .. } => {
                dependency_graph.insert(name, base);
            }
            Image::Root { .. } => {}
        }
    }

    use std::collections::HashSet;

    let mut build_order= Vec::new();
    let mut pending:HashSet<String> = HashSet::from_iter(model.images.keys().cloned());
    let mut limit = 1000;

    while !pending.is_empty() {
        limit -= 1;
        if limit == 0 {
            panic!("circular dependency detected");
        }
        for name in pending.clone() {
            let base = dependency_graph.get(&name);

            if base.is_none() || !pending.contains(base.unwrap().as_str()) {
                build_order.push(name.clone());
                pending.remove(&name);
            }
        }
    }

    let mut bash = Vec::new();
    bash.push("#!/bin/bash".to_string());
    bash.push("set -e".to_string());
    bash.push("base_dir=$(realpath $(dirname $0))".to_string());
    bash.push("mkdir -p .bbin/ .bbin/assets/ .bbin/cache/".to_string());
    bash.push(r#"build_image() {
    fname=$1
    tag=$2
    log=$3

    docker buildx build \
        --file $fname \
        --cache-from bambi.mr19.site:5000/bbin-cache \
        --cache-to bambi.mr19.site:5000/bbin-cache \
        --tag $tag \
        --load .bbin/assets/ &>> $log &

    last_log_line=""
    while kill -0 $! 2>/dev/null; do
        if [ "$(tail -n 1 $log)" != "$last_log_line" ]; then
            last_log_line="$(tail -n 1 $log)"
            echo -n "."
        fi
        sleep 0.2
    done

    if grep -q "ERROR" $log || grep -q "error: " $log; then
        echo "Log file:"
        echo "  $log"
        echo "Error building:"
        echo "  $tag"
        exit 1
    fi
}
"#.to_string());

let bbin = path
            .parent()
            .unwrap()
            .join(".bbin");

    std::fs::create_dir_all(&bbin).unwrap();

    for fullname in build_order.iter() {
        let imagenameversion = fullname.split('/').last()       .expect("invalid image name");
        let [name, version ]= imagenameversion.split(':').collect::<Vec<_>>()[..] else {
            panic!("invalid image name: {}", imagenameversion);
        };
        let logname = format!("{name}-{version}.log");
        bash.push(format!("touch .bbin/{logname}"));
        }

    let len = build_order.len();
    for (idx,fullname) in build_order.iter().enumerate() {
        let image = &model.images[fullname];

        let imagenameversion = fullname.split('/').last()       .expect("invalid image name");
        let [name, version ]= imagenameversion.split(':').collect::<Vec<_>>()[..] else {
            panic!("invalid image name: {}", imagenameversion);
        };
        let fname = format!("{name}-{version}.Dockerfile");
        
        let num = idx + 1;
        bash.push(format!("echo -n \"{num}/{len} - Building {name} tagged {version}\""));
        bash.push(format!("build_image .bbin/{fname} ghcr.io/de-groen-solutions/{name}:{version} .bbin/{name}-{version}.log"));
        bash.push(format!("echo \"\""));
        
        let output_path = path
            .parent()
            .unwrap()
            .join(".bbin")
            .join(fname);

        let mut writer = std::fs::File::create(&output_path).unwrap();
        image.print(&mut writer);
    }

    let output_path = path
        .parent()
        .unwrap()
        .join("build.sh");

std::    fs::write(&output_path, bash.join("\n")).unwrap();
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
            } => {
                writeln!(writer, "FROM --platform=linux/amd64 {}", base).unwrap();
                writeln!(writer).unwrap();
                for step in steps {
                    step.print(writer);
                    writeln!(writer, ).unwrap();
                }
            }
            Image::Root {
                image,
                steps,
            } => {
                writeln!(writer, "FROM --platform=linux/amd64 {}", image).unwrap();
                for step in steps {
                    step.print(writer);
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


