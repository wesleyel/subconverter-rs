"use client";

import { useState } from "react";
import Link from "next/link";

// Dummy data for demonstration
const dummyLinks = [
    {
        id: "1",
        name: "My Clash Config",
        url: "/api/subconverter?config=abc123",
        target: "clash",
        createdAt: "2023-06-15",
    },
    {
        id: "2",
        name: "Quantumult X Setup",
        url: "/api/subconverter?config=def456",
        target: "quanx",
        createdAt: "2023-07-20",
    },
];

export default function SavedLinks() {
    const [links, setLinks] = useState(dummyLinks);

    const handleDelete = (id: string) => {
        setLinks(links.filter((link) => link.id !== id));
    };

    return (
        <main className="flex min-h-screen flex-col items-center p-8">
            <div className="z-10 max-w-5xl w-full items-center font-mono text-sm">
                <div className="flex justify-between items-center mb-8">
                    <h1 className="text-3xl font-bold">My Saved Links</h1>
                    <Link
                        href="/"
                        className="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded"
                    >
                        Back to Home
                    </Link>
                </div>

                <div className="bg-white/5 p-6 rounded-lg shadow-md">
                    {links.length > 0 ? (
                        <div className="space-y-4">
                            {links.map((link) => (
                                <div
                                    key={link.id}
                                    className="border border-gray-700 rounded-lg p-4 flex flex-col md:flex-row md:items-center justify-between"
                                >
                                    <div className="mb-4 md:mb-0">
                                        <h3 className="text-xl font-semibold">{link.name}</h3>
                                        <p className="text-sm text-gray-400">
                                            Format: {link.target} • Created: {link.createdAt}
                                        </p>
                                    </div>
                                    <div className="flex flex-col md:flex-row gap-2">
                                        <button
                                            onClick={() => navigator.clipboard.writeText(`${window.location.origin}${link.url}`)}
                                            className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                                        >
                                            Copy Link
                                        </button>
                                        <button
                                            onClick={() => handleDelete(link.id)}
                                            className="bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded"
                                        >
                                            Delete
                                        </button>
                                    </div>
                                </div>
                            ))}
                        </div>
                    ) : (
                        <div className="text-center py-8">
                            <p className="text-lg mb-4">You don't have any saved links yet.</p>
                            <Link
                                href="/config"
                                className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                            >
                                Create Your First Config
                            </Link>
                        </div>
                    )}
                </div>
            </div>
        </main>
    );
} 