use std::{
    env,
    ffi::OsString,
    io::{BufRead, BufReader, ErrorKind},
    path::PathBuf,
    process::{ExitStatus, Stdio},
};

use fs_extra::file::read_to_string;
use lazy_static::lazy_static;
use serde_json::{from_str, Value};

use crate::command as CommandTool;

lazy_static! {
    pub static ref NVMD_PATH: PathBuf = get_nvmd_path();
    pub static ref VERSION: String = get_version();
    pub static ref DEFAULT_INSTALLTION_PATH: PathBuf = get_default_installtion_path();
    pub static ref INSTALLTION_PATH: PathBuf = get_installtion_path();
    pub static ref NPM_PREFIX: PathBuf = get_npm_prefix();
    pub static ref ENV_PATH: OsString = get_env_path(false);
    pub static ref BINARY_ENV_PATH: OsString = get_env_path(true);
}

fn get_npm_prefix() -> PathBuf {
    let mut command = CommandTool::create_command("npm");

    let child = command
        .env("PATH", ENV_PATH.clone())
        .args(["config", "get", "prefix"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("nvmd-desktop: get npm perfix error");

    let output = child.stdout.unwrap();
    let lines = BufReader::new(output).lines();
    let mut perfix = String::from("");
    for line in lines {
        let cur_line = line.unwrap();
        if PathBuf::from(&cur_line).is_dir() {
            perfix = cur_line;
        }
    }

    PathBuf::from(perfix)
}

fn get_env_path(binary: bool) -> OsString {
    if VERSION.is_empty() {
        return OsString::from("");
    }

    let bin_path = match Some(binary) {
        Some(true) => get_binary_bin_path(),
        Some(false) => get_bin_path(),
        None => get_bin_path(),
    };

    if !PathBuf::from(&bin_path).exists() {
        return OsString::from("");
    }

    match env::var_os("PATH") {
        Some(path) => {
            let mut paths = env::split_paths(&path).collect::<Vec<_>>();
            paths.insert(0, PathBuf::from(bin_path));

            match env::join_paths(paths) {
                Ok(p) => p,
                Err(_) => OsString::from(""),
            }
        }
        None => bin_path,
    }
}

fn get_bin_path() -> OsString {
    let mut nvmd_path = INSTALLTION_PATH.clone();
    nvmd_path.push(VERSION.clone());

    if cfg!(unix) {
        nvmd_path.push("bin");
    }

    nvmd_path.into_os_string()
}

fn get_binary_bin_path() -> OsString {
    let mut nvmd_path = NPM_PREFIX.clone();

    if cfg!(unix) {
        nvmd_path.push("bin");
    }

    nvmd_path.into_os_string()
}

// $HOME/.nvmd/setting.json -> directory
fn get_installtion_path() -> PathBuf {
    let mut setting_path = NVMD_PATH.clone();
    setting_path.push("setting.json");

    let setting_content = match read_to_string(&setting_path) {
        Err(_) => String::from(""),
        Ok(content) => content,
    };

    if setting_content.is_empty() {
        return DEFAULT_INSTALLTION_PATH.clone();
    }

    let json_obj: Value = from_str(&setting_content).unwrap();

    if json_obj.is_null() || !json_obj.is_object() {
        return DEFAULT_INSTALLTION_PATH.clone();
    }

    if json_obj["directory"].is_null() || !json_obj["directory"].is_string() {
        return DEFAULT_INSTALLTION_PATH.clone();
    }

    let directory = json_obj["directory"].as_str().unwrap();

    PathBuf::from(directory)
}

fn get_default_installtion_path() -> PathBuf {
    let mut default_path = NVMD_PATH.clone();
    default_path.push("versions");

    default_path
}

fn get_version() -> String {
    let mut nvmdrc = match env::current_dir() {
        Err(_) => PathBuf::from(""),
        Ok(dir) => dir,
    };
    nvmdrc.push(".nvmdrc");

    let project_version = match read_to_string(&nvmdrc) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    if !project_version.is_empty() {
        return project_version;
    }

    let mut default_path = NVMD_PATH.clone();
    default_path.push("default");

    match read_to_string(&default_path) {
        Err(_) => String::from(""),
        Ok(v) => v,
    }
}

fn get_nvmd_path() -> PathBuf {
    match default_home_dir() {
        Ok(p) => p,
        Err(_) => PathBuf::from(""),
    }
}

fn default_home_dir() -> Result<PathBuf, ErrorKind> {
    let mut home = dirs::home_dir().ok_or(ErrorKind::NotFound)?;
    home.push(".nvmd");
    Ok(home)
}

pub enum Error {
    Message(String),
    Code(i32),
}

pub trait IntoResult<T> {
    fn into_result(self) -> Result<T, Error>;
}

impl IntoResult<()> for Result<ExitStatus, String> {
    fn into_result(self) -> Result<(), Error> {
        match self {
            Ok(status) => {
                if status.success() {
                    Ok(())
                } else {
                    let code = status.code().unwrap_or(1);
                    Err(Error::Code(code))
                }
            }
            Err(err) => Err(Error::Message(err)),
        }
    }
}
