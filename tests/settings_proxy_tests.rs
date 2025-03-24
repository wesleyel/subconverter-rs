use std::sync::Arc;
use subconverter::settings::settings::settings_struct::Settings;
use subconverter::settings::settings::update_settings_from_content;

#[cfg(test)]
mod settings_proxy_tests {
    use subconverter::models::ProxyGroupType;

    use super::*;

    #[test]
    fn test_proxy_groups_yaml() {
        let yaml_content = r#"
common:
  enable_sort: true
  clash_use_new_field: true
  singbox_add_clash_modes: true
  clash_proxies_style: "flow"
  clash_proxy_groups_style: "compact"

custom_proxy_groups:
  - name: "Proxy"
    type: "select"
    rule: [".*"]
  - name: "Auto"
    type: "url-test"
    rule: [".*"]
    url: "http://www.gstatic.com/generate_204"
    interval: 300
    tolerance: 150
  - name: "Fallback"
    type: "fallback"
    rule: [".*"]
    url: "http://www.gstatic.com/generate_204"
    interval: 300
        "#;

        // Update settings with YAML content containing proxy groups
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify proxy-related settings
        assert_eq!(settings.enable_sort, true);
        assert_eq!(settings.clash_use_new_field, true);
        assert_eq!(settings.singbox_add_clash_modes, true);
        assert_eq!(settings.clash_proxies_style, "flow");
        assert_eq!(settings.clash_proxy_groups_style, "compact");

        // Verify custom proxy groups
        assert_eq!(settings.custom_proxy_groups.len(), 3);

        assert_eq!(settings.custom_proxy_groups[0].name, "Proxy");
        assert_eq!(
            settings.custom_proxy_groups[0].group_type,
            ProxyGroupType::Select
        );
        assert_eq!(settings.custom_proxy_groups[0].proxies, vec![".*"]);

        assert_eq!(settings.custom_proxy_groups[1].name, "Auto");
        assert_eq!(
            settings.custom_proxy_groups[1].group_type,
            ProxyGroupType::URLTest
        );
        assert_eq!(settings.custom_proxy_groups[1].proxies, vec![".*"]);
        assert_eq!(
            settings.custom_proxy_groups[1].url,
            "http://www.gstatic.com/generate_204"
        );
        assert_eq!(settings.custom_proxy_groups[1].interval, 300);
        assert_eq!(settings.custom_proxy_groups[1].tolerance, 150);
    }

    #[test]
    fn test_proxy_groups_toml() {
        let toml_content = r#"
[common]
enable_sort = true
clash_use_new_field = true
singbox_add_clash_modes = true
clash_proxies_style = "flow"
clash_proxy_groups_style = "compact"

[[custom_proxy_groups]]
name = "Proxy"
type = "select"
rule = [".*"]

[[custom_proxy_groups]]
name = "Auto"
type = "url-test"
rule = [".*"]
url = "http://www.gstatic.com/generate_204"
interval = 300
tolerance = 150

[[custom_proxy_groups]]
name = "Fallback"
type = "fallback"
rule = [".*"]
url = "http://www.gstatic.com/generate_204"
interval = 300
        "#;

        // Update settings with TOML content containing proxy groups
        update_settings_from_content(toml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify proxy-related settings
        assert_eq!(settings.enable_sort, true);
        assert_eq!(settings.clash_use_new_field, true);
        assert_eq!(settings.singbox_add_clash_modes, true);
        assert_eq!(settings.clash_proxies_style, "flow");
        assert_eq!(settings.clash_proxy_groups_style, "compact");

        // Verify custom proxy groups
        assert_eq!(settings.custom_proxy_groups.len(), 3);

        assert_eq!(settings.custom_proxy_groups[0].name, "Proxy");
        assert_eq!(
            settings.custom_proxy_groups[0].group_type,
            ProxyGroupType::Select
        );
        assert_eq!(settings.custom_proxy_groups[0].proxies, vec![".*"]);

        assert_eq!(settings.custom_proxy_groups[1].name, "Auto");
        assert_eq!(
            settings.custom_proxy_groups[1].group_type,
            ProxyGroupType::URLTest
        );
        assert_eq!(settings.custom_proxy_groups[1].proxies, vec![".*"]);
        assert_eq!(
            settings.custom_proxy_groups[1].url,
            "http://www.gstatic.com/generate_204"
        );
        assert_eq!(settings.custom_proxy_groups[1].interval, 300);
        assert_eq!(settings.custom_proxy_groups[1].tolerance, 150);
    }

    #[test]
    fn test_proxy_groups_ini() {
        let ini_content = r#"
[common]
enable_sort=true
clash_use_new_field=true
singbox_add_clash_modes=true
clash_proxies_style=flow
clash_proxy_groups_style=compact

[proxy_group]
custom_proxy_group=Proxy`select`.*
custom_proxy_group=Auto`url-test`.*`http://www.gstatic.com/generate_204`300`150
custom_proxy_group=Fallback`fallback`.*`http://www.gstatic.com/generate_204`300
        "#;

        // Update settings with INI content containing proxy groups
        update_settings_from_content(ini_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify proxy-related settings
        assert_eq!(settings.enable_sort, true);
        assert_eq!(settings.clash_use_new_field, true);
        assert_eq!(settings.singbox_add_clash_modes, true);
        assert_eq!(settings.clash_proxies_style, "flow");
        assert_eq!(settings.clash_proxy_groups_style, "compact");

        // Verify custom proxy groups
        assert_eq!(settings.custom_proxy_groups.len(), 3);

        assert_eq!(settings.custom_proxy_groups[0].name, "Proxy");
        assert_eq!(
            settings.custom_proxy_groups[0].group_type,
            ProxyGroupType::Select
        );
        assert_eq!(settings.custom_proxy_groups[0].proxies, vec![".*"]);

        assert_eq!(settings.custom_proxy_groups[1].name, "Auto");
        assert_eq!(
            settings.custom_proxy_groups[1].group_type,
            ProxyGroupType::URLTest
        );
        assert_eq!(settings.custom_proxy_groups[1].proxies, vec![".*"]);
        assert_eq!(
            settings.custom_proxy_groups[1].url,
            "http://www.gstatic.com/generate_204"
        );
        assert_eq!(settings.custom_proxy_groups[1].interval, 300);
        assert_eq!(settings.custom_proxy_groups[1].tolerance, 150);
    }

    #[test]
    fn test_proxy_flags() {
        let yaml_content = r#"
common:
  udp_flag: true
  tfo_flag: false
  skip_cert_verify: true
  tls13_flag: false
        "#;

        // Update settings with YAML content containing proxy flags
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify proxy flags
        assert_eq!(settings.udp_flag, Some(true));
        assert_eq!(settings.tfo_flag, Some(false));
        assert_eq!(settings.skip_cert_verify, Some(true));
        assert_eq!(settings.tls13_flag, Some(false));
    }
}
