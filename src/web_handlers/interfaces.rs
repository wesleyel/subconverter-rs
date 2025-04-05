use actix_web::{web, HttpRequest, HttpResponse};
use log::{debug, error};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::constants::regex_black_list::REGEX_BLACK_LIST;
use crate::interfaces::subconverter::{subconverter, SubconverterConfigBuilder};
use crate::models::ruleset::RulesetConfigs;
use crate::models::{AppState, ProxyGroupConfigs, RegexMatchConfigs, SubconverterTarget};
use crate::settings::external::ExternalSettings;
use crate::settings::{refresh_configuration, FromIni, FromIniWithDelimiter};
use crate::utils::{file_exists, is_link, reg_valid, starts_with, url_decode};
use crate::{RuleBases, Settings, TemplateArgs};
fn default_ver() -> u32 {
    3
}
/// Query parameters for subscription conversion
#[derive(Deserialize, Debug, Default, Clone)]
pub struct SubconverterQuery {
    /// Target format
    pub target: Option<String>,
    /// Surge version number
    #[serde(default = "default_ver")]
    pub ver: u32,
    /// Clash new field name
    pub new_name: Option<bool>,
    /// URLs to convert (pipe separated)
    pub url: Option<String>,
    /// Custom group name
    pub group: Option<String>,
    /// Upload path (optional)
    pub upload_path: Option<String>,
    /// Include remarks regex, multiple regexes separated by '|'
    pub include: Option<String>,
    /// Exclude remarks regex, multiple regexes separated by '|'
    pub exclude: Option<String>,
    /// custom groups
    pub groups: Option<String>,
    /// Ruleset contents
    pub ruleset: Option<String>,
    /// External configuration file (optional)
    pub config: Option<String>,

    /// Device ID (for device-specific configurations)
    pub dev_id: Option<String>,
    /// Whether to insert nodes
    pub insert: Option<bool>,
    /// Whether to prepend insert nodes
    pub prepend: Option<bool>,
    /// Custom filename for download
    pub filename: Option<String>,
    /// Append proxy type to remarks
    pub append_type: Option<bool>,
    /// Whether to remove old emoji and add new emoji
    pub emoji: Option<bool>,
    /// Whether to add emoji
    pub add_emoji: Option<bool>,
    /// Whether to remove emoji
    pub remove_emoji: Option<bool>,
    /// List mode (node list only)
    pub list: Option<bool>,
    /// Sort nodes
    pub sort: Option<bool>,

    /// Sort Script
    pub sort_script: Option<String>,

    /// argFilterDeprecated
    pub fdn: Option<bool>,

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

    /// Clash script
    pub script: Option<bool>,
    pub classic: Option<bool>,

    pub expand: Option<bool>,
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
    req: HttpRequest,
    query: web::Query<SubconverterQuery>,
    app_state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    debug!("Received subconverter request: {:?}", query);
    let mut global = Settings::current();

    // Check if we should reload the config
    if global.reload_conf_on_request && !global.api_mode && !global.generator_mode {
        refresh_configuration();
        global = Settings::current();
    }

    // Start building configuration
    let mut builder = SubconverterConfigBuilder::new();

    let target;
    if let Some(_target) = &query.target {
        match SubconverterTarget::from_str(&_target) {
            Some(_target) => {
                target = _target.clone();
                if _target == SubconverterTarget::Auto {
                    // TODO: Check user agent and set target accordingly
                    // if let Some(user_agent) = req.headers().get("User-Agent") {
                    //     if let Ok(user_agent) = user_agent.to_str() {

                    //         // match_user_agent(
                    //         //     user_agent,
                    //         //     &target,
                    //         //      query.new_name,
                    //         //      &query.ver);
                    //     }
                    // }
                    return HttpResponse::BadRequest()
                        .body("Auto user agent is not supported for now.");
                }
                builder.target(_target);
            }
            None => {
                return HttpResponse::BadRequest().body("Invalid target parameter");
            }
        }
    } else {
        return HttpResponse::BadRequest().body("Missing target parameter");
    }

    builder.update_interval(match query.interval {
        Some(interval) => interval,
        None => global.update_interval,
    });
    // Check if we should authorize the request, if we are in API mode
    let authorized =
        !global.api_mode || query.token.as_deref().unwrap_or_default() == global.api_access_token;
    builder.authorized(authorized);
    builder.update_strict(query.strict.unwrap_or(global.update_strict));

    if query
        .include
        .clone()
        .is_some_and(|include| REGEX_BLACK_LIST.contains(&include))
        || query
            .exclude
            .clone()
            .is_some_and(|exclude| REGEX_BLACK_LIST.contains(&exclude))
    {
        return HttpResponse::BadRequest().body("Invalid regex in request!");
    }

    let enable_insert = match query.insert {
        Some(insert) => insert,
        None => global.enable_insert,
    };

    if enable_insert {
        builder.insert_urls(global.insert_urls.clone());
        // 加在前面还是加在后面
        builder.prepend_insert(query.prepend.unwrap_or(global.prepend_insert));
    }

    let urls = match query.url.as_deref() {
        Some(url) => url_decode(url).split('|').map(|s| s.to_owned()).collect(),
        None => {
            if authorized {
                global.default_urls.clone()
            } else {
                vec![]
            }
        }
    };
    builder.urls(urls);

    // TODO: what if urls still empty after insert?

    // Create template args from request parameters and other settings
    let mut template_args = TemplateArgs::default();
    template_args.global_vars = global.template_vars.clone();

    template_args.request_params = url::form_urlencoded::parse(req.query_string().as_bytes())
        .into_owned()
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    builder.append_proxy_type(query.append_type.unwrap_or(global.append_type));

    let mut arg_expand_rulesets = query.expand;
    if target.is_clash() && query.script.is_none() {
        arg_expand_rulesets = Some(true);
    }

    // flags
    builder.tfo(query.tfo.or(global.tfo_flag));
    builder.udp(query.udp.or(global.udp_flag));
    builder.skip_cert_verify(query.scv.or(global.skip_cert_verify));
    builder.tls13(query.tls13.or(global.tls13_flag));
    builder.sort(query.sort.unwrap_or(global.enable_sort));
    if let Some(script) = &query.sort_script {
        builder.sort_script(script.clone());
    }

    builder.filter_deprecated(query.fdn.unwrap_or(global.filter_deprecated));
    builder.clash_new_field_name(query.new_name.unwrap_or(global.clash_use_new_field));
    builder.clash_script(query.script.unwrap_or_default());
    builder.clash_classical_ruleset(query.classic.unwrap_or_default());
    let nodelist = query.list.unwrap_or_default();
    builder.nodelist(nodelist);

    if arg_expand_rulesets != Some(true) {
        builder.clash_new_field_name(true);
    } else {
        builder.managed_config_prefix(global.managed_config_prefix.clone());
        builder.clash_script(false);
    }

    let mut ruleset_configs = global.custom_rulesets.clone();
    let mut custom_group_configs = global.custom_proxy_groups.clone();

    // 这部分参数有优先级：query > external > global
    builder.include_remarks(global.include_remarks.clone());
    builder.exclude_remarks(global.exclude_remarks.clone());
    builder.rename_array(global.renames.clone());
    builder.emoji_array(global.emojis.clone());
    builder.add_emoji(global.add_emoji);
    builder.remove_emoji(global.remove_emoji);
    builder.enable_rule_generator(global.enable_rule_gen);

    let ext_config = query
        .config
        .as_deref()
        .unwrap_or(&global.default_ext_config);
    if !ext_config.is_empty() {
        let ext_config_clone = ext_config.to_string();
        let handler = std::thread::spawn(move || {
            match ExternalSettings::load_from_file_sync(&ext_config_clone) {
                Ok(extconf) => Some(extconf),
                Err(e) => {
                    error!(
                        "Failed to load external config from {}: {}",
                        ext_config_clone, e
                    );
                    None
                }
            }
        });
        // Process external config if provided
        match handler.join().unwrap() {
            Some(extconf) => {
                if !nodelist {
                    let mut rule_bases = RuleBases {
                        clash_rule_base: global.clash_base.clone(),
                        surge_rule_base: global.surge_base.clone(),
                        surfboard_rule_base: global.surfboard_base.clone(),
                        mellow_rule_base: global.mellow_base.clone(),
                        quan_rule_base: global.quan_base.clone(),
                        quanx_rule_base: global.quanx_base.clone(),
                        loon_rule_base: global.loon_base.clone(),
                        sssub_rule_base: global.ssub_base.clone(),
                        singbox_rule_base: global.singbox_base.clone(),
                    };
                    rule_bases.check_external_bases(&extconf, &global.base_path);
                    builder.rule_bases(rule_bases);

                    if let Some(tpl_args) = extconf.tpl_args {
                        template_args.local_vars = tpl_args;
                    }

                    builder.template_args(template_args);

                    if !target.is_simple() {
                        if !extconf.custom_rulesets.is_empty() {
                            ruleset_configs = extconf.custom_rulesets;
                        }
                        if !extconf.custom_proxy_groups.is_empty() {
                            custom_group_configs = extconf.custom_proxy_groups;
                        }
                        if let Some(enable_rule_gen) = extconf.enable_rule_generator {
                            builder.enable_rule_generator(enable_rule_gen);
                        }
                        if let Some(overwrite_original_rules) = extconf.overwrite_original_rules {
                            builder.overwrite_original_rules(overwrite_original_rules);
                        }
                    }
                }
                if !extconf.rename_nodes.is_empty() {
                    builder.rename_array(extconf.rename_nodes);
                }
                if !extconf.emojis.is_empty() {
                    builder.emoji_array(extconf.emojis);
                }
                if !extconf.include_remarks.is_empty() {
                    builder.include_remarks(extconf.include_remarks);
                }
                if !extconf.exclude_remarks.is_empty() {
                    builder.exclude_remarks(extconf.exclude_remarks);
                }
                if extconf.add_emoji.is_some() {
                    builder.add_emoji(extconf.add_emoji.unwrap());
                }
                if extconf.remove_old_emoji.is_some() {
                    builder.remove_emoji(extconf.remove_old_emoji.unwrap());
                }
            }
            None => {
                error!("Failed to load external config from {}", ext_config);
            }
        }
    }

    // 请求参数的覆盖优先级最高
    if let Some(include) = query.include.as_deref() {
        if reg_valid(&include) {
            builder.include_remarks(vec![include.to_owned()]);
        }
    }
    if let Some(exclude) = query.exclude.as_deref() {
        if reg_valid(&exclude) {
            builder.exclude_remarks(vec![exclude.to_owned()]);
        }
    }
    if let Some(emoji) = query.emoji {
        builder.add_emoji(emoji);
        builder.remove_emoji(true);
    }

    if let Some(add_emoji) = query.add_emoji {
        builder.add_emoji(add_emoji);
    }
    if let Some(remove_emoji) = query.remove_emoji {
        builder.remove_emoji(remove_emoji);
    }
    if let Some(rename) = query.rename.as_deref() {
        if !rename.is_empty() {
            let v_array: Vec<String> = rename.split('`').map(|s| s.to_string()).collect();
            builder.rename_array(RegexMatchConfigs::from_ini_with_delimiter(&v_array, "@"));
        }
    }

    if !target.is_simple() {
        // loading custom groups
        if !query
            .groups
            .as_deref()
            .is_none_or(|groups| groups.is_empty())
            && !nodelist
        {
            if let Some(groups) = query.groups.as_deref() {
                let v_array: Vec<String> = groups.split('@').map(|s| s.to_string()).collect();
                custom_group_configs = ProxyGroupConfigs::from_ini(&v_array);
            }
        }
        // loading custom rulesets
        if !query
            .ruleset
            .as_deref()
            .is_none_or(|ruleset| ruleset.is_empty())
            && !nodelist
        {
            if let Some(ruleset) = query.ruleset.as_deref() {
                let v_array: Vec<String> = ruleset.split('@').map(|s| s.to_string()).collect();
                ruleset_configs = RulesetConfigs::from_ini(&v_array);
            }
        }
    }
    builder.proxy_groups(custom_group_configs);
    builder.ruleset_configs(ruleset_configs);

    // TODO: process with the script runtime

    // parse settings

    // Process group name
    builder.group_name(query.group.clone());
    builder.filename(query.filename.clone());
    builder.upload(query.upload.unwrap_or_default());

    // // Process filter script
    // if let Some(filter) = &query.filter {
    //     builder = builder.filter_script(Some(filter.clone()));
    // }

    // // Process device ID
    // if let Some(dev_id) = &query.dev_id {
    //     builder = builder.device_id(Some(dev_id.clone()));
    // }

    // // Set managed config prefix from global settings
    // if !global.managed_config_prefix.is_empty() {
    //     builder = builder.managed_config_prefix(global.managed_config_prefix.clone());
    // }

    // Build and validate configuration
    let config = match builder.build() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to build subconverter config: {}", e);
            return HttpResponse::BadRequest().body(format!("Configuration error: {}", e));
        }
    };

    // Run subconverter
    let subconverter_result = std::thread::spawn(move || subconverter(config));

    match subconverter_result
        .join()
        .unwrap_or(Err(format!("Subconverter thread panicked")))
    {
        Ok(result) => {
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
    req: HttpRequest,
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
            sub_handler(req, web::Query(modified_query), app_state).await
        }
        _ => HttpResponse::BadRequest().body(format!("Unsupported target type: {}", target_type)),
    }
}

/// Handler for Clash from Surge configuration
pub async fn surge_to_clash_handler(
    req: HttpRequest,
    query: web::Query<SubconverterQuery>,
    app_state: web::Data<Arc<AppState>>,
) -> HttpResponse {
    // Create a modified query with the target set to Clash
    let mut modified_query = query.into_inner();
    modified_query.target = Some("clash".to_string());

    // Set nodelist to true for this special case
    modified_query.list = Some(true);

    // Reuse the sub_handler
    sub_handler(req, web::Query(modified_query), app_state).await
}

/// Register the API endpoints with Actix Web
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/sub", web::get().to(sub_handler))
        .route("/surge2clash", web::get().to(surge_to_clash_handler))
        .route("/{target_type}", web::get().to(simple_handler));
}
