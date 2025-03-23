use std::sync::{Arc, RwLock};

use crate::models::ruleset::RulesetContent;
use crate::models::ExtraSettings;
use crate::settings::config::Settings;
use crate::settings::external::ExternalConfig;
use crate::settings::ruleset::{refresh_rulesets, RulesetConfig};

/// Unified settings that combines global settings with per-request configs
pub struct UnifiedSettings {
    /// Global settings (loaded from config file)
    pub global: Arc<RwLock<Settings>>,
    /// Global ruleset content cache
    pub ruleset_content: Arc<RwLock<Vec<RulesetContent>>>,
}

/// Settings resulting from merging global and external configs
#[derive(Debug, Clone)]
pub struct MergedSettings {
    /// Base settings from global config
    pub base: Settings,
    /// External configuration (from URL params or external file)
    pub external: ExternalConfig,
    /// Extra settings for export operations
    pub extra: ExtraSettings,
    /// Ruleset content for this request
    pub ruleset_content: Vec<RulesetContent>,
}

impl Default for UnifiedSettings {
    fn default() -> Self {
        Self {
            global: Arc::new(RwLock::new(Settings::default())),
            ruleset_content: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

lazy_static::lazy_static! {
    /// Global singleton instance of unified settings
    pub static ref INSTANCE: UnifiedSettings = UnifiedSettings::default();
}

impl UnifiedSettings {
    /// Get global settings
    pub fn get_global(&self) -> Settings {
        self.global.read().unwrap().clone()
    }

    /// Update global settings
    pub fn update_global(&self, settings: Settings) {
        *self.global.write().unwrap() = settings;
    }

    /// Refresh global rulesets
    pub fn refresh_global_rulesets(&self) {
        let settings = self.get_global();

        // Convert string rulesets to RulesetConfig format
        let rulesets: Vec<RulesetConfig> = settings
            .custom_rulesets
            .iter()
            .filter_map(|r| RulesetConfig::from_str(r))
            .collect();

        // Refresh ruleset content
        let mut ruleset_content = self.ruleset_content.write().unwrap();
        refresh_rulesets(&rulesets, &mut ruleset_content);
    }

    /// Get current ruleset content
    pub fn get_ruleset_content(&self) -> Vec<RulesetContent> {
        self.ruleset_content.read().unwrap().clone()
    }

    /// Create merged settings from global and external configs
    pub fn create_merged(&self, external_config: Option<ExternalConfig>) -> MergedSettings {
        let global = self.get_global();
        let external = external_config.unwrap_or_default();

        // Create extra settings from the combination
        let extra = self.derive_extra_settings(&global, &external);

        // Handle rulesets - use external rulesets if provided, otherwise use global ones
        let ruleset_content = if !external.surge_ruleset.is_empty() {
            // Use external rulesets
            let mut content = Vec::new();
            refresh_rulesets(&external.surge_ruleset, &mut content);
            content
        } else {
            // Use global rulesets
            self.get_ruleset_content()
        };

        MergedSettings {
            base: global,
            external,
            extra,
            ruleset_content,
        }
    }

    /// Derive ExtraSettings from Settings and ExternalConfig
    fn derive_extra_settings(
        &self,
        settings: &Settings,
        external: &ExternalConfig,
    ) -> ExtraSettings {
        let mut extra = ExtraSettings::default();

        // Rule generation settings
        extra.enable_rule_generator = match external.enable_rule_generator {
            Some(val) => val,
            None => settings.enable_rule_gen,
        };

        extra.overwrite_original_rules = match external.overwrite_original_rules {
            Some(val) => val,
            None => settings.overwrite_original_rules,
        };

        // Emoji settings
        extra.add_emoji = match external.add_emoji {
            Some(val) => val,
            None => settings.add_emoji,
        };

        extra.remove_emoji = match external.remove_old_emoji {
            Some(val) => val,
            None => settings.remove_emoji,
        };

        // Other preferences
        extra.append_proxy_type = settings.append_type;
        extra.sort_flag = settings.enable_sort;
        extra.filter_deprecated = settings.filter_deprecated;
        extra.clash_new_field_name = settings.clash_use_new_field;

        // Base paths
        extra.surge_ssr_path = settings.surge_ssr_path.clone();
        extra.managed_config_prefix = settings.managed_config_prefix.clone();
        extra.quanx_dev_id = settings.quanx_dev_id.clone();

        // Proxy flags
        extra.udp = settings.udp_flag;
        extra.tfo = settings.tfo_flag;
        extra.skip_cert_verify = settings.skip_cert_verify;
        extra.tls13 = settings.tls13_flag;

        // Clash styles
        extra.clash_proxies_style = settings.clash_proxies_style.clone();
        extra.clash_proxy_groups_style = settings.clash_proxy_groups_style.clone();

        // Sort script
        extra.sort_script = settings.sort_script.clone();

        // Process rename and emoji arrays
        extra.rename_array = external.rename.clone();
        extra.emoji_array = external.emoji.clone();

        extra
    }
}

/// Initialize the settings system
pub fn init_settings(settings_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match Settings::load_from_file(settings_path) {
        Ok(settings) => {
            INSTANCE.update_global(settings);
            INSTANCE.refresh_global_rulesets();
            Ok(())
        }
        Err(err) => Err(err),
    }
}

/// Get the unified settings instance
pub fn get_instance() -> &'static UnifiedSettings {
    &INSTANCE
}
