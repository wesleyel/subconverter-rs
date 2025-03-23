use actix_web::{web, HttpResponse};
use log::{debug, error};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::interfaces::subconverter::{subconverter, SubconverterConfigBuilder};
use crate::models::{AppState, SubconverterTarget};
use crate::settings::external::ExternalConfig;
use crate::settings::refresh_configuration;
use crate::settings::unified::get_instance;
/// Query parameters for subscription conversion
#[derive(Deserialize, Debug, Default)]
pub struct SubconverterQuery {
    /// Target format
    pub target: Option<String>,
    /// URLs to convert (pipe separated)
    pub url: Option<String>,
    /// Insert URLs to append/prepend (pipe separated)
    pub insert: Option<String>,
    /// Whether to prepend insert nodes
    #[serde(default)]
    pub prepend: bool,
    /// Custom group name
    pub group: Option<String>,
    /// Custom filename for download
    pub filename: Option<String>,
    /// Base configuration file (optional)
    pub config: Option<String>,
    /// Include remarks regex
    pub include: Option<String>,
    /// Exclude remarks regex
    pub exclude: Option<String>,
    /// Surge version number
    pub ver: Option<u32>,
    /// Append proxy type to remarks
    pub append_type: Option<bool>,
    /// Whether to add emoji
    pub emoji: Option<bool>,
    /// List mode (node list only)
    pub list: Option<bool>,
    /// Sort nodes
    pub sort: Option<bool>,
    /// Information for filtering, rename, emoji addition
    pub rename: Option<String>,
    /// Whether to enable TCP Fast Open
    pub tfo: Option<bool>,
    /// Whether to enable UDP
    pub udp: Option<bool>,
    /// Whether to skip certificate verification
    pub scv: Option<bool>,
    /// Whether to enable TLS 1.3
    pub tls13: Option<bool>,
    /// Enable rule generator
    pub rename_node: Option<bool>,
    /// Update interval in seconds
    pub interval: Option<u32>,
    /// Update strict mode
    pub strict: Option<bool>,
    /// Upload to gist
    pub upload: Option<bool>,
    /// Authentication token
    pub token: Option<String>,
    /// Filter script
    pub filter: Option<String>,
    /// Device ID (for device-specific configurations)
    pub dev_id: Option<String>,
    /// Whether to use new field names in Clash
    pub new_name: Option<bool>,
}

/// Parse a query string into a HashMap
pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if let Some(key) = parts.next() {
            let value = parts.next().unwrap_or("");
            params.insert(key.to_string(), value.to_string());
        }
    }
    params
}

/// Handler for subscription conversion
pub async fn sub_handler(
    query: web::Query<SubconverterQuery>,
    app_state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    debug!("Received subconverter request: {:?}", query);

    // Check if we should reload the config
    if app_state.config.reload_conf_on_request && !app_state.config.api_mode {
        // Try to refresh the configuration
        if let Err(e) = refresh_configuration() {
            error!("Failed to reload configuration: {}", e);
            // Continue with the existing configuration
        }
    }

    // Process external config if provided
    let mut external_config = None;
    if let Some(conf_url) = &query.config {
        match ExternalConfig::load_from_file(conf_url) {
            Ok(config) => {
                external_config = Some(config);
            }
            Err(e) => {
                error!("Failed to load external config from {}: {}", conf_url, e);
                // Continue without external config
            }
        }
    }

    // Create merged settings from global and external configs
    let merged_settings = get_instance().create_merged(external_config);

    // Start building configuration
    let mut builder = SubconverterConfigBuilder::new();

    // Add extra settings from the merged configuration
    builder = builder.extra(merged_settings.extra.clone());

    // Process target parameter
    if let Some(target) = &query.target {
        builder = builder.target_from_str(target);

        // Process Surge version
        if let Some(ver) = query.ver {
            if let Ok(cfg) = builder.clone().build() {
                if matches!(cfg.target, SubconverterTarget::Surge(_)) {
                    builder = builder.surge_version(ver as i32);
                }
            }
        }
    }

    // Process URL parameter (mandatory)
    if let Some(url) = &query.url {
        builder = builder.urls_from_str(url);
    } else {
        return HttpResponse::BadRequest().body("Missing URL parameter");
    }

    // Process insert URLs if provided
    if let Some(insert) = &query.insert {
        builder = builder.insert_urls_from_str(insert);
        builder = builder.prepend_insert(query.prepend);
    }

    // Process group name
    if let Some(group) = &query.group {
        builder = builder.group_name(Some(group.clone()));
    }

    // Process include/exclude remarks
    if let Some(include) = &query.include {
        builder = builder.add_include_remark(include);
    } else if !merged_settings.external.include.is_empty() {
        for remark in &merged_settings.external.include {
            builder = builder.add_include_remark(remark);
        }
    }

    if let Some(exclude) = &query.exclude {
        builder = builder.add_exclude_remark(exclude);
    } else if !merged_settings.external.exclude.is_empty() {
        for remark in &merged_settings.external.exclude {
            builder = builder.add_exclude_remark(remark);
        }
    }

    // Process rename patterns
    if let Some(rename) = &query.rename {
        // Parse rename patterns from format "pattern1@replacement1|pattern2@replacement2"
        for pair in rename.split('|') {
            let parts: Vec<&str> = pair.split('@').collect();
            if parts.len() == 2 {
                builder = builder.add_rename_pattern(parts[0], parts[1]);
            }
        }
    }

    // Process emoji setting
    if let Some(emoji) = query.emoji {
        // Create a copy of the current extra settings
        let mut extra = merged_settings.extra.clone();
        extra.add_emoji = emoji;
        builder = builder.extra(extra);
    }

    // Process other extra settings
    if let Some(append_type) = query.append_type {
        builder = builder.append_proxy_type(append_type);
    }
    if let Some(tfo) = query.tfo {
        builder = builder.tfo(Some(tfo));
    }
    if let Some(udp) = query.udp {
        builder = builder.udp(Some(udp));
    }
    if let Some(scv) = query.scv {
        builder = builder.skip_cert_verify(Some(scv));
    }
    if let Some(tls13) = query.tls13 {
        builder = builder.tls13(Some(tls13));
    }
    if let Some(sort) = query.sort {
        builder = builder.sort(sort);
    }
    if let Some(nodelist) = query.list {
        builder = builder.nodelist(nodelist);
    }
    if let Some(new_name) = query.new_name {
        builder = builder.clash_new_field_name(new_name);
    }

    // Process filename
    if let Some(filename) = &query.filename {
        builder = builder.filename(Some(filename.clone()));
    }

    // Process update settings
    if let Some(interval) = query.interval {
        builder = builder.update_interval(interval as i32);
    }
    if let Some(strict) = query.strict {
        builder = builder.update_strict(strict);
    }

    // Process upload settings
    if let Some(upload) = query.upload {
        builder = builder.upload(upload);
    }

    // Process filter script
    if let Some(filter) = &query.filter {
        builder = builder.filter_script(Some(filter.clone()));
    }

    // Process device ID
    if let Some(dev_id) = &query.dev_id {
        builder = builder.device_id(Some(dev_id.clone()));
    }

    // Process authorization
    if let Some(token) = &query.token {
        builder = builder.token(Some(token.clone()));
        builder = builder.authorized(token == &app_state.config.api_access_token);
    }

    // Add base configuration from merged settings or global settings
    if let Ok(cfg) = builder.clone().build() {
        let target = cfg.target;

        // Get base content from external config using the helper method
        let base_content = target.get_base_content_from_external(&merged_settings.external);

        // If external config has a base, use it
        if let Some(base) = base_content {
            builder = builder.add_base_content(target, base);
        } else if let Some(base_conf) = app_state.get_base_config(&target) {
            // Otherwise fall back to app_state's base config
            builder = builder.add_base_content(target, base_conf);
        }
    }

    // Set managed config prefix from global settings
    if !app_state.config.managed_config_prefix.is_empty() {
        builder = builder.managed_config_prefix(app_state.config.managed_config_prefix.clone());
    }

    // Set emoji patterns from global settings
    if let Some(emoji_map) = &app_state.emoji_map {
        for (pattern, emoji) in emoji_map {
            builder = builder.add_emoji_pattern(pattern, emoji);
        }
    }

    // Build and validate configuration
    let config = match builder.build() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to build subconverter config: {}", e);
            return HttpResponse::BadRequest().body(format!("Configuration error: {}", e));
        }
    };

    let target = config.target.clone();

    let subconverter_result = std::thread::spawn(move || subconverter(config));

    // Run subconverter
    match subconverter_result.join().unwrap() {
        Ok(result) => {
            // Build response with headers
            let mut resp = HttpResponse::Ok();

            // Add headers from result
            for (name, value) in result.headers {
                resp.append_header((name, value));
            }

            // Set content type based on target
            match target {
                SubconverterTarget::Clash
                | SubconverterTarget::ClashR
                | SubconverterTarget::SingBox => {
                    resp.content_type("application/yaml");
                }
                SubconverterTarget::SSSub | SubconverterTarget::SSD => {
                    resp.content_type("application/json");
                }
                _ => {
                    resp.content_type("text/plain");
                }
            }

            // Return the response with the conversion result
            resp.body(result.content)
        }
        Err(e) => {
            error!("Subconverter error: {}", e);
            HttpResponse::InternalServerError().body(format!("Conversion error: {}", e))
        }
    }
}

/// Handler for simple conversion (no rules)
pub async fn simple_handler(
    path: web::Path<(String,)>,
    query: web::Query<SubconverterQuery>,
    app_state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    let target_type = &path.0;

    // Set appropriate target based on path
    match target_type.as_str() {
        "clash" | "clashr" | "surge" | "quan" | "quanx" | "loon" | "ss" | "ssr" | "ssd"
        | "v2ray" | "trojan" | "mixed" | "singbox" => {
            // Create a modified query with the target set
            let mut modified_query = query.into_inner();
            modified_query.target = Some(target_type.clone());

            // Reuse the sub_handler
            sub_handler(web::Query(modified_query), app_state).await
        }
        _ => HttpResponse::BadRequest().body(format!("Unsupported target type: {}", target_type)),
    }
}

/// Handler for Clash from Surge configuration
pub async fn surge_to_clash_handler(
    query: web::Query<SubconverterQuery>,
    app_state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    // Create a modified query with the target set to Clash
    let mut modified_query = query.into_inner();
    modified_query.target = Some("clash".to_string());

    // Set nodelist to true for this special case
    modified_query.list = Some(true);

    // Reuse the sub_handler
    sub_handler(web::Query(modified_query), app_state).await
}

/// Register the API endpoints with Actix Web
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/sub", web::get().to(sub_handler))
        .route("/surge2clash", web::get().to(surge_to_clash_handler))
        .route("/{target_type}", web::get().to(simple_handler));
}
