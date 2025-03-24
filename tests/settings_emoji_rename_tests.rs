use std::sync::Arc;
use subconverter::settings::settings::settings_struct::Settings;
use subconverter::settings::settings::update_settings_from_content;

#[cfg(test)]
mod settings_emoji_rename_tests {
    use super::*;

    #[test]
    fn test_emoji_yaml() {
        let yaml_content = r#"
common:
  add_emoji: true
  remove_emoji: false

emojis:
  - match: "(?i)港"
    emoji: "🇭🇰"
  - match: "(?i)日本|东京|大阪|JP"
    emoji: "🇯🇵"
  - match: "(?i)美|洛杉矶|硅谷|达拉斯|费利蒙|凤凰城|芝加哥|圣何塞|西雅图|弗里蒙特|US"
    emoji: "🇺🇸"
  - match: "(?i)新加坡|狮城|SG"
    emoji: "🇸🇬"
        "#;

        // Update settings with YAML content containing emoji rules
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify emoji settings
        assert_eq!(settings.add_emoji, true);
        assert_eq!(settings.remove_emoji, false);

        // Verify emoji rules
        assert_eq!(settings.emojis.len(), 4);

        assert_eq!(settings.emojis[0]._match, "(?i)港");
        assert_eq!(settings.emojis[0].replace, "🇭🇰");

        assert_eq!(settings.emojis[1]._match, "(?i)日本|东京|大阪|JP");
        assert_eq!(settings.emojis[1].replace, "🇯🇵");

        assert_eq!(
            settings.emojis[2]._match,
            "(?i)美|洛杉矶|硅谷|达拉斯|费利蒙|凤凰城|芝加哥|圣何塞|西雅图|弗里蒙特|US"
        );
        assert_eq!(settings.emojis[2].replace, "🇺🇸");
    }

    #[test]
    fn test_emoji_toml() {
        let toml_content = r#"
[common]
add_emoji = true
remove_emoji = false

[[emojis]]
match = "(?i)港"
emoji = "🇭🇰"

[[emojis]]
match = "(?i)日本|东京|大阪|JP"
emoji = "🇯🇵"

[[emojis]]
match = "(?i)美|洛杉矶|硅谷|达拉斯|费利蒙|凤凰城|芝加哥|圣何塞|西雅图|弗里蒙特|US"
emoji = "🇺🇸"

[[emojis]]
match = "(?i)新加坡|狮城|SG"
emoji = "🇸🇬"
        "#;

        // Update settings with TOML content containing emoji rules
        update_settings_from_content(toml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify emoji settings
        assert_eq!(settings.add_emoji, true);
        assert_eq!(settings.remove_emoji, false);

        // Verify emoji rules
        assert_eq!(settings.emojis.len(), 4);

        assert_eq!(settings.emojis[0]._match, "(?i)港");
        assert_eq!(settings.emojis[0].replace, "🇭🇰");

        assert_eq!(settings.emojis[1]._match, "(?i)日本|东京|大阪|JP");
        assert_eq!(settings.emojis[1].replace, "🇯🇵");

        assert_eq!(
            settings.emojis[2]._match,
            "(?i)美|洛杉矶|硅谷|达拉斯|费利蒙|凤凰城|芝加哥|圣何塞|西雅图|弗里蒙特|US"
        );
        assert_eq!(settings.emojis[2].replace, "🇺🇸");
    }

    #[test]
    fn test_emoji_ini() {
        let ini_content = r#"
[common]
add_emoji=true
remove_emoji=false

[emojis]
add=(?i)港,🇭🇰
add=(?i)日本|东京|大阪|JP,🇯🇵
add=(?i)美|洛杉矶|硅谷|达拉斯|费利蒙|凤凰城|芝加哥|圣何塞|西雅图|弗里蒙特|US,🇺🇸
add=(?i)新加坡|狮城|SG,🇸🇬
        "#;

        // Update settings with INI content containing emoji rules
        update_settings_from_content(ini_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify emoji settings
        assert_eq!(settings.add_emoji, true);
        assert_eq!(settings.remove_emoji, false);

        // Verify emoji rules
        assert_eq!(settings.emojis.len(), 4);

        assert_eq!(settings.emojis[0]._match, "(?i)港");
        assert_eq!(settings.emojis[0].replace, "🇭🇰");

        assert_eq!(settings.emojis[1]._match, "(?i)日本|东京|大阪|JP");
        assert_eq!(settings.emojis[1].replace, "🇯🇵");

        assert_eq!(
            settings.emojis[2]._match,
            "(?i)美|洛杉矶|硅谷|达拉斯|费利蒙|凤凰城|芝加哥|圣何塞|西雅图|弗里蒙特|US"
        );
        assert_eq!(settings.emojis[2].replace, "🇺🇸");
    }

    #[test]
    fn test_rename_yaml() {
        let yaml_content = r#"
common:
  append_type: true

renames:
  - match: "(?i)流量|时间|应急|过期|Bandwidth|expire"
    rename: "[流量]"
  - match: "(?i)回国|China|CN|CHN"
    rename: "[回国]"
  - match: "(?i)香港|HK|Hong Kong"
    rename: "香港"
  - match: "(?i)台湾|TW|Taiwan"
    rename: "台湾"
        "#;

        // Update settings with YAML content containing rename rules
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify rename settings
        assert_eq!(settings.append_type, true);

        // Verify rename rules
        assert_eq!(settings.renames.len(), 4);

        assert_eq!(
            settings.renames[0]._match,
            "(?i)流量|时间|应急|过期|Bandwidth|expire"
        );
        assert_eq!(settings.renames[0].replace, "[流量]");

        assert_eq!(settings.renames[1]._match, "(?i)回国|China|CN|CHN");
        assert_eq!(settings.renames[1].replace, "[回国]");

        assert_eq!(settings.renames[2]._match, "(?i)香港|HK|Hong Kong");
        assert_eq!(settings.renames[2].replace, "香港");
    }

    #[test]
    fn test_rename_toml() {
        let toml_content = r#"
[common]
append_type = true

[[renames]]
match = "(?i)流量|时间|应急|过期|Bandwidth|expire"
rename = "[流量]"

[[renames]]
match = "(?i)回国|China|CN|CHN"
rename = "[回国]"

[[renames]]
match = "(?i)香港|HK|Hong Kong"
rename = "香港"

[[renames]]
match = "(?i)台湾|TW|Taiwan"
rename = "台湾"
        "#;

        // Update settings with TOML content containing rename rules
        update_settings_from_content(toml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify rename settings
        assert_eq!(settings.append_type, true);

        // Verify rename rules
        assert_eq!(settings.renames.len(), 4);

        assert_eq!(
            settings.renames[0]._match,
            "(?i)流量|时间|应急|过期|Bandwidth|expire"
        );
        assert_eq!(settings.renames[0].replace, "[流量]");

        assert_eq!(settings.renames[1]._match, "(?i)回国|China|CN|CHN");
        assert_eq!(settings.renames[1].replace, "[回国]");

        assert_eq!(settings.renames[2]._match, "(?i)香港|HK|Hong Kong");
        assert_eq!(settings.renames[2].replace, "香港");
    }

    #[test]
    fn test_rename_ini() {
        let ini_content = r#"
[common]
append_type=true

[rename]
rename=(?i)流量|时间|应急|过期|Bandwidth|expire,[流量]
rename=(?i)回国|China|CN|CHN,[回国]
rename=(?i)香港|HK|Hong Kong,香港
rename=(?i)台湾|TW|Taiwan,台湾
        "#;

        // Update settings with INI content containing rename rules
        update_settings_from_content(ini_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify rename settings
        assert_eq!(settings.append_type, true);

        // Verify rename rules
        assert_eq!(settings.renames.len(), 4);

        assert_eq!(
            settings.renames[0]._match,
            "(?i)流量|时间|应急|过期|Bandwidth|expire"
        );
        assert_eq!(settings.renames[0].replace, "[流量]");

        assert_eq!(settings.renames[1]._match, "(?i)回国|China|CN|CHN");
        assert_eq!(settings.renames[1].replace, "[回国]");

        assert_eq!(settings.renames[2]._match, "(?i)香港|HK|Hong Kong");
        assert_eq!(settings.renames[2].replace, "香港");
    }

    #[test]
    fn test_aliases_yaml() {
        let yaml_content = r#"
common:
  skip_failed_links: true

aliases:
  v2ray: vmess
  ss: shadowsocks
  trojan: trojan-gfw
  ssr: shadowsocksr
        "#;

        // Update settings with YAML content containing aliases
        update_settings_from_content(yaml_content).unwrap();

        // Get the current settings
        let settings = Settings::current();

        // Verify basic settings
        assert_eq!(settings.skip_failed_links, true);

        // Verify aliases
        assert_eq!(settings.aliases.len(), 4);
        assert_eq!(settings.aliases.get("v2ray"), Some(&"vmess".to_string()));
        assert_eq!(settings.aliases.get("ss"), Some(&"shadowsocks".to_string()));
        assert_eq!(
            settings.aliases.get("trojan"),
            Some(&"trojan-gfw".to_string())
        );
        assert_eq!(
            settings.aliases.get("ssr"),
            Some(&"shadowsocksr".to_string())
        );
    }
}
