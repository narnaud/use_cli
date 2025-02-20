use clap::Parser;
use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;
use std::str;

static CONFIG_FILE_NAME: &str = ".useconfig.json";
static CONFIG_FILE_EXAMPLE: &str = r#"
{
    "example": {
        "display": "Name of the configuration",
        "use": [
            "other",
            "configuration",
            "names"
        ],
        "defer": [
            "C:\\example\\path\\to\\script.bat",
            "C:\\example\\other\\path\\to\\script.bat"
        ],
        "set": {
            "EXAMPLE_VAR": "example value"
        },
        "append": {
            "EXAMPLE_VAR_APPEND": "value appended to EXAMPLE_VAR_APPEND"
        },
        "prepend": {
            "EXAMPLE_VAR_PREPEND": "value prepended to EXAMPLE_VAR_PREPEND"
        },
        "path": [
            "C:\\example\\path\\to\\add\\to\\path",
            "C:\\example\\other\\path\\to\\add\\to\\path"
        ],
        "go": "C:\\example\\path\\to\\go\\to",
    },
    "msvc2022": {
        "display": "Microsoft Visual Studio 2022 - x64",
        "defer": [
            "C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional\\VC\\Auxiliary\\Build\\vcvars64.bat"
        ]
    },
    "qt6.8": {
        "display": "Qt 6.8.2 - MSVC - x64",
        "use": [
            "msvc2022"
        ],
        "set": {
            "QTDIR": "C:\\Qt\\6.8.2\\msvc2019_64\\"
        },
        "append": {
            "CMAKE_PREFIX_PATH": "C:\\Qt\\6.8.2\\msvc2019_64\\"
        },
        "path": [
            "C:\\Qt\\6.8.2\\msvc2019_64\\bin"
        ]
    },
}
"#;

#[derive(Debug, Deserialize, Clone, PartialEq)]
struct Environment {
    display: Option<String>,
    defer: Option<Vec<String>>,
    set: Option<HashMap<String, String>>,
    append: Option<HashMap<String, String>>,
    prepend: Option<HashMap<String, String>>,
    path: Option<Vec<String>>,
    #[serde(rename = "use")]
    reuse: Option<Vec<String>>,
    go: Option<String>,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the environment to use
    env_name: Option<String>,
    /// List all environments
    #[clap(short, long)]
    list: bool,
    /// Create a new config file
    #[clap(short, long)]
    create: bool,
}

fn main() {
    env_logger::init();

    let args = Args::parse();

    let mut config_file_path = dirs::home_dir().expect("Could not find home directory");
    config_file_path.push(CONFIG_FILE_NAME);
    let config_file = config_file_path.to_str().unwrap();

    if args.create {
        create_config_file(config_file);
        println!("Created {} file", config_file);
        std::process::exit(0);
    }

    if !config_file_path.exists() {
        print!("Error {} does not exist", config_file);
        std::process::exit(1);
    }
    debug!("Find config file: {}", config_file);

    let environments = match read_config_file(config_file) {
        Ok(environments) => environments,
        Err(e) => {
            println!("Error reading {} file: {}", config_file, e);
            std::process::exit(1);
        }
    };
    debug!("Read config file");

    if args.list || args.env_name.is_none() {
        list_environments(environments);
        std::process::exit(0);
    }

    let env_name = args.env_name.unwrap();
    if !environments.contains_key(env_name.as_str()) {
        println!("Error: Environment {} not found", env_name);
        std::process::exit(1);
    }
    debug!("Use environment: {}", env_name);

    let env_names = list_all_envs_for(env_name, &environments);
    debug!("Setup environments: {:?}", env_names);

    for env_name in env_names.iter().rev() {
        let env = environments.get(env_name).unwrap();
        print_environment(env);
    }
}

/// Create a config file in the home directory if it does not exist
fn create_config_file(path: &str) {
    // Open the file and writhe the CONFIG_FILE_CONTENT to it
    let mut file = std::fs::File::create(path).expect("Failed to create file");
    file.write_all(CONFIG_FILE_EXAMPLE.as_bytes()).expect("Failed to write to file");
}

/// Read the congig file and return a map of environments
fn read_config_file(file_path: &str) -> Result<HashMap<String, Environment>, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

/// Function to list all environments in the config file
fn list_environments(envs: HashMap<String, Environment>) {
    // Get keys from configs map, sort then and print them
    let mut keys: Vec<_> = envs.keys().collect();
    keys.sort();
    keys.iter().for_each(|key| println!("{}", key));
}

/// List all environment that should be used based on the environment name
fn list_all_envs_for(
    env_name: String,
    envs: &HashMap<String, Environment>,
) -> Vec<String> {
    let mut env_names = vec![env_name.clone()];
    let env = envs.get(env_name.as_str()).unwrap();

    if let Some(reuse) = env.reuse.as_ref() {
        for env_name in reuse.iter() {
            let reuse_env_names = list_all_envs_for(env_name.clone(), envs);
            // Add the environment to the list of environments to use
            // Only if it is not already in the list
            for reuse_env_name in reuse_env_names.iter() {
                if !env_names.contains(reuse_env_name) {
                    env_names.push(reuse_env_name.clone());
                }
            }
        }
    }

    env_names
}

/// Print the environment to the console
fn print_environment(env: &Environment) {
    let print_map = |label: &str, map: &Option<HashMap<String, String>>| {
        if let Some(map) = map {
            for (key, value) in map {
                println!("{}: {} = {}", label, key, value);
            }
        }
    };

    let print_vec = |label: &str, vec: &Option<Vec<String>>| {
        if let Some(vec) = vec {
            for item in vec {
                println!("{}: {}", label, item);
            }
        }
    };

    if let Some(display) = &env.display {
        println!("DISPLAY: {}", display);
    }
    print_vec("DEFER", &env.defer);
    print_map("SET", &env.set);
    print_map("APPEND", &env.append);
    print_map("PREPEND", &env.prepend);
    print_vec("PATH", &env.path);
    if let Some(go) = &env.go {
        println!("GO: {}", go);
    }
}
