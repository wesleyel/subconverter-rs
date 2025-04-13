"use client";

import { useState } from "react";
import Link from "next/link";

export default function ConfigEditor() {
    const [activeTab, setActiveTab] = useState("general");
    const [configName, setConfigName] = useState("");
    const [generatedLink, setGeneratedLink] = useState("");

    const handleSaveConfig = () => {
        // Generate a unique ID for the config
        const configId = Math.random().toString(36).substring(2, 15);
        const baseUrl = window.location.origin;
        const link = `${baseUrl}/api/subconverter?config=${configId}`;

        setGeneratedLink(link);

        // In a real implementation, this would save the config to a database
        console.log("Config saved:", configName);
    };

    return (
        <main className="flex min-h-screen flex-col items-center p-8">
            <div className="z-10 max-w-5xl w-full items-center font-mono text-sm">
                <div className="flex justify-between items-center mb-8">
                    <h1 className="text-3xl font-bold">Config Editor</h1>
                    <Link
                        href="/"
                        className="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded"
                    >
                        Back to Home
                    </Link>
                </div>

                <div className="bg-white/5 p-6 rounded-lg shadow-md mb-8">
                    <div className="flex mb-4">
                        <input
                            type="text"
                            value={configName}
                            onChange={(e) => setConfigName(e.target.value)}
                            placeholder="Config Name"
                            className="w-full p-2 border border-gray-300 rounded-l bg-white/10"
                        />
                        <button
                            onClick={handleSaveConfig}
                            className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-r"
                        >
                            Save Config
                        </button>
                    </div>

                    {generatedLink && (
                        <div className="mt-4 p-4 bg-gray-800 rounded">
                            <p className="text-sm mb-2">Your generated link:</p>
                            <div className="flex">
                                <input
                                    type="text"
                                    readOnly
                                    value={generatedLink}
                                    className="w-full p-2 bg-gray-700 rounded-l"
                                />
                                <button
                                    onClick={() => navigator.clipboard.writeText(generatedLink)}
                                    className="bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded-r"
                                >
                                    Copy
                                </button>
                            </div>
                        </div>
                    )}
                </div>

                <div className="bg-white/5 rounded-lg shadow-md">
                    <div className="flex border-b border-gray-700">
                        <button
                            className={`px-4 py-2 ${activeTab === "general"
                                ? "bg-blue-600 text-white"
                                : "bg-transparent"
                                }`}
                            onClick={() => setActiveTab("general")}
                        >
                            General
                        </button>
                        <button
                            className={`px-4 py-2 ${activeTab === "proxy"
                                ? "bg-blue-600 text-white"
                                : "bg-transparent"
                                }`}
                            onClick={() => setActiveTab("proxy")}
                        >
                            Proxy Settings
                        </button>
                        <button
                            className={`px-4 py-2 ${activeTab === "rules"
                                ? "bg-blue-600 text-white"
                                : "bg-transparent"
                                }`}
                            onClick={() => setActiveTab("rules")}
                        >
                            Rules
                        </button>
                        <button
                            className={`px-4 py-2 ${activeTab === "advanced"
                                ? "bg-blue-600 text-white"
                                : "bg-transparent"
                                }`}
                            onClick={() => setActiveTab("advanced")}
                        >
                            Advanced
                        </button>
                    </div>

                    <div className="p-6">
                        {activeTab === "general" && (
                            <div className="space-y-4">
                                <div>
                                    <label htmlFor="target" className="block text-sm font-medium mb-1">
                                        Target Format
                                    </label>
                                    <select
                                        id="target"
                                        className="w-full p-2 border border-gray-300 rounded bg-white/10"
                                    >
                                        <option value="clash">Clash</option>
                                        <option value="surge">Surge</option>
                                        <option value="quantumult">Quantumult</option>
                                        <option value="quanx">Quantumult X</option>
                                        <option value="loon">Loon</option>
                                        <option value="ss">SS</option>
                                        <option value="ssr">SSR</option>
                                        <option value="v2ray">V2Ray</option>
                                    </select>
                                </div>

                                <div>
                                    <label htmlFor="subscriptionUrl" className="block text-sm font-medium mb-1">
                                        Subscription URL
                                    </label>
                                    <input
                                        type="url"
                                        id="subscriptionUrl"
                                        placeholder="https://example.com/subscription"
                                        className="w-full p-2 border border-gray-300 rounded bg-white/10"
                                    />
                                </div>

                                <div>
                                    <label htmlFor="includedTypes" className="block text-sm font-medium mb-1">
                                        Include Proxy Types
                                    </label>
                                    <div className="grid grid-cols-2 gap-2">
                                        <div className="flex items-center">
                                            <input type="checkbox" id="typeSSR" className="mr-2" />
                                            <label htmlFor="typeSSR">SSR</label>
                                        </div>
                                        <div className="flex items-center">
                                            <input type="checkbox" id="typeSS" className="mr-2" />
                                            <label htmlFor="typeSS">SS</label>
                                        </div>
                                        <div className="flex items-center">
                                            <input type="checkbox" id="typeVMess" className="mr-2" />
                                            <label htmlFor="typeVMess">VMess</label>
                                        </div>
                                        <div className="flex items-center">
                                            <input type="checkbox" id="typeTrojan" className="mr-2" />
                                            <label htmlFor="typeTrojan">Trojan</label>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}

                        {activeTab === "proxy" && (
                            <div className="space-y-4">
                                <p>Proxy configuration settings go here.</p>
                            </div>
                        )}

                        {activeTab === "rules" && (
                            <div className="space-y-4">
                                <p>Rule configuration settings go here.</p>
                            </div>
                        )}

                        {activeTab === "advanced" && (
                            <div className="space-y-4">
                                <p className="mb-4">Advanced configuration settings:</p>

                                <div className="space-y-4 mb-6">
                                    <div>
                                        <label htmlFor="updateInterval" className="block text-sm font-medium mb-1">
                                            Update Interval (seconds)
                                        </label>
                                        <input
                                            type="number"
                                            id="updateInterval"
                                            placeholder="86400"
                                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                                        />
                                    </div>

                                    <div>
                                        <label htmlFor="strictMode" className="block text-sm font-medium mb-1">
                                            Strict Mode
                                        </label>
                                        <select
                                            id="strictMode"
                                            className="w-full p-2 border border-gray-300 rounded bg-white/10"
                                        >
                                            <option value="false">Disabled</option>
                                            <option value="true">Enabled</option>
                                        </select>
                                    </div>
                                </div>

                                <div className="border-t border-gray-700 pt-4">
                                    <h3 className="text-md font-medium mb-2">Admin Tools</h3>
                                    <div className="grid grid-cols-1 gap-2">
                                        <Link
                                            href="/admin"
                                            className="flex items-center bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded"
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                                <path fillRule="evenodd" d="M2 5a2 2 0 012-2h12a2 2 0 012 2v10a2 2 0 01-2 2H4a2 2 0 01-2-2V5zm3.293 1.293a1 1 0 011.414 0l3 3a1 1 0 010 1.414l-3 3a1 1 0 01-1.414-1.414L7.586 10 5.293 7.707a1 1 0 010-1.414zM11 12a1 1 0 100 2h3a1 1 0 100-2h-3z" clipRule="evenodd" />
                                            </svg>
                                            File Browser & Editor
                                        </Link>
                                        <button
                                            className="flex items-center bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded"
                                            onClick={() => window.open('/api/test/wasm', '_blank')}
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                                                <path fillRule="evenodd" d="M6.267 3.455a3.066 3.066 0 001.745-.723 3.066 3.066 0 013.976 0 3.066 3.066 0 001.745.723 3.066 3.066 0 012.812 2.812c.051.643.304 1.254.723 1.745a3.066 3.066 0 010 3.976 3.066 3.066 0 00-.723 1.745 3.066 3.066 0 01-2.812 2.812 3.066 3.066 0 00-1.745.723 3.066 3.066 0 01-3.976 0 3.066 3.066 0 00-1.745-.723 3.066 3.066 0 01-2.812-2.812 3.066 3.066 0 00-.723-1.745 3.066 3.066 0 010-3.976 3.066 3.066 0 00.723-1.745 3.066 3.066 0 012.812-2.812zm7.44 5.252a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                                            </svg>
                                            Test WASM Status
                                        </button>
                                    </div>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </main>
    );
} 