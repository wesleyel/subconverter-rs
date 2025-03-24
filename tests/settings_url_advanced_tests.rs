use std::sync::Arc;
use subconverter::settings::settings::settings_struct::Settings;
use subconverter::settings::settings::update_settings_from_content;

#[cfg(test)]
mod settings_url_advanced_tests {
    use subconverter::constants::log_level::LOG_LEVEL_VERBOSE;

    use super::*;

    #[test]
    fn test_url_settings_yaml() {
        let yaml_content = r#"
common:
  default_ext_config: "https://example.com/config/config.ini"
  enable_insert: true
  prepend_insert: true
  default_urls: 
    - "https://example.com/subscription1"
    - "https://example.com/subscription2"
    - "https://example.com/subscription3"
  insert_urls:
    - "https://example.com/insert1"
    - "https://example.com/insert2"
  proxy_config: "http://127.0.0.1:1080"
  proxy_ruleset: "http://127.0.0.1:1080"
  proxy_subscription: "http://127.0.0.1:1080"
        "#;

        // Update settings with YAML content containing URL settings
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify URL settings
        assert_eq!(
            settings.default_ext_config,
            "https://example.com/config/config.ini"
        );
        assert_eq!(settings.enable_insert, true);
        assert_eq!(settings.prepend_insert, true);

        assert_eq!(settings.default_urls.len(), 3);
        assert_eq!(
            settings.default_urls[0],
            "https://example.com/subscription1"
        );
        assert_eq!(
            settings.default_urls[1],
            "https://example.com/subscription2"
        );
        assert_eq!(
            settings.default_urls[2],
            "https://example.com/subscription3"
        );

        assert_eq!(settings.insert_urls.len(), 2);
        assert_eq!(settings.insert_urls[0], "https://example.com/insert1");
        assert_eq!(settings.insert_urls[1], "https://example.com/insert2");

        assert_eq!(settings.proxy_config, "http://127.0.0.1:1080");
        assert_eq!(settings.proxy_ruleset, "http://127.0.0.1:1080");
        assert_eq!(settings.proxy_subscription, "http://127.0.0.1:1080");
    }

    #[test]
    fn test_url_settings_toml() {
        let toml_content = r#"
[common]
default_ext_config = "https://example.com/config/config.toml"
enable_insert = false
prepend_insert = false
default_urls = [
  "https://example.com/sub1",
  "https://example.com/sub2"
]
insert_urls = [
  "https://example.com/insert3"
]
proxy_config = "socks5://127.0.0.1:1086"
proxy_ruleset = "socks5://127.0.0.1:1086"
proxy_subscription = "socks5://127.0.0.1:1086"
        "#;

        // Update settings with TOML content containing URL settings
        update_settings_from_content(toml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify URL settings
        assert_eq!(
            settings.default_ext_config,
            "https://example.com/config/config.toml"
        );
        assert_eq!(settings.enable_insert, false);
        assert_eq!(settings.prepend_insert, false);

        assert_eq!(settings.default_urls.len(), 2);
        assert_eq!(settings.default_urls[0], "https://example.com/sub1");
        assert_eq!(settings.default_urls[1], "https://example.com/sub2");

        assert_eq!(settings.insert_urls.len(), 1);
        assert_eq!(settings.insert_urls[0], "https://example.com/insert3");

        assert_eq!(settings.proxy_config, "socks5://127.0.0.1:1086");
        assert_eq!(settings.proxy_ruleset, "socks5://127.0.0.1:1086");
        assert_eq!(settings.proxy_subscription, "socks5://127.0.0.1:1086");
    }

    #[test]
    fn test_url_settings_ini() {
        let ini_content = r#"
[common]
default_ext_config=https://example.com/config/config.ini
enable_insert=true
prepend_insert=false

[node]
default_url=https://example.com/node1
default_url=https://example.com/node2
insert_url=https://example.com/insert_node

[advanced]
proxy_config=http://localhost:8888
proxy_ruleset=http://localhost:8888
proxy_subscription=http://localhost:8888
        "#;

        // Update settings with INI content containing URL settings
        update_settings_from_content(ini_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify URL settings
        assert_eq!(
            settings.default_ext_config,
            "https://example.com/config/config.ini"
        );
        assert_eq!(settings.enable_insert, true);
        assert_eq!(settings.prepend_insert, false);

        assert_eq!(settings.default_urls.len(), 2);
        assert_eq!(settings.default_urls[0], "https://example.com/node1");
        assert_eq!(settings.default_urls[1], "https://example.com/node2");

        assert_eq!(settings.insert_urls.len(), 1);
        assert_eq!(settings.insert_urls[0], "https://example.com/insert_node");

        assert_eq!(settings.proxy_config, "http://localhost:8888");
        assert_eq!(settings.proxy_ruleset, "http://localhost:8888");
        assert_eq!(settings.proxy_subscription, "http://localhost:8888");
    }

    #[test]
    fn test_template_settings() {
        let yaml_content = r#"
common:
  template_path: "./templates"
  template_vars:
    clash_dns_port: "5353"
    clash_api_port: "9090"
    clash_ui_port: "8080"
    singbox_direct_domain: "example.com"
    singbox_proxy_domain: "google.com"
        "#;

        // Update settings with YAML content containing template settings
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify template settings
        assert_eq!(settings.template_path, "./templates");

        assert_eq!(settings.template_vars.len(), 5);
        assert_eq!(
            settings.template_vars.get("clash_dns_port"),
            Some(&"5353".to_string())
        );
        assert_eq!(
            settings.template_vars.get("clash_api_port"),
            Some(&"9090".to_string())
        );
        assert_eq!(
            settings.template_vars.get("clash_ui_port"),
            Some(&"8080".to_string())
        );
        assert_eq!(
            settings.template_vars.get("singbox_direct_domain"),
            Some(&"example.com".to_string())
        );
        assert_eq!(
            settings.template_vars.get("singbox_proxy_domain"),
            Some(&"google.com".to_string())
        );
    }

    #[test]
    fn test_generator_settings() {
        let yaml_content = r#"
common:
  generator_mode: true
  generate_profiles: "all"
  write_managed_config: true
  filter_deprecated: true
  update_interval: 86400
  sort_script: "function compare(_, __) return true end"
  filter_script: "function filter(t) return true end"
        "#;

        // Update settings with YAML content containing generator settings
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify generator settings
        assert_eq!(settings.generator_mode, true);
        assert_eq!(settings.generate_profiles, "all");
        assert_eq!(settings.write_managed_config, true);
        assert_eq!(settings.filter_deprecated, true);
        assert_eq!(settings.update_interval, 86400);
        assert_eq!(
            settings.sort_script,
            "function compare(_, __) return true end"
        );
        assert_eq!(settings.filter_script, "function filter(t) return true end");
    }

    #[test]
    fn test_script_limits() {
        let yaml_content = r#"
common:
  script_clean_context: true
  log_level: 2
  print_dbg_info: true
        "#;

        // Update settings with YAML content containing script and logging settings
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify script and logging settings
        assert_eq!(settings.script_clean_context, true);
        assert_eq!(settings.log_level, LOG_LEVEL_VERBOSE);
    }

    #[test]
    fn test_cron_settings() {
        let yaml_content = r#"
common:
  enable_cron: true

cron_tasks:
  - name: "update_certs"
    expression: "0 0 * * *" 
    path: "/scripts/update_certs.sh"
    enable: true
  - name: "backup_config"
    expression: "0 */12 * * *"
    path: "/scripts/backup.sh"
    enable: true
        "#;

        // Update settings with YAML content containing cron settings
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify cron settings
        assert_eq!(settings.enable_cron, true);

        assert_eq!(settings.cron_tasks.len(), 2);
        assert_eq!(settings.cron_tasks[0].name, "update_certs");
        assert_eq!(settings.cron_tasks[0].cron_exp, "0 0 * * *");
        assert_eq!(settings.cron_tasks[0].path, "/scripts/update_certs.sh");

        assert_eq!(settings.cron_tasks[1].name, "backup_config");
        assert_eq!(settings.cron_tasks[1].cron_exp, "0 */12 * * *");
        assert_eq!(settings.cron_tasks[1].path, "/scripts/backup.sh");
    }
}
