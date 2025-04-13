"use client";

import { useState, useEffect } from "react";

interface ApiEndpoint {
    name: string;
    path: string;
    method: string;
    description: string;
    params?: {
        name: string;
        type: string;
        required: boolean;
        description: string;
    }[];
}

export default function DebugPanel() {
    const [isOpen, setIsOpen] = useState(false);
    const [endpoints] = useState<ApiEndpoint[]>([
        {
            name: "Convert Subscription",
            path: "/api/subconverter",
            method: "GET",
            description: "Convert a subscription to a different format",
            params: [
                {
                    name: "url",
                    type: "string",
                    required: true,
                    description: "URL of the subscription to convert"
                },
                {
                    name: "target",
                    type: "string",
                    required: false,
                    description: "Target format (clash, surge, etc.)"
                }
            ]
        },
        {
            name: "WASM Status",
            path: "/api/debug/wasm",
            method: "GET",
            description: "Check WASM module initialization and status"
        },
        {
            name: "Directory List",
            path: "/api/admin/list",
            method: "GET",
            description: "List files and directories",
            params: [
                {
                    name: "path",
                    type: "string",
                    required: false,
                    description: "Path to list (defaults to root)"
                }
            ]
        },
        {
            name: "File Operations",
            path: "/api/admin/file",
            method: "GET/POST/DELETE",
            description: "Read, write, or delete files",
            params: [
                {
                    name: "path",
                    type: "string",
                    required: true,
                    description: "File path"
                },
                {
                    name: "content",
                    type: "string",
                    required: false,
                    description: "Content to write (POST only)"
                }
            ]
        }
    ]);

    const [isProd, setIsProd] = useState(true);
    const [netlifyEnv, setNetlifyEnv] = useState<string | null>(null);

    useEffect(() => {
        // Check if we're in production by looking at environment variables
        const context = window.location.hostname === "localhost" ? "dev" :
            window.location.hostname.includes("netlify.app") ? "deploy-preview" : "production";

        setIsProd(context === "production");
        setNetlifyEnv(context);
    }, []);

    // Don't render anything in production
    if (isProd) return null;

    return (
        <div className="fixed bottom-0 right-0 z-50 bg-gray-800 text-white rounded-tl-lg shadow-lg max-w-xl">
            <button
                onClick={() => setIsOpen(!isOpen)}
                className="w-full flex justify-between items-center p-2 font-bold"
            >
                <span>🐞 Debug Panel {netlifyEnv ? `(${netlifyEnv})` : ""}</span>
                <span>{isOpen ? "▼" : "▲"}</span>
            </button>

            {isOpen && (
                <div className="p-4 overflow-auto max-h-[60vh]">
                    <h3 className="text-lg font-bold mb-4">API Endpoints</h3>

                    <div className="space-y-4">
                        {endpoints.map((endpoint, index) => (
                            <div key={index} className="border border-gray-600 rounded p-3">
                                <div className="flex justify-between">
                                    <h4 className="font-bold">{endpoint.name}</h4>
                                    <span className="bg-blue-600 text-xs px-2 py-1 rounded">{endpoint.method}</span>
                                </div>
                                <p className="text-gray-400 text-sm my-1">{endpoint.description}</p>
                                <code className="bg-gray-700 p-1 text-xs block my-2 rounded">{endpoint.path}</code>

                                {endpoint.params && endpoint.params.length > 0 && (
                                    <div className="mt-2">
                                        <h5 className="text-sm font-semibold">Parameters:</h5>
                                        <ul className="text-xs ml-4 list-disc">
                                            {endpoint.params.map((param, pidx) => (
                                                <li key={pidx}>
                                                    <span className="font-mono">{param.name}</span>
                                                    <span className="text-gray-400"> ({param.type}{param.required ? ", required" : ""}): </span>
                                                    <span>{param.description}</span>
                                                </li>
                                            ))}
                                        </ul>
                                    </div>
                                )}

                                <button
                                    onClick={() => window.open(`${window.location.origin}${endpoint.path}`, "_blank")}
                                    className="mt-2 bg-gray-700 hover:bg-gray-600 text-xs px-2 py-1 rounded"
                                >
                                    Test Endpoint
                                </button>
                            </div>
                        ))}
                    </div>

                    <div className="mt-4 pt-4 border-t border-gray-600">
                        <h3 className="text-lg font-bold mb-2">Environment Info</h3>
                        <pre className="bg-gray-700 p-2 rounded text-xs overflow-x-auto">
                            {`Netlify Environment: ${netlifyEnv || "Unknown"}
Node Environment: ${process.env.NODE_ENV || "Unknown"}
Base URL: ${window.location.origin}`}
                        </pre>
                    </div>
                </div>
            )}
        </div>
    );
} 