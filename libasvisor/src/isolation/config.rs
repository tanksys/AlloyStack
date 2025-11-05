use std::{collections::BTreeMap, fs, io::BufReader, path::PathBuf};

use anyhow;
use log::{debug, warn};
use as_hostcall::types::ServiceName;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

use std::io;

use crate::utils;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct App {
    pub name: ServiceName,
    pub args: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum IsolationGroupApp {
    Name(ServiceName),
    Detailed(App),
}

impl IsolationGroupApp {
    pub fn to_isolation(&self, id: String) -> App {
        let mut app = match self {
            IsolationGroupApp::Name(name) => App {
                name: name.to_owned(),
                args: BTreeMap::default(),
            },
            IsolationGroupApp::Detailed(app) => app.clone(),
        };

        app.args.insert("id".to_owned(), id);
        app
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IsolationGroup {
    pub list: Vec<IsolationGroupApp>,
    pub args: BTreeMap<String, String>,
}

impl IsolationGroup {
    pub fn to_isolation(&self) -> Vec<App> {
        self.list
            .iter()
            .enumerate()
            .map(|(idx, app)| {
                let mut app = app.to_isolation(idx.to_string());
                let mut args = self.args.clone();

                args.extend(app.args);
                app.args = args;
                debug!("App info: {:?}", app);

                app
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoadableUnit(
    pub ServiceName,
    #[serde(default = "Default::default")] 
    pub PathBuf,
);

#[derive(Serialize, Deserialize, Clone)]
pub struct IsolationConfig {
    pub services: Vec<LoadableUnit>,
    pub apps: Vec<LoadableUnit>,
    pub fs_image: Option<String>,
    pub with_libos: Option<bool>,
    #[serde(default = "Vec::default")]
    pub groups: Vec<IsolationGroup>,
}

impl IsolationConfig {
    pub fn to_file(&self, p: PathBuf) -> Result<(), io::Error> {
        fs::write(
            p,
            serde_json::to_string_pretty(self).expect("failed to pretty json string"),
        )?;
        Ok(())
    }

    pub fn from_file(p: PathBuf) -> Result<Self, anyhow::Error> {
        let p = if !p.is_file() {
            utils::ISOL_CONFIG_PATH.join(p)
        } else {
            p
        };

        debug!("config file path: {}", p.to_str().unwrap());
        let content = fs::File::open(p)?;

        let mut config: IsolationConfig = serde_json::from_reader(BufReader::new(content))?;
        for LoadableUnit(_, path) in config.services.iter_mut().chain(config.apps.iter_mut()) {
            *path = PathBuf::from("target")
                .join(if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                })
                .join(&path)
        }

        #[cfg(feature = "namespace")]
        config.services.insert(
            0,
            LoadableUnit(
                "libc".to_owned(),
                PathBuf::from("/usr/lib/x86_64-linux-gnu/libc.so.6"),
            ),
        );

        if config.with_libos.eq(&Some(false)) && !config.services.is_empty() {
            warn!("disable_libos is true, will ignore services");
        }

        Ok(config)
    }

    pub fn all_modules(&self) -> Vec<&LoadableUnit> {
        self.services.iter().chain(self.apps.iter()).collect()
    }
}

#[cfg(feature = "namespace")]
#[test]
fn all_modules_test() {
    let config = IsolationConfig::from_file(utils::ISOL_CONFIG_PATH.join("base_config.json"))
        .expect("load config failed");

    assert!(
        config
            .all_modules()
            .iter()
            .map(|x| x.0.clone())
            .collect::<Vec<String>>()
            .contains(&"libc".to_owned()),
        "{:?}",
        config.all_modules()
    )
}
