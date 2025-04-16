'use client';

import React, { useState, useCallback, ChangeEvent, FormEvent } from 'react';
import {
    convertSubscription,
    SubconverterFormParams,
    SubResponseData,
    ErrorData
} from '@/lib/api-client';

// Define supported targets
const SUPPORTED_TARGETS = [
    'auto', 'clash', 'clashr', 'surge', 'quan', 'quanx',
    'mellow', 'surfboard', 'loon', 'ss', 'ssr', 'sssub',
    'v2ray', 'trojan', 'trojan-go', 'hysteria', 'hysteria2',
    'ssd', 'mixed', 'singbox'
];

export default function ConvertPage() {
    const [formData, setFormData] = useState<SubconverterFormParams>({
        target: 'clash',
        url: '',
    });

    // Track which fields have been explicitly set by the user
    const [setFields, setSetFields] = useState<Set<string>>(new Set(['target', 'url']));

    const [isLoading, setIsLoading] = useState(false);
    const [result, setResult] = useState<SubResponseData | null>(null);
    const [error, setError] = useState<ErrorData | null>(null);
    const [saveApiUrl, setSaveApiUrl] = useState(false);

    const handleInputChange = useCallback((e: ChangeEvent<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>) => {
        const { name, value, type } = e.target;
        const propertyName = name as keyof SubconverterFormParams;

        setFormData(prev => {
            const newFormData = { ...prev };

            // Update the form field value
            if (type === 'checkbox') {
                (newFormData[propertyName] as any) = (e.target as HTMLInputElement).checked;
            } else if (type === 'number') {
                const numValue = value === '' ? undefined : parseInt(value, 10);
                (newFormData[propertyName] as any) = numValue;
            } else {
                // For text fields, empty string is a valid value
                (newFormData[propertyName] as any) = value;
            }

            // Special handling for target changes
            if (name === 'target' && (!setFields.has('filename') ||
                ['config.yaml', 'config.json', 'profile.conf'].includes(prev.filename || ''))) {
                const newTarget = value;
                let defaultFilename = 'config.txt';
                if (newTarget.startsWith('clash') || newTarget === 'singbox') {
                    defaultFilename = 'config.yaml';
                } else if (newTarget === 'sssub' || newTarget === 'ssd') {
                    defaultFilename = 'config.json';
                } else if (['surge', 'quan', 'quanx', 'loon', 'surfboard', 'mellow'].includes(newTarget)) {
                    defaultFilename = 'profile.conf';
                }
                newFormData.filename = defaultFilename;
            }

            // Handle emoji flags
            if (name === 'emoji') {
                const checked = (e.target as HTMLInputElement).checked;
                newFormData.emoji = checked;
                if (checked) {
                    // When enabling combined emoji, implicitly set these too
                    newFormData.add_emoji = true;
                    newFormData.remove_emoji = true;
                    // Update set fields
                    setSetFields(prev => new Set([...prev, 'emoji', 'add_emoji', 'remove_emoji']));
                    return newFormData;
                }
            } else if (name === 'add_emoji' || name === 'remove_emoji') {
                // If a specific flag is changed, uncheck the combined 'emoji' flag
                newFormData.emoji = false;
                setSetFields(prev => new Set([...prev, 'emoji', name]));
                return newFormData;
            }

            // Mark this field as set
            setSetFields(prev => new Set([...prev, name]));
            return newFormData;
        });
    }, [setFields]);

    const handleResetField = useCallback((fieldName: string) => {
        setFormData(prev => {
            const newFormData = { ...prev };
            // Delete the property to truly unset it
            delete (newFormData as any)[fieldName];
            return newFormData;
        });

        setSetFields(prev => {
            const newSet = new Set(prev);
            newSet.delete(fieldName);
            return newSet;
        });
    }, []);

    const handleSubmit = useCallback(async (e: FormEvent) => {
        e.preventDefault();
        setIsLoading(true);
        setResult(null);
        setError(null);

        try {
            const responseData = await convertSubscription(formData);
            setResult(responseData);
        } catch (err) {
            console.error("Conversion API call failed:", err);
            setError(err as ErrorData || {
                error: 'Failed to connect to the conversion API.',
                details: String(err)
            });
        } finally {
            setIsLoading(false);
        }
    }, [formData]);

    const handleDownload = useCallback(() => {
        if (!result || !result.content) return;

        const blob = new Blob([result.content], { type: result.content_type || 'text/plain' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = formData.filename || 'config'; // Use filename from form or default
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
    }, [result, formData.filename]);

    const isSubmitDisabled = !formData.target || !formData.url || isLoading;

    // Basic styling using Tailwind (assuming setup)
    const inputClass = "mt-1 block w-full px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm";
    const checkboxClass = "h-4 w-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500";
    const labelClass = "block text-sm font-medium text-gray-700";
    const fieldsetLegendClass = "text-lg font-semibold text-gray-900 mb-2";
    const buttonClass = "inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50";

    // Add a helper component for field labels with reset button
    const FieldLabel = ({ htmlFor, children, fieldName, required = false }: {
        htmlFor: string,
        children: React.ReactNode,
        fieldName: string,
        required?: boolean
    }) => {
        const isSet = setFields.has(fieldName);
        const canReset = isSet && !required;

        return (
            <div className="flex justify-between items-center">
                <label htmlFor={htmlFor} className={labelClass}>
                    {children}
                    {required && <span className="text-red-500 ml-1">*</span>}
                    {isSet && !required && (
                        <span className="ml-2 text-xs font-normal text-green-600">
                            (set)
                        </span>
                    )}
                </label>
                {canReset && (
                    <button
                        type="button"
                        onClick={() => handleResetField(fieldName)}
                        className="text-xs text-gray-500 hover:text-red-500"
                        title="Reset to unset"
                    >
                        unset
                    </button>
                )}
            </div>
        );
    };

    // Update the input classes to show set vs. unset state
    const getInputClass = (fieldName: string) => {
        const baseClass = "mt-1 block w-full px-3 py-2 bg-white border rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm";
        if (setFields.has(fieldName)) {
            return `${baseClass} border-green-300`;
        }
        return `${baseClass} border-gray-300`;
    };

    // Generate API URL from form data
    const generateApiUrl = useCallback(() => {
        const baseUrl = window.location.origin + '/api/sub';
        const params = new URLSearchParams();

        // Add all set fields to the URL params
        Object.entries(formData).forEach(([key, value]) => {
            if (value !== undefined && value !== null && setFields.has(key)) {
                if (typeof value === 'boolean') {
                    // For boolean values, just include the parameter name if true
                    if (value) {
                        params.append(key, '1');
                    }
                } else {
                    params.append(key, String(value));
                }
            }
        });

        return `${baseUrl}?${params.toString()}`;
    }, [formData, setFields]);

    // Replace the placeholder return statement with the actual form UI
    return (
        <div className="container mx-auto p-4 max-w-4xl">
            <h1 className="text-2xl font-bold mb-6">Subscription Converter</h1>

            <form onSubmit={handleSubmit} className="space-y-6">
                {/* Required Section */}
                <fieldset className="p-4 border rounded-md">
                    <legend className={fieldsetLegendClass}>Required</legend>
                    <div className="grid grid-cols-1 gap-6">
                        <div>
                            <FieldLabel htmlFor="target" fieldName="target" required>Target Format</FieldLabel>
                            <select
                                id="target"
                                name="target"
                                value={formData.target}
                                onChange={handleInputChange}
                                required
                                className={getInputClass("target")}
                            >
                                {SUPPORTED_TARGETS.map(t => <option key={t} value={t}>{t}</option>)}
                            </select>
                        </div>
                        <div>
                            <FieldLabel htmlFor="url" fieldName="url" required>Subscription URL(s)</FieldLabel>
                            <textarea
                                id="url"
                                name="url"
                                value={formData.url}
                                onChange={handleInputChange}
                                required
                                placeholder="Enter one or more URLs, separated by |"
                                rows={3}
                                className={getInputClass("url")}
                            />
                            <p className="mt-1 text-xs text-gray-500">Separate multiple URLs with a pipe (|).</p>
                        </div>
                    </div>
                </fieldset>

                {/* Config Section */}
                <fieldset className="p-4 border-2 rounded-md border-blue-300 bg-blue-50 shadow-sm">
                    <legend className={`${fieldsetLegendClass} text-blue-800`}>Configuration Presets</legend>
                    <div className="grid grid-cols-1 gap-4">
                        <div>
                            <FieldLabel htmlFor="config" fieldName="config">External Config</FieldLabel>
                            <div className="grid grid-cols-1 gap-2">
                                <div className="flex flex-wrap gap-2 mb-2">
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setFormData(prev => ({ ...prev, config: 'https://raw.githubusercontent.com/ACL4SSR/ACL4SSR/master/Clash/config/ACL4SSR_Online.ini' }));
                                            setSetFields(prev => new Set([...prev, 'config']));
                                        }}
                                        className="px-3 py-1.5 text-xs rounded bg-blue-100 hover:bg-blue-200 border border-blue-300 transition-colors"
                                    >
                                        ACL4SSR Online
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setFormData(prev => ({ ...prev, config: 'https://raw.githubusercontent.com/ACL4SSR/ACL4SSR/master/Clash/config/ACL4SSR_Online_Full.ini' }));
                                            setSetFields(prev => new Set([...prev, 'config']));
                                        }}
                                        className="px-3 py-1.5 text-xs rounded bg-blue-100 hover:bg-blue-200 border border-blue-300 transition-colors"
                                    >
                                        ACL4SSR Online Full
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setFormData(prev => ({ ...prev, config: 'https://raw.githubusercontent.com/ACL4SSR/ACL4SSR/master/Clash/config/ACL4SSR_Online_Mini.ini' }));
                                            setSetFields(prev => new Set([...prev, 'config']));
                                        }}
                                        className="px-3 py-1.5 text-xs rounded bg-blue-100 hover:bg-blue-200 border border-blue-300 transition-colors"
                                    >
                                        ACL4SSR Online Mini
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setFormData(prev => ({ ...prev, config: 'https://raw.githubusercontent.com/DivineEngine/Profiles/master/Clash/config/China.yaml' }));
                                            setSetFields(prev => new Set([...prev, 'config']));
                                        }}
                                        className="px-3 py-1.5 text-xs rounded bg-blue-100 hover:bg-blue-200 border border-blue-300 transition-colors"
                                    >
                                        Divine Engine China
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setFormData(prev => ({ ...prev, config: 'https://gist.githubusercontent.com/tindy2013/1fa08640a9088ac8652dbd40c5d2715b/raw/loon_simple.conf' }));
                                            setSetFields(prev => new Set([...prev, 'config']));
                                        }}
                                        className="px-3 py-1.5 text-xs rounded bg-blue-100 hover:bg-blue-200 border border-blue-300 transition-colors"
                                    >
                                        Loon Simple
                                    </button>
                                </div>
                                <input
                                    type="text"
                                    id="config"
                                    name="config"
                                    value={formData.config ?? ''}
                                    onChange={handleInputChange}
                                    className={getInputClass("config")}
                                    placeholder="Enter a URL or select a preset above"
                                />
                                <p className="mt-1 text-xs text-gray-600">URL for ruleset configuration or local path (if server allows)</p>
                            </div>
                        </div>
                    </div>
                </fieldset>

                {/* Filtering & Renaming Section */}
                <fieldset className="p-4 border rounded-md">
                    <legend className={fieldsetLegendClass}>Filtering & Renaming</legend>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div>
                            <FieldLabel htmlFor="include" fieldName="include">Include Remarks (Regex)</FieldLabel>
                            <input
                                type="text"
                                id="include"
                                name="include"
                                value={formData.include ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("include")}
                                placeholder="e.g., HK|SG"
                            />
                        </div>
                        <div>
                            <FieldLabel htmlFor="exclude" fieldName="exclude">Exclude Remarks (Regex)</FieldLabel>
                            <input
                                type="text"
                                id="exclude"
                                name="exclude"
                                value={formData.exclude ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("exclude")}
                                placeholder="e.g., expire|disabled"
                            />
                        </div>
                        <div>
                            <FieldLabel htmlFor="rename" fieldName="rename">Rename Nodes</FieldLabel>
                            <textarea
                                id="rename"
                                name="rename"
                                value={formData.rename ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("rename")}
                                rows={2}
                                placeholder="oldName@newName`anotherOld@anotherNew"
                            />
                            <p className="mt-1 text-xs text-gray-500">Use ` to separate multiple rules, @ as delimiter.</p>
                        </div>
                        <div className="space-y-2">
                            <FieldLabel htmlFor="emoji" fieldName="emoji">Emoji Handling</FieldLabel>
                            <div className="flex items-center space-x-4">
                                <div className="flex items-center">
                                    <input
                                        id="emoji"
                                        name="emoji"
                                        type="checkbox"
                                        checked={formData.emoji}
                                        onChange={handleInputChange}
                                        className={checkboxClass}
                                    />
                                    <label htmlFor="emoji" className="ml-2 block text-sm text-gray-900">Add & Remove Old Emoji</label>
                                </div>
                            </div>
                            <div className="flex items-center space-x-4 pl-4">
                                <div className="flex items-center">
                                    <input
                                        id="add_emoji"
                                        name="add_emoji"
                                        type="checkbox"
                                        checked={formData.add_emoji}
                                        onChange={handleInputChange}
                                        className={checkboxClass}
                                        disabled={formData.emoji}
                                    />
                                    <label htmlFor="add_emoji" className="ml-2 block text-sm text-gray-900">Add Emoji (Only)</label>
                                </div>
                                <div className="flex items-center">
                                    <input
                                        id="remove_emoji"
                                        name="remove_emoji"
                                        type="checkbox"
                                        checked={formData.remove_emoji}
                                        onChange={handleInputChange}
                                        className={checkboxClass}
                                        disabled={formData.emoji}
                                    />
                                    <label htmlFor="remove_emoji" className="ml-2 block text-sm text-gray-900">Remove Old Emoji (Only)</label>
                                </div>
                            </div>
                            <p className="mt-1 text-xs text-gray-500 pl-4">Checking the first option overrides the specifics.</p>
                        </div>
                        <div className="flex items-center">
                            <FieldLabel htmlFor="fdn" fieldName="fdn">Filter Deprecated Nodes</FieldLabel>
                            <input
                                id="fdn"
                                name="fdn"
                                type="checkbox"
                                checked={formData.fdn}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                    </div>
                </fieldset>

                {/* Output Options Section */}
                <fieldset className="p-4 border rounded-md">
                    <legend className={fieldsetLegendClass}>Output Options</legend>
                    <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                        {/* Surge Specific */}
                        {formData.target === 'surge' && (
                            <div className="col-span-1">
                                <FieldLabel htmlFor="ver" fieldName="ver">Surge Version</FieldLabel>
                                <input
                                    type="number"
                                    id="ver"
                                    name="ver"
                                    value={formData.ver ?? ''}
                                    onChange={handleInputChange}
                                    min="2"
                                    max="4"
                                    className={getInputClass("ver")}
                                />
                            </div>
                        )}
                        {/* Clash Specific */}
                        {(formData.target === 'clash' || formData.target === 'clashr') && (
                            <>
                                <div className="flex items-center">
                                    <FieldLabel htmlFor="new_name" fieldName="new_name">Clash: New Field Names</FieldLabel>
                                    <input
                                        id="new_name"
                                        name="new_name"
                                        type="checkbox"
                                        checked={formData.new_name}
                                        onChange={handleInputChange}
                                        className={checkboxClass}
                                    />
                                </div>
                                <div className="flex items-center">
                                    <FieldLabel htmlFor="script" fieldName="script">Clash: Enable Scripting</FieldLabel>
                                    <input
                                        id="script"
                                        name="script"
                                        type="checkbox"
                                        checked={formData.script}
                                        onChange={handleInputChange}
                                        className={checkboxClass}
                                    />
                                </div>
                                <div className="flex items-center">
                                    <FieldLabel htmlFor="classic" fieldName="classic">Clash: Classic Ruleset</FieldLabel>
                                    <input
                                        id="classic"
                                        name="classic"
                                        type="checkbox"
                                        checked={formData.classic}
                                        onChange={handleInputChange}
                                        className={checkboxClass}
                                    />
                                </div>
                            </>
                        )}
                        <div className="flex items-center">
                            <FieldLabel htmlFor="append_type" fieldName="append_type">Append Proxy Type</FieldLabel>
                            <input
                                id="append_type"
                                name="append_type"
                                type="checkbox"
                                checked={formData.append_type}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                        <div className="flex items-center">
                            <FieldLabel htmlFor="list" fieldName="list">Node List Only</FieldLabel>
                            <input
                                id="list"
                                name="list"
                                type="checkbox"
                                checked={formData.list}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                        <div className="flex items-center">
                            <FieldLabel htmlFor="sort" fieldName="sort">Sort Nodes</FieldLabel>
                            <input
                                id="sort"
                                name="sort"
                                type="checkbox"
                                checked={formData.sort}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                        <div className="flex items-center">
                            <FieldLabel htmlFor="rename_node" fieldName="rename_node">Enable Rule Generator Rename</FieldLabel>
                            <input
                                id="rename_node"
                                name="rename_node"
                                type="checkbox"
                                checked={formData.rename_node}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                        <div className="flex items-center">
                            <FieldLabel htmlFor="expand" fieldName="expand">Expand Rulesets (Clash)</FieldLabel>
                            <input
                                id="expand"
                                name="expand"
                                type="checkbox"
                                checked={formData.expand}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                    </div>
                </fieldset>

                {/* Protocol Flags Section */}
                <fieldset className="p-4 border rounded-md">
                    <legend className={fieldsetLegendClass}>Protocol Flags</legend>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div className="flex items-center">
                            <FieldLabel htmlFor="tfo" fieldName="tfo">TCP Fast Open</FieldLabel>
                            <input
                                id="tfo"
                                name="tfo"
                                type="checkbox"
                                checked={formData.tfo}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                        <div className="flex items-center">
                            <FieldLabel htmlFor="udp" fieldName="udp">UDP Relay</FieldLabel>
                            <input
                                id="udp"
                                name="udp"
                                type="checkbox"
                                checked={formData.udp}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                        <div className="flex items-center">
                            <FieldLabel htmlFor="scv" fieldName="scv">Skip Cert Verify</FieldLabel>
                            <input
                                id="scv"
                                name="scv"
                                type="checkbox"
                                checked={formData.scv}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                        <div className="flex items-center">
                            <FieldLabel htmlFor="tls13" fieldName="tls13">Enable TLS 1.3</FieldLabel>
                            <input
                                id="tls13"
                                name="tls13"
                                type="checkbox"
                                checked={formData.tls13}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                    </div>
                </fieldset>

                {/* Advanced Section */}
                <fieldset className="p-4 border rounded-md">
                    <legend className={fieldsetLegendClass}>Advanced</legend>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div>
                            <FieldLabel htmlFor="group" fieldName="group">Custom Group Name</FieldLabel>
                            <input
                                type="text"
                                id="group"
                                name="group"
                                value={formData.group ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("group")}
                            />
                        </div>
                        <div>
                            <FieldLabel htmlFor="groups" fieldName="groups">Custom Proxy Groups</FieldLabel>
                            <textarea
                                id="groups"
                                name="groups"
                                value={formData.groups ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("groups")}
                                rows={2}
                                placeholder="[Proxy Group]`..."
                            />
                            <p className="mt-1 text-xs text-gray-500">INI-like format, separated by @.</p>
                        </div>
                        <div>
                            <FieldLabel htmlFor="ruleset" fieldName="ruleset">Custom Ruleset</FieldLabel>
                            <textarea
                                id="ruleset"
                                name="ruleset"
                                value={formData.ruleset ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("ruleset")}
                                rows={2}
                                placeholder="[Rule]`..."
                            />
                            <p className="mt-1 text-xs text-gray-500">INI-like format, separated by @.</p>
                        </div>
                        <div className="flex items-center space-x-4">
                            <FieldLabel htmlFor="insert" fieldName="insert">Insert Nodes (from server config)</FieldLabel>
                            <input
                                id="insert"
                                name="insert"
                                type="checkbox"
                                checked={formData.insert}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                        <div className="flex items-center space-x-4">
                            <FieldLabel htmlFor="prepend" fieldName="prepend">Prepend Inserted Nodes</FieldLabel>
                            <input
                                id="prepend"
                                name="prepend"
                                type="checkbox"
                                checked={formData.prepend}
                                onChange={handleInputChange}
                                className={checkboxClass}
                                disabled={!formData.insert}
                            />
                        </div>
                        <div>
                            <FieldLabel htmlFor="interval" fieldName="interval">Update Interval (seconds)</FieldLabel>
                            <input
                                type="number"
                                id="interval"
                                name="interval"
                                value={formData.interval ?? ''}
                                onChange={handleInputChange}
                                min="0"
                                className={getInputClass("interval")}
                            />
                        </div>
                        <div className="flex items-center">
                            <FieldLabel htmlFor="strict" fieldName="strict">Strict Update Mode</FieldLabel>
                            <input
                                id="strict"
                                name="strict"
                                type="checkbox"
                                checked={formData.strict}
                                onChange={handleInputChange}
                                className={checkboxClass}
                            />
                        </div>
                        <div>
                            <FieldLabel htmlFor="sort_script" fieldName="sort_script">Sort Script URL/Path</FieldLabel>
                            <input
                                type="text"
                                id="sort_script"
                                name="sort_script"
                                value={formData.sort_script ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("sort_script")}
                            />
                        </div>
                        <div>
                            <FieldLabel htmlFor="filter" fieldName="filter">Filter Script URL/Path</FieldLabel>
                            <input
                                type="text"
                                id="filter"
                                name="filter"
                                value={formData.filter ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("filter")}
                            />
                        </div>
                        <div>
                            <FieldLabel htmlFor="dev_id" fieldName="dev_id">Device ID</FieldLabel>
                            <input
                                type="text"
                                id="dev_id"
                                name="dev_id"
                                value={formData.dev_id ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("dev_id")}
                            />
                        </div>
                        <div>
                            <FieldLabel htmlFor="token" fieldName="token">API Token (if required)</FieldLabel>
                            <input
                                type="password"
                                id="token"
                                name="token"
                                value={formData.token ?? ''}
                                onChange={handleInputChange}
                                className={getInputClass("token")}
                            />
                        </div>
                    </div>
                </fieldset>

                {/* Submission Button */}
                <div className="flex justify-between items-center">
                    <div className="text-sm text-gray-500">
                        Fields marked as <span className="text-green-600">(set)</span> will be included in the request.
                    </div>
                    <div className="flex items-center gap-4">
                        <div className="flex items-center">
                            <input
                                id="saveApiUrl"
                                type="checkbox"
                                checked={saveApiUrl}
                                onChange={(e) => setSaveApiUrl(e.target.checked)}
                                className={checkboxClass}
                            />
                            <label htmlFor="saveApiUrl" className="ml-2 text-sm text-gray-700">
                                Save as subscription
                            </label>
                        </div>
                        <button
                            type="submit"
                            disabled={isSubmitDisabled}
                            className={buttonClass}
                        >
                            {isLoading ? 'Generating...' : 'Generate Configuration'}
                        </button>
                    </div>
                </div>
            </form>

            {/* Results Section */}
            <div className="mt-8">
                {isLoading && (
                    <div className="text-center p-4">
                        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-500 mx-auto"></div>
                        <p className="mt-2 text-sm text-gray-600">Processing...</p>
                    </div>
                )}

                {error && (
                    <div className="p-4 border border-red-400 bg-red-50 rounded-md">
                        <h3 className="text-lg font-semibold text-red-800">Error</h3>
                        <p className="text-red-700">{error.error}</p>
                        {error.details && <p className="mt-1 text-sm text-red-600">{error.details}</p>}
                    </div>
                )}

                {result && !error && (
                    <div className="p-4 border border-green-400 bg-green-50 rounded-md">
                        <h3 className="text-lg font-semibold text-green-800">Result</h3>
                        <p className="text-sm text-gray-600 mb-2">Content-Type: {result.content_type}</p>

                        {/* API URL Display */}
                        <div className="mb-4 p-3 bg-white border border-gray-300 rounded-md">
                            <div className="flex justify-between items-center mb-2">
                                <h4 className="font-medium text-gray-800">Subscription URL</h4>
                                <button
                                    onClick={() => navigator.clipboard.writeText(generateApiUrl())}
                                    className="text-xs px-2 py-1 bg-gray-200 hover:bg-gray-300 rounded"
                                >
                                    Copy
                                </button>
                            </div>
                            <p className="text-xs break-all font-mono bg-gray-50 p-2 rounded border border-gray-200">
                                {generateApiUrl()}
                            </p>
                            <p className="text-xs text-gray-500 mt-1">
                                You can use this URL directly as a subscription link.
                                {saveApiUrl && " This URL will be saved for later use."}
                            </p>
                        </div>

                        <textarea
                            readOnly
                            value={result.content}
                            rows={15}
                            className="w-full p-2 border border-gray-300 rounded-md font-mono text-sm bg-gray-50"
                            aria-label="Conversion Result"
                        />
                        <div className="mt-4 flex justify-end">
                            <button
                                onClick={handleDownload}
                                className={buttonClass}
                            >
                                Download Config
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
} 