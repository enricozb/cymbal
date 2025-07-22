use std::path::PathBuf;

use anyhow::{Context, Result};

use super::raw::RawConfig;

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub global_config: Option<PathBuf>,
    pub user_config: Option<PathBuf>,
    pub project_config: Option<PathBuf>,
}

impl ConfigPaths {
    pub fn detect() -> Self {
        let global_config = dirs::config_dir().map(|dir| dir.join("cymbal").join("config.toml"));

        let user_config = dirs::home_dir().map(|dir| dir.join(".cymbal.toml"));

        let project_config = std::env::current_dir()
            .ok()
            .and_then(|mut dir| {
                loop {
                    let config_path = dir.join(".cymbal.toml");
                    if config_path.exists() {
                        return Some(config_path);
                    }

                    let cymbal_dir = dir.join(".cymbal");
                    let cymbal_config = cymbal_dir.join("config.toml");
                    if cymbal_config.exists() {
                        return Some(cymbal_config);
                    }

                    if !dir.pop() {
                        break;
                    }
                }
                None
            });

        Self {
            global_config,
            user_config,
            project_config,
        }
    }

    pub fn iter_existing(&self) -> impl Iterator<Item = &PathBuf> {
        [&self.global_config, &self.user_config, &self.project_config]
            .into_iter()
            .flatten()
            .filter(|path| path.exists())
    }
}

#[derive(Debug, Clone)]
pub struct ConfigLoader {
    paths: ConfigPaths,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self {
            paths: ConfigPaths::detect(),
        }
    }

    pub fn load_merged_config(&self, user_specified_path: Option<&str>) -> Result<RawConfig> {
        let mut merged_config = RawConfig::default();

        for config_path in self.paths.iter_existing() {
            let config_content = std::fs::read_to_string(config_path)
                .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

            let config: RawConfig = toml::from_str(&config_content)
                .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

            merged_config = merge_configs(merged_config, config);
        }

        if let Some(user_path) = user_specified_path {
            let user_config = if user_path.ends_with(".toml") {
                let config_content = std::fs::read_to_string(user_path)
                    .with_context(|| format!("Failed to read user-specified config: {}", user_path))?;

                toml::from_str(&config_content)
                    .with_context(|| format!("Failed to parse user-specified config: {}", user_path))?
            } else {
                toml::from_str(user_path)
                    .context("Failed to parse user-specified config string")?
            };

            merged_config = merge_configs(merged_config, user_config);
        }

        Ok(merged_config)
    }

    pub fn get_config_paths(&self) -> &ConfigPaths {
        &self.paths
    }
}

fn merge_configs(mut base: RawConfig, overlay: RawConfig) -> RawConfig {
    for (language, queries) in overlay.languages {
        base.languages.insert(language, queries);
    }
    base
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_paths_detection() {
        let paths = ConfigPaths::detect();

        if let Some(global) = &paths.global_config {
            assert!(global.ends_with("cymbal/config.toml"));
        }

        if let Some(user) = &paths.user_config {
            assert!(user.ends_with(".cymbal.toml"));
        }
    }

    #[test]
    fn test_merge_configs() {
        let base = RawConfig::default();
        let overlay = RawConfig::default();

        let merged = merge_configs(base, overlay);
        assert!(!merged.languages.is_empty());
    }
}
