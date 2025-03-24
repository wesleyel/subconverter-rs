use std::sync::Arc;
use subconverter::settings::settings::settings_struct::Settings;
use subconverter::settings::settings::update_settings_from_content;

#[cfg(test)]
mod settings_server_cache_tests {
    use super::*;

    #[test]
    fn test_server_settings_yaml() {
        let yaml_content = r#"
common:
  listen_address: "0.0.0.0"
  listen_port: 9090
  api_mode: true
  api_access_token: "some-secret-token"
  max_pending_conns: 20480
  max_concur_threads: 8
  serve_file: true
  serve_file_root: "./www"
  managed_config_prefix: "https://example.com/sub"
        "#;

        // Update settings with YAML content containing server settings
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify server settings
        assert_eq!(settings.listen_address, "0.0.0.0");
        assert_eq!(settings.listen_port, 9090);
        assert_eq!(settings.api_mode, true);
        assert_eq!(settings.api_access_token, "some-secret-token");
        assert_eq!(settings.max_pending_conns, 20480);
        assert_eq!(settings.max_concur_threads, 8);
        assert_eq!(settings.serve_file, true);
        assert_eq!(settings.serve_file_root, "./www");
        assert_eq!(settings.managed_config_prefix, "https://example.com/sub");
    }

    #[test]
    fn test_server_settings_toml() {
        let toml_content = r#"
[common]
listen_address = "127.0.0.1"
listen_port = 8080
api_mode = false
api_access_token = ""
max_pending_conns = 10240
max_concur_threads = 4
serve_file = false
serve_file_root = ""
managed_config_prefix = "http://localhost:8080/sub"
        "#;

        // Update settings with TOML content containing server settings
        update_settings_from_content(toml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify server settings
        assert_eq!(settings.listen_address, "127.0.0.1");
        assert_eq!(settings.listen_port, 8080);
        assert_eq!(settings.api_mode, false);
        assert_eq!(settings.api_access_token, "");
        assert_eq!(settings.max_pending_conns, 10240);
        assert_eq!(settings.max_concur_threads, 4);
        assert_eq!(settings.serve_file, false);
        assert_eq!(settings.serve_file_root, "");
        assert_eq!(settings.managed_config_prefix, "http://localhost:8080/sub");
    }

    #[test]
    fn test_server_settings_ini() {
        let ini_content = r#"
[common]
listen_address=192.168.1.1
listen_port=7070
api_mode=true
api_access_token=api-token-123
max_pending_conns=5120
max_concur_threads=2
serve_file=true
serve_file_root=./static
managed_config_prefix=http://192.168.1.1:7070/getconf
        "#;

        // Update settings with INI content containing server settings
        update_settings_from_content(ini_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify server settings
        assert_eq!(settings.listen_address, "192.168.1.1");
        assert_eq!(settings.listen_port, 7070);
        assert_eq!(settings.api_mode, true);
        assert_eq!(settings.api_access_token, "api-token-123");
        assert_eq!(settings.max_pending_conns, 5120);
        assert_eq!(settings.max_concur_threads, 2);
        assert_eq!(settings.serve_file, true);
        assert_eq!(settings.serve_file_root, "./static");
        assert_eq!(
            settings.managed_config_prefix,
            "http://192.168.1.1:7070/getconf"
        );
    }

    #[test]
    fn test_cache_settings_yaml() {
        let yaml_content = r#"
common:
  serve_cache_on_fetch_fail: true
  cache_subscription: 120
  cache_config: 600
  cache_ruleset: 43200
  max_allowed_download_size: 67108864
        "#;

        // Update settings with YAML content containing cache settings
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify cache settings
        assert_eq!(settings.serve_cache_on_fetch_fail, true);
        assert_eq!(settings.cache_subscription, 120);
        assert_eq!(settings.cache_config, 600);
        assert_eq!(settings.cache_ruleset, 43200);
        assert_eq!(settings.max_allowed_download_size, 67108864);
    }

    #[test]
    fn test_cache_settings_toml() {
        let toml_content = r#"
[common]
serve_cache_on_fetch_fail = false
cache_subscription = 60
cache_config = 300
cache_ruleset = 21600
max_allowed_download_size = 33554432
        "#;

        // Update settings with TOML content containing cache settings
        update_settings_from_content(toml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify cache settings
        assert_eq!(settings.serve_cache_on_fetch_fail, false);
        assert_eq!(settings.cache_subscription, 60);
        assert_eq!(settings.cache_config, 300);
        assert_eq!(settings.cache_ruleset, 21600);
        assert_eq!(settings.max_allowed_download_size, 33554432);
    }

    #[test]
    fn test_cache_settings_ini() {
        let ini_content = r#"
[common]
serve_cache_on_fetch_fail=true
cache_subscription=30
cache_config=150
cache_ruleset=10800
max_allowed_download_size=16777216
        "#;

        // Update settings with INI content containing cache settings
        update_settings_from_content(ini_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify cache settings
        assert_eq!(settings.serve_cache_on_fetch_fail, true);
        assert_eq!(settings.cache_subscription, 30);
        assert_eq!(settings.cache_config, 150);
        assert_eq!(settings.cache_ruleset, 10800);
        assert_eq!(settings.max_allowed_download_size, 16777216);
    }

    #[test]
    fn test_base_config_settings() {
        let yaml_content = r#"
common:
  clash_base: "./profiles/clash.yml"
  surge_base: "./profiles/surge.conf"
  surfboard_base: "./profiles/surfboard.conf"
  mellow_base: "./profiles/mellow.conf"
  quan_base: "./profiles/quan.conf"
  quanx_base: "./profiles/quanx.conf"
  loon_base: "./profiles/loon.conf"
  ssub_base: "./profiles/ssub.json"
  singbox_base: "./profiles/singbox.json"
  surge_ssr_path: "./profiles/surge.ssr"
  quanx_dev_id: "device-id-12345"
        "#;

        // Update settings with YAML content containing base config settings
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify base config settings
        assert_eq!(settings.clash_base, "./profiles/clash.yml");
        assert_eq!(settings.surge_base, "./profiles/surge.conf");
        assert_eq!(settings.surfboard_base, "./profiles/surfboard.conf");
        assert_eq!(settings.mellow_base, "./profiles/mellow.conf");
        assert_eq!(settings.quan_base, "./profiles/quan.conf");
        assert_eq!(settings.quanx_base, "./profiles/quanx.conf");
        assert_eq!(settings.loon_base, "./profiles/loon.conf");
        assert_eq!(settings.ssub_base, "./profiles/ssub.json");
        assert_eq!(settings.singbox_base, "./profiles/singbox.json");
        assert_eq!(settings.surge_ssr_path, "./profiles/surge.ssr");
        assert_eq!(settings.quanx_dev_id, "device-id-12345");
    }
}
