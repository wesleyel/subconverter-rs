"use client";

import { useState, useEffect } from 'react';
import { readSettingsFile, writeSettingsFile } from '@/lib/api-client';
import yaml from 'js-yaml';

// Settings interface based on pref.yml structure
interface SubconverterSettings {
    common?: {
        api_mode?: boolean;
        api_access_token?: string;
        default_url?: string[];
        enable_insert?: boolean;
        insert_url?: string[];
        prepend_insert_url?: boolean;
        exclude_remarks?: string[];
        include_remarks?: string[];
        enable_filter?: boolean;
        filter_script?: string;
        default_external_config?: string;
        base_path?: string;
        clash_rule_base?: string;
        surge_rule_base?: string;
        surfboard_rule_base?: string;
        mellow_rule_base?: string;
        quan_rule_base?: string;
        quanx_rule_base?: string;
        loon_rule_base?: string;
        sssub_rule_base?: string;
        singbox_rule_base?: string;
        proxy_config?: string;
        proxy_ruleset?: string;
        proxy_subscription?: string;
        append_proxy_type?: boolean;
        reload_conf_on_request?: boolean;
    };
    userinfo?: {
        stream_rule?: Array<{ match: string; replace: string }>;
        time_rule?: Array<{ match: string; replace: string }>;
    };
    node_pref?: {
        udp_flag?: boolean;
        tcp_fast_open_flag?: boolean;
        skip_cert_verify_flag?: boolean;
        tls13_flag?: boolean;
        sort_flag?: boolean;
        sort_script?: string;
        filter_deprecated_nodes?: boolean;
        append_sub_userinfo?: boolean;
        clash_use_new_field_name?: boolean;
        clash_proxies_style?: string;
        clash_proxy_groups_style?: string;
        singbox_add_clash_modes?: boolean;
        rename_node?: Array<{ match?: string; replace?: string; script?: string; import?: string }>;
    };
    managed_config?: {
        write_managed_config?: boolean;
        managed_config_prefix?: string;
        config_update_interval?: number;
        config_update_strict?: boolean;
        quanx_device_id?: string;
    };
    surge_external_proxy?: {
        surge_ssr_path?: string;
        resolve_hostname?: boolean;
    };
    emojis?: {
        add_emoji?: boolean;
        remove_old_emoji?: boolean;
        rules?: Array<{ match?: string; emoji?: string; script?: string; import?: string }>;
    };
    rulesets?: {
        enabled?: boolean;
        overwrite_original_rules?: boolean;
        update_ruleset_on_request?: boolean;
        rulesets?: Array<{ rule?: string; ruleset?: string; group?: string; interval?: number; import?: string }>;
    };
    proxy_groups?: {
        custom_proxy_group?: Array<{ name?: string; type?: string; rule?: string[]; url?: string; interval?: number; tolerance?: number; timeout?: number; import?: string }>;
    };
    template?: {
        template_path?: string;
        globals?: Array<{ key: string; value: any }>;
    };
    aliases?: Array<{ uri: string; target: string }>;
    tasks?: Array<{ name: string; cronexp: string; path: string; timeout?: number }>;
    server?: {
        listen?: string;
        port?: number;
        serve_file_root?: string;
    };
    advanced?: {
        log_level?: string;
        print_debug_info?: boolean;
        max_pending_connections?: number;
        max_concurrent_threads?: number;
        max_allowed_rulesets?: number;
        max_allowed_rules?: number;
        max_allowed_download_size?: number;
        enable_cache?: boolean;
        cache_subscription?: number;
        cache_config?: number;
        cache_ruleset?: number;
        script_clean_context?: boolean;
        async_fetch_ruleset?: boolean;
        skip_failed_links?: boolean;
    };
}

export default function SettingsPage() {
    const [settings, setSettings] = useState<SubconverterSettings>({});
    const [originalYaml, setOriginalYaml] = useState<string>('');
    const [isLoading, setIsLoading] = useState(true);
    const [isSaving, setIsSaving] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [saveSuccess, setSaveSuccess] = useState(false);
    const [activeTab, setActiveTab] = useState('common');

    useEffect(() => {
        loadSettings();
    }, []);

    const loadSettings = async () => {
        setIsLoading(true);
        setError(null);
        try {
            const yamlContent = await readSettingsFile();
            setOriginalYaml(yamlContent);

            const parsedSettings = yaml.load(yamlContent) as SubconverterSettings;
            setSettings(parsedSettings || {});
        } catch (err) {
            setError(`Failed to load settings: ${err instanceof Error ? err.message : String(err)}`);
            console.error("Error loading settings:", err);
        } finally {
            setIsLoading(false);
        }
    };

    const saveSettings = async () => {
        setIsSaving(true);
        setSaveSuccess(false);
        setError(null);

        try {
            const yamlContent = yaml.dump(settings, {
                indent: 2,
                lineWidth: -1, // Don't wrap lines
                noRefs: true,
                sortKeys: false // Preserve key order
            });

            await writeSettingsFile(yamlContent);
            setOriginalYaml(yamlContent);
            setSaveSuccess(true);

            // Hide success message after 3 seconds
            setTimeout(() => setSaveSuccess(false), 3000);
        } catch (err) {
            setError(`Failed to save settings: ${err instanceof Error ? err.message : String(err)}`);
            console.error("Error saving settings:", err);
        } finally {
            setIsSaving(false);
        }
    };

    const handleInputChange = (section: keyof SubconverterSettings, key: string, value: any) => {
        setSettings(prevSettings => ({
            ...prevSettings,
            [section]: {
                ...prevSettings[section],
                [key]: value
            }
        }));
    };

    // Handle array input changes (like exclude_remarks)
    const handleArrayChange = (section: keyof SubconverterSettings, key: string, value: string) => {
        const arrayValue = value.split(',').map(item => item.trim());
        setSettings(prevSettings => ({
            ...prevSettings,
            [section]: {
                ...prevSettings[section],
                [key]: arrayValue
            }
        }));
    };

    const renderCommonSection = () => {
        const common = settings.common || {};
        return (
            <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">API Mode</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={common.api_mode ? "true" : "false"}
                            onChange={(e) => handleInputChange('common', 'api_mode', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">API Access Token</label>
                        <input
                            type="text"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={common.api_access_token || ''}
                            onChange={(e) => handleInputChange('common', 'api_access_token', e.target.value)}
                        />
                    </div>
                </div>

                <div>
                    <label className="block text-sm font-medium mb-1">Default URLs (comma separated)</label>
                    <textarea
                        className="w-full p-2 border border-gray-300 rounded bg-white/10"
                        value={(common.default_url || []).join(', ')}
                        onChange={(e) => handleArrayChange('common', 'default_url', e.target.value)}
                        rows={2}
                    />
                </div>

                <div>
                    <label className="block text-sm font-medium mb-1">Insert URLs (comma separated)</label>
                    <textarea
                        className="w-full p-2 border border-gray-300 rounded bg-white/10"
                        value={(common.insert_url || []).join(', ')}
                        onChange={(e) => handleArrayChange('common', 'insert_url', e.target.value)}
                        rows={2}
                    />
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Enable Insert</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={common.enable_insert ? "true" : "false"}
                            onChange={(e) => handleInputChange('common', 'enable_insert', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Prepend Insert</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={common.prepend_insert_url ? "true" : "false"}
                            onChange={(e) => handleInputChange('common', 'prepend_insert_url', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>
                </div>

                <div>
                    <label className="block text-sm font-medium mb-1">Exclude Remarks (regex patterns, comma separated)</label>
                    <textarea
                        className="w-full p-2 border border-gray-300 rounded bg-white/10"
                        value={(common.exclude_remarks || []).join(', ')}
                        onChange={(e) => handleArrayChange('common', 'exclude_remarks', e.target.value)}
                        rows={2}
                    />
                </div>

                <div>
                    <label className="block text-sm font-medium mb-1">Include Remarks (regex patterns, comma separated)</label>
                    <textarea
                        className="w-full p-2 border border-gray-300 rounded bg-white/10"
                        value={(common.include_remarks || []).join(', ')}
                        onChange={(e) => handleArrayChange('common', 'include_remarks', e.target.value)}
                        rows={2}
                    />
                </div>
            </div>
        );
    };

    const renderServerSection = () => {
        const server = settings.server || {};
        return (
            <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Listen Address</label>
                        <input
                            type="text"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={server.listen || ''}
                            onChange={(e) => handleInputChange('server', 'listen', e.target.value)}
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Port</label>
                        <input
                            type="number"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={server.port || ''}
                            onChange={(e) => handleInputChange('server', 'port', parseInt(e.target.value) || 0)}
                        />
                    </div>
                </div>

                <div>
                    <label className="block text-sm font-medium mb-1">Serve File Root</label>
                    <input
                        type="text"
                        className="w-full p-2 border border-gray-300 rounded bg-white/10"
                        value={server.serve_file_root || ''}
                        onChange={(e) => handleInputChange('server', 'serve_file_root', e.target.value)}
                    />
                </div>
            </div>
        );
    };

    const renderAdvancedSection = () => {
        const advanced = settings.advanced || {};
        return (
            <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Log Level</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.log_level || 'info'}
                            onChange={(e) => handleInputChange('advanced', 'log_level', e.target.value)}
                        >
                            <option value="debug">Debug</option>
                            <option value="info">Info</option>
                            <option value="warn">Warning</option>
                            <option value="error">Error</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Print Debug Info</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.print_debug_info ? "true" : "false"}
                            onChange={(e) => handleInputChange('advanced', 'print_debug_info', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Max Pending Connections</label>
                        <input
                            type="number"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.max_pending_connections || ''}
                            onChange={(e) => handleInputChange('advanced', 'max_pending_connections', parseInt(e.target.value) || 0)}
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Max Concurrent Threads</label>
                        <input
                            type="number"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.max_concurrent_threads || ''}
                            onChange={(e) => handleInputChange('advanced', 'max_concurrent_threads', parseInt(e.target.value) || 0)}
                        />
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Max Allowed Rulesets</label>
                        <input
                            type="number"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.max_allowed_rulesets || ''}
                            onChange={(e) => handleInputChange('advanced', 'max_allowed_rulesets', parseInt(e.target.value) || 0)}
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Max Allowed Rules</label>
                        <input
                            type="number"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.max_allowed_rules || ''}
                            onChange={(e) => handleInputChange('advanced', 'max_allowed_rules', parseInt(e.target.value) || 0)}
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Max Download Size (bytes)</label>
                        <input
                            type="number"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.max_allowed_download_size || ''}
                            onChange={(e) => handleInputChange('advanced', 'max_allowed_download_size', parseInt(e.target.value) || 0)}
                        />
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Enable Cache</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.enable_cache ? "true" : "false"}
                            onChange={(e) => handleInputChange('advanced', 'enable_cache', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Skip Failed Links</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.skip_failed_links ? "true" : "false"}
                            onChange={(e) => handleInputChange('advanced', 'skip_failed_links', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Cache Subscription (seconds)</label>
                        <input
                            type="number"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.cache_subscription || ''}
                            onChange={(e) => handleInputChange('advanced', 'cache_subscription', parseInt(e.target.value) || 0)}
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Cache Config (seconds)</label>
                        <input
                            type="number"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.cache_config || ''}
                            onChange={(e) => handleInputChange('advanced', 'cache_config', parseInt(e.target.value) || 0)}
                        />
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Cache Ruleset (seconds)</label>
                        <input
                            type="number"
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={advanced.cache_ruleset || ''}
                            onChange={(e) => handleInputChange('advanced', 'cache_ruleset', parseInt(e.target.value) || 0)}
                        />
                    </div>
                </div>
            </div>
        );
    };

    const renderNodePrefSection = () => {
        const nodePref = settings.node_pref || {};
        return (
            <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Enable UDP</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.udp_flag === undefined ? "" : (nodePref.udp_flag ? "true" : "false")}
                            onChange={(e) => {
                                if (e.target.value === "") {
                                    const newSettings = { ...settings };
                                    if (newSettings.node_pref) {
                                        delete newSettings.node_pref.udp_flag;
                                        setSettings(newSettings);
                                    }
                                } else {
                                    handleInputChange('node_pref', 'udp_flag', e.target.value === "true");
                                }
                            }}
                        >
                            <option value="">Not Set</option>
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">TFO (TCP Fast Open)</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.tcp_fast_open_flag === undefined ? "" : (nodePref.tcp_fast_open_flag ? "true" : "false")}
                            onChange={(e) => {
                                if (e.target.value === "") {
                                    const newSettings = { ...settings };
                                    if (newSettings.node_pref) {
                                        delete newSettings.node_pref.tcp_fast_open_flag;
                                        setSettings(newSettings);
                                    }
                                } else {
                                    handleInputChange('node_pref', 'tcp_fast_open_flag', e.target.value === "true");
                                }
                            }}
                        >
                            <option value="">Not Set</option>
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Skip Cert Verify</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.skip_cert_verify_flag === undefined ? "" : (nodePref.skip_cert_verify_flag ? "true" : "false")}
                            onChange={(e) => {
                                if (e.target.value === "") {
                                    const newSettings = { ...settings };
                                    if (newSettings.node_pref) {
                                        delete newSettings.node_pref.skip_cert_verify_flag;
                                        setSettings(newSettings);
                                    }
                                } else {
                                    handleInputChange('node_pref', 'skip_cert_verify_flag', e.target.value === "true");
                                }
                            }}
                        >
                            <option value="">Not Set</option>
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Enable TLS 1.3</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.tls13_flag === undefined ? "" : (nodePref.tls13_flag ? "true" : "false")}
                            onChange={(e) => {
                                if (e.target.value === "") {
                                    const newSettings = { ...settings };
                                    if (newSettings.node_pref) {
                                        delete newSettings.node_pref.tls13_flag;
                                        setSettings(newSettings);
                                    }
                                } else {
                                    handleInputChange('node_pref', 'tls13_flag', e.target.value === "true");
                                }
                            }}
                        >
                            <option value="">Not Set</option>
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Sort Nodes</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.sort_flag ? "true" : "false"}
                            onChange={(e) => handleInputChange('node_pref', 'sort_flag', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Filter Deprecated Nodes</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.filter_deprecated_nodes ? "true" : "false"}
                            onChange={(e) => handleInputChange('node_pref', 'filter_deprecated_nodes', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Append Sub User Info</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.append_sub_userinfo ? "true" : "false"}
                            onChange={(e) => handleInputChange('node_pref', 'append_sub_userinfo', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Clash Use New Field Names</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.clash_use_new_field_name ? "true" : "false"}
                            onChange={(e) => handleInputChange('node_pref', 'clash_use_new_field_name', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">SingBox Add Clash Modes</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.singbox_add_clash_modes ? "true" : "false"}
                            onChange={(e) => handleInputChange('node_pref', 'singbox_add_clash_modes', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Clash Proxies Style</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.clash_proxies_style || 'flow'}
                            onChange={(e) => handleInputChange('node_pref', 'clash_proxies_style', e.target.value)}
                        >
                            <option value="flow">Flow</option>
                            <option value="block">Block</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Clash Proxy Groups Style</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={nodePref.clash_proxy_groups_style || 'flow'}
                            onChange={(e) => handleInputChange('node_pref', 'clash_proxy_groups_style', e.target.value)}
                        >
                            <option value="flow">Flow</option>
                            <option value="block">Block</option>
                        </select>
                    </div>
                </div>
            </div>
        );
    };

    const renderEmojisSection = () => {
        const emojis = settings.emojis || {};
        return (
            <div className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label className="block text-sm font-medium mb-1">Add Emoji</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={emojis.add_emoji ? "true" : "false"}
                            onChange={(e) => handleInputChange('emojis', 'add_emoji', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>

                    <div>
                        <label className="block text-sm font-medium mb-1">Remove Old Emoji</label>
                        <select
                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                            value={emojis.remove_old_emoji ? "true" : "false"}
                            onChange={(e) => handleInputChange('emojis', 'remove_old_emoji', e.target.value === "true")}
                        >
                            <option value="true">Enabled</option>
                            <option value="false">Disabled</option>
                        </select>
                    </div>
                </div>

                <div>
                    <label className="block text-sm font-medium mb-1">Emoji Rules</label>
                    <p className="text-xs mb-2 text-gray-400">
                        Emoji rules are defined in snippets/emoji.txt by default. You can modify that file directly.
                    </p>
                </div>
            </div>
        );
    };

    const renderTab = () => {
        switch (activeTab) {
            case 'common':
                return renderCommonSection();
            case 'server':
                return renderServerSection();
            case 'advanced':
                return renderAdvancedSection();
            case 'node_pref':
                return renderNodePrefSection();
            case 'emojis':
                return renderEmojisSection();
            default:
                return <p>Select a section to edit settings.</p>;
        }
    };

    if (isLoading) {
        return (
            <div className="flex min-h-screen flex-col items-center justify-center">
                <div className="mb-4 text-xl">Loading settings...</div>
            </div>
        );
    }

    return (
        <main className="flex min-h-screen flex-col items-center justify-between p-4 md:p-8 lg:p-24">
            <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm">
                <div className="flex flex-col sm:flex-row items-center justify-between mb-6">
                    <h1 className="text-4xl font-bold mb-4 sm:mb-0 text-center">Server Settings</h1>
                    <div className="flex space-x-2">
                        <button
                            onClick={loadSettings}
                            disabled={isSaving}
                            className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded disabled:opacity-50"
                        >
                            Reload
                        </button>
                        <button
                            onClick={saveSettings}
                            disabled={isSaving}
                            className="bg-green-600 hover:bg-green-700 text-white px-4 py-2 rounded disabled:opacity-50"
                        >
                            {isSaving ? "Saving..." : "Save Changes"}
                        </button>
                    </div>
                </div>

                {error && (
                    <div className="mb-4 p-4 border border-red-400 bg-red-50 text-red-700 rounded-md">
                        {error}
                    </div>
                )}

                {saveSuccess && (
                    <div className="mb-4 p-4 border border-green-400 bg-green-50 text-green-700 rounded-md">
                        Settings saved successfully!
                    </div>
                )}

                <div className="bg-white/5 p-6 rounded-lg shadow-md mb-8">
                    <div className="flex flex-wrap mb-6 border-b border-gray-300 pb-2">
                        <button
                            className={`mr-4 py-2 px-4 ${activeTab === 'common' ? 'border-b-2 border-blue-500 font-bold' : ''}`}
                            onClick={() => setActiveTab('common')}
                        >
                            Common
                        </button>
                        <button
                            className={`mr-4 py-2 px-4 ${activeTab === 'server' ? 'border-b-2 border-blue-500 font-bold' : ''}`}
                            onClick={() => setActiveTab('server')}
                        >
                            Server
                        </button>
                        <button
                            className={`mr-4 py-2 px-4 ${activeTab === 'node_pref' ? 'border-b-2 border-blue-500 font-bold' : ''}`}
                            onClick={() => setActiveTab('node_pref')}
                        >
                            Node Preferences
                        </button>
                        <button
                            className={`mr-4 py-2 px-4 ${activeTab === 'emojis' ? 'border-b-2 border-blue-500 font-bold' : ''}`}
                            onClick={() => setActiveTab('emojis')}
                        >
                            Emojis
                        </button>
                        <button
                            className={`mr-4 py-2 px-4 ${activeTab === 'advanced' ? 'border-b-2 border-blue-500 font-bold' : ''}`}
                            onClick={() => setActiveTab('advanced')}
                        >
                            Advanced
                        </button>
                    </div>

                    {renderTab()}
                </div>

                <div className="bg-white/5 p-6 rounded-lg shadow-md mb-8">
                    <h2 className="text-xl font-semibold mb-4">YAML Editor</h2>
                    <p className="text-sm mb-4 text-gray-400">
                        For advanced users: You can edit the YAML directly. This will be applied when you save changes.
                    </p>
                    <textarea
                        className="w-full h-64 p-2 border border-gray-300 rounded bg-white/10 font-mono text-xs"
                        value={yaml.dump(settings, {
                            indent: 2,
                            lineWidth: -1,
                            noRefs: true,
                            sortKeys: false
                        })}
                        onChange={(e) => {
                            try {
                                const parsed = yaml.load(e.target.value) as SubconverterSettings;
                                setSettings(parsed || {});
                            } catch (err) {
                                // Don't update state on invalid YAML, but still allow editing
                                console.error("Invalid YAML:", err);
                            }
                        }}
                    />
                </div>
            </div>
        </main>
    );
} 