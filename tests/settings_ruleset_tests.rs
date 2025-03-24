use std::sync::Arc;
use subconverter::settings::settings::settings_struct::Settings;
use subconverter::settings::settings::update_settings_from_content;

#[cfg(test)]
mod settings_ruleset_tests {
    use super::*;

    #[test]
    fn test_custom_rulesets_yaml() {
        let yaml_content = r#"
common:
  enable_rule_gen: true
  update_ruleset_on_request: true
  overwrite_original_rules: true

custom_rulesets:
  - name: "Netflix"
    url: "https://example.com/netflix.list"
    type: "surge-ruleset"
  - name: "YouTube"
    url: "https://example.com/youtube.list"
    type: "clash-domain"
  - name: "Direct"
    url: "https://example.com/direct.list"
    type: "surge-ruleset"
        "#;

        // Update settings with YAML content containing custom rulesets
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify ruleset settings
        assert_eq!(settings.enable_rule_gen, true);
        assert_eq!(settings.update_ruleset_on_request, true);
        assert_eq!(settings.overwrite_original_rules, true);

        // Verify custom rulesets
        assert_eq!(settings.custom_rulesets.len(), 3);
        assert_eq!(settings.custom_rulesets[0].group, "Netflix");
        assert_eq!(
            settings.custom_rulesets[0].url,
            "https://example.com/netflix.list"
        );
        assert_eq!(settings.custom_rulesets[0].interval, 300);

        assert_eq!(settings.custom_rulesets[1].group, "YouTube");
        assert_eq!(
            settings.custom_rulesets[1].url,
            "https://example.com/youtube.list"
        );
        assert_eq!(settings.custom_rulesets[1].interval, 300);
    }

    #[test]
    fn test_custom_rulesets_toml() {
        let toml_content = r#"
[common]
enable_rule_gen = true
update_ruleset_on_request = true
overwrite_original_rules = true

[[custom_rulesets]]
name = "Netflix"
url = "https://example.com/netflix.list"
type = "surge-ruleset"

[[custom_rulesets]]
name = "YouTube"
url = "https://example.com/youtube.list"
type = "clash-domain"

[[custom_rulesets]]
name = "Direct"
url = "https://example.com/direct.list"
type = "surge-ruleset"
        "#;

        // Update settings with TOML content containing custom rulesets
        update_settings_from_content(toml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify ruleset settings
        assert_eq!(settings.enable_rule_gen, true);
        assert_eq!(settings.update_ruleset_on_request, true);
        assert_eq!(settings.overwrite_original_rules, true);

        // Verify custom rulesets
        assert_eq!(settings.custom_rulesets.len(), 3);
        assert_eq!(settings.custom_rulesets[0].group, "Netflix");
        assert_eq!(
            settings.custom_rulesets[0].url,
            "https://example.com/netflix.list"
        );
        assert_eq!(settings.custom_rulesets[0].interval, 300);

        assert_eq!(settings.custom_rulesets[1].group, "YouTube");
        assert_eq!(
            settings.custom_rulesets[1].url,
            "https://example.com/youtube.list"
        );
        assert_eq!(settings.custom_rulesets[1].interval, 300);
    }

    #[test]
    fn test_custom_rulesets_ini() {
        let ini_content = r#"
[common]
enable_rule_gen=true
update_ruleset_on_request=true
overwrite_original_rules=true

[ruleset]
ruleset=Netflix,https://example.com/netflix.list,surge-ruleset
ruleset=YouTube,https://example.com/youtube.list,clash-domain
ruleset=Direct,https://example.com/direct.list,surge-ruleset
        "#;

        // Update settings with INI content containing custom rulesets
        update_settings_from_content(ini_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify ruleset settings
        assert_eq!(settings.enable_rule_gen, true);
        assert_eq!(settings.update_ruleset_on_request, true);
        assert_eq!(settings.overwrite_original_rules, true);

        // Verify custom rulesets
        assert_eq!(settings.custom_rulesets.len(), 3);
        assert_eq!(settings.custom_rulesets[0].group, "Netflix");
        assert_eq!(
            settings.custom_rulesets[0].url,
            "https://example.com/netflix.list"
        );
        assert_eq!(settings.custom_rulesets[0].interval, 300);

        assert_eq!(settings.custom_rulesets[1].group, "YouTube");
        assert_eq!(
            settings.custom_rulesets[1].url,
            "https://example.com/youtube.list"
        );
        assert_eq!(settings.custom_rulesets[1].interval, 300);
    }

    #[test]
    fn test_ruleset_limits() {
        let yaml_content = r#"
common:
  enable_rule_gen: true
  max_allowed_rulesets: 10
  max_allowed_rules: 1000
        "#;

        // Update settings with YAML content containing ruleset limits
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify ruleset limits
        assert_eq!(settings.enable_rule_gen, true);
        assert_eq!(settings.max_allowed_rulesets, 10);
        assert_eq!(settings.max_allowed_rules, 1000);
    }
}
