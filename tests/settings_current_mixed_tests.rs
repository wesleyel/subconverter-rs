use std::sync::Arc;
use subconverter::settings::settings::settings_struct::Settings;
use subconverter::settings::settings::update_settings_from_content;

#[cfg(test)]
mod settings_current_mixed_tests {
    use subconverter::models::ProxyGroupType;

    use super::*;

    #[test]
    fn test_settings_current() {
        // Test that Settings::current() returns a valid Arc<Settings>
        let settings = Settings::current();

        // Ensure we have valid settings with expected defaults
        assert_eq!(settings.listen_address, "127.0.0.1");
        assert_eq!(settings.listen_port, 25500);
        assert_eq!(settings.max_pending_conns, 10240);
        assert_eq!(settings.max_concur_threads, 4);
    }

    #[test]
    fn test_settings_update_and_current() {
        // First, use a simple settings content
        let yaml_content = r#"
common:
  listen_address: "0.0.0.0"
  listen_port: 9090
        "#;

        update_settings_from_content(yaml_content).unwrap();

        // Get current settings and verify
        let settings = Settings::current();
        assert_eq!(settings.listen_address, "0.0.0.0");
        assert_eq!(settings.listen_port, 9090);

        // Now update with new settings
        let toml_content = r#"
[common]
listen_address = "127.0.0.1"
listen_port = 8080
        "#;

        update_settings_from_content(toml_content).unwrap();

        // Get current settings again and verify they were updated
        let updated_settings = Settings::current();
        assert_eq!(updated_settings.listen_address, "127.0.0.1");
        assert_eq!(updated_settings.listen_port, 8080);
    }

    #[test]
    fn test_mixed_settings_yaml() {
        let yaml_content = r#"
common:
  add_emoji: true
  append_type: true
  api_mode: true
  log_level: 2
  listen_address: "0.0.0.0"
  listen_port: 9090
  max_pending_conns: 20480
  max_concur_threads: 8

emojis:
  - match: "(?i)港"
    emoji: "🇭🇰"
  - match: "(?i)台"
    emoji: "🇹🇼"

renames:
  - match: "(?i)HK"
    rename: "香港"
  - match: "(?i)TW"
    rename: "台湾"

custom_proxy_groups:
  - name: "Proxy"
    type: "select"
    rule: [".*"]
  - name: "Auto"
    type: "url-test"
    rule: [".*"]
    url: "http://www.gstatic.com/generate_204"
    interval: 300

custom_rulesets:
  - name: "Streaming"
    url: "https://example.com/streaming.list"
    type: "surge-ruleset"
  - name: "Direct"
    url: "https://example.com/direct.list"
    type: "surge-ruleset"

cron_tasks:
  - name: "update"
    expression: "0 0 * * *"
    path: "/scripts/update.sh"
    enable: true
        "#;

        // Update settings with mixed YAML content
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify mixed settings
        assert_eq!(settings.add_emoji, true);
        assert_eq!(settings.append_type, true);
        assert_eq!(settings.api_mode, true);
        assert_eq!(settings.log_level, 2);
        assert_eq!(settings.listen_address, "0.0.0.0");
        assert_eq!(settings.listen_port, 9090);
        assert_eq!(settings.max_pending_conns, 20480);
        assert_eq!(settings.max_concur_threads, 8);

        // Verify emojis
        assert_eq!(settings.emojis.len(), 2);
        assert_eq!(settings.emojis[0]._match, "(?i)港");
        assert_eq!(settings.emojis[0].replace, "🇭🇰");
        assert_eq!(settings.emojis[1]._match, "(?i)台");
        assert_eq!(settings.emojis[1].replace, "🇹🇼");

        // Verify renames
        assert_eq!(settings.renames.len(), 2);
        assert_eq!(settings.renames[0]._match, "(?i)HK");
        assert_eq!(settings.renames[0].replace, "香港");
        assert_eq!(settings.renames[1]._match, "(?i)TW");
        assert_eq!(settings.renames[1].replace, "台湾");

        // Verify proxy groups
        assert_eq!(settings.custom_proxy_groups.len(), 2);
        assert_eq!(settings.custom_proxy_groups[0].name, "Proxy");
        assert_eq!(
            settings.custom_proxy_groups[0].group_type,
            ProxyGroupType::Select
        );
        assert_eq!(settings.custom_proxy_groups[1].name, "Auto");
        assert_eq!(
            settings.custom_proxy_groups[1].group_type,
            ProxyGroupType::URLTest
        );
        assert_eq!(settings.custom_proxy_groups[1].interval, 300);

        // Verify rulesets
        assert_eq!(settings.custom_rulesets.len(), 2);
        assert_eq!(settings.custom_rulesets[0].group, "Streaming");
        assert_eq!(
            settings.custom_rulesets[0].url,
            "https://example.com/streaming.list"
        );
        assert_eq!(settings.custom_rulesets[1].group, "Direct");
        assert_eq!(
            settings.custom_rulesets[1].url,
            "https://example.com/direct.list"
        );

        // Verify cron tasks
        assert_eq!(settings.cron_tasks.len(), 1);
        assert_eq!(settings.cron_tasks[0].name, "update");
        assert_eq!(settings.cron_tasks[0].cron_exp, "0 0 * * *");
        assert_eq!(settings.cron_tasks[0].path, "/scripts/update.sh");
    }

    #[test]
    fn test_mixed_settings_toml() {
        let toml_content = r#"
[common]
add_emoji = true
append_type = true
api_mode = true
log_level = 2
listen_address = "127.0.0.1"
listen_port = 8080
max_pending_conns = 10240
max_concur_threads = 4
enable_rule_gen = true
update_ruleset_on_request = true
write_managed_config = true

[[emojis]]
match = "(?i)港"
emoji = "🇭🇰"

[[emojis]]
match = "(?i)台"
emoji = "🇹🇼"

[[renames]]
match = "(?i)HK"
rename = "香港"

[[renames]]
match = "(?i)TW"
rename = "台湾"

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

[[custom_rulesets]]
name = "Streaming"
url = "https://example.com/streaming.list"
type = "surge-ruleset"

[[custom_rulesets]]
name = "Direct"
url = "https://example.com/direct.list"
type = "surge-ruleset"

[aliases]
v2ray = "vmess"
ss = "shadowsocks"
        "#;

        // Update settings with mixed TOML content
        update_settings_from_content(toml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify mixed settings
        assert_eq!(settings.add_emoji, true);
        assert_eq!(settings.append_type, true);
        assert_eq!(settings.api_mode, true);
        assert_eq!(settings.log_level, 2);
        assert_eq!(settings.listen_address, "127.0.0.1");
        assert_eq!(settings.listen_port, 8080);
        assert_eq!(settings.max_pending_conns, 10240);
        assert_eq!(settings.max_concur_threads, 4);
        assert_eq!(settings.enable_rule_gen, true);
        assert_eq!(settings.update_ruleset_on_request, true);
        assert_eq!(settings.write_managed_config, true);

        // Verify emojis
        assert_eq!(settings.emojis.len(), 2);
        assert_eq!(settings.emojis[0]._match, "(?i)港");
        assert_eq!(settings.emojis[0].replace, "🇭🇰");
        assert_eq!(settings.emojis[1]._match, "(?i)台");
        assert_eq!(settings.emojis[1].replace, "🇹🇼");

        // Verify renames
        assert_eq!(settings.renames.len(), 2);
        assert_eq!(settings.renames[0]._match, "(?i)HK");
        assert_eq!(settings.renames[0].replace, "香港");
        assert_eq!(settings.renames[1]._match, "(?i)TW");
        assert_eq!(settings.renames[1].replace, "台湾");

        // Verify proxy groups
        assert_eq!(settings.custom_proxy_groups.len(), 2);
        assert_eq!(settings.custom_proxy_groups[0].name, "Proxy");
        assert_eq!(
            settings.custom_proxy_groups[0].group_type,
            ProxyGroupType::Select
        );
        assert_eq!(settings.custom_proxy_groups[1].name, "Auto");
        assert_eq!(
            settings.custom_proxy_groups[1].group_type,
            ProxyGroupType::URLTest
        );
        assert_eq!(settings.custom_proxy_groups[1].interval, 300);

        // Verify rulesets
        assert_eq!(settings.custom_rulesets.len(), 2);
        assert_eq!(settings.custom_rulesets[0].group, "Streaming");
        assert_eq!(
            settings.custom_rulesets[0].url,
            "https://example.com/streaming.list"
        );
        assert_eq!(settings.custom_rulesets[1].group, "Direct");
        assert_eq!(
            settings.custom_rulesets[1].url,
            "https://example.com/direct.list"
        );

        // Verify aliases
        assert_eq!(settings.aliases.len(), 2);
        assert_eq!(settings.aliases.get("v2ray"), Some(&"vmess".to_string()));
        assert_eq!(settings.aliases.get("ss"), Some(&"shadowsocks".to_string()));
    }

    #[test]
    fn test_mixed_settings_ini() {
        let ini_content = r#"
[common]
add_emoji=true
append_type=true
api_mode=true
log_level=2
listen_address=192.168.1.1
listen_port=7070
max_pending_conns=5120
max_concur_threads=2
enable_rule_gen=true
update_ruleset_on_request=true
write_managed_config=true

[emojis]
add=(?i)港,🇭🇰
add=(?i)台,🇹🇼

[rename]
rename=(?i)HK,香港
rename=(?i)TW,台湾

[proxy_group]
custom_proxy_group=Proxy`select`.*
custom_proxy_group=Auto`url-test`.*`http://www.gstatic.com/generate_204`300

[ruleset]
ruleset=Streaming,https://example.com/streaming.list,surge-ruleset
ruleset=Direct,https://example.com/direct.list,surge-ruleset

[aliases]
v2ray=vmess
ss=shadowsocks
        "#;

        // Update settings with mixed INI content
        update_settings_from_content(ini_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify mixed settings
        assert_eq!(settings.add_emoji, true);
        assert_eq!(settings.append_type, true);
        assert_eq!(settings.api_mode, true);
        assert_eq!(settings.log_level, 2);
        assert_eq!(settings.listen_address, "192.168.1.1");
        assert_eq!(settings.listen_port, 7070);
        assert_eq!(settings.max_pending_conns, 5120);
        assert_eq!(settings.max_concur_threads, 2);
        assert_eq!(settings.enable_rule_gen, true);
        assert_eq!(settings.update_ruleset_on_request, true);
        assert_eq!(settings.write_managed_config, true);

        // Verify emojis
        assert_eq!(settings.emojis.len(), 2);
        assert_eq!(settings.emojis[0]._match, "(?i)港");
        assert_eq!(settings.emojis[0].replace, "🇭🇰");
        assert_eq!(settings.emojis[1]._match, "(?i)台");
        assert_eq!(settings.emojis[1].replace, "🇹🇼");

        // Verify renames
        assert_eq!(settings.renames.len(), 2);
        assert_eq!(settings.renames[0]._match, "(?i)HK");
        assert_eq!(settings.renames[0].replace, "香港");
        assert_eq!(settings.renames[1]._match, "(?i)TW");
        assert_eq!(settings.renames[1].replace, "台湾");

        // Verify proxy groups
        assert_eq!(settings.custom_proxy_groups.len(), 2);
        assert_eq!(settings.custom_proxy_groups[0].name, "Proxy");
        assert_eq!(
            settings.custom_proxy_groups[0].group_type,
            ProxyGroupType::Select
        );
        assert_eq!(settings.custom_proxy_groups[1].name, "Auto");
        assert_eq!(
            settings.custom_proxy_groups[1].group_type,
            ProxyGroupType::URLTest
        );
        assert_eq!(settings.custom_proxy_groups[1].interval, 300);

        // Verify rulesets
        assert_eq!(settings.custom_rulesets.len(), 2);
        assert_eq!(settings.custom_rulesets[0].group, "Streaming");
        assert_eq!(
            settings.custom_rulesets[0].url,
            "https://example.com/streaming.list"
        );
        assert_eq!(settings.custom_rulesets[1].group, "Direct");
        assert_eq!(
            settings.custom_rulesets[1].url,
            "https://example.com/direct.list"
        );

        // Verify aliases
        assert_eq!(settings.aliases.len(), 2);
        assert_eq!(settings.aliases.get("v2ray"), Some(&"vmess".to_string()));
        assert_eq!(settings.aliases.get("ss"), Some(&"shadowsocks".to_string()));
    }
}
