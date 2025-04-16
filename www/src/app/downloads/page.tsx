"use client";

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { getAvailableDownloads, getDownloadUrl, AppDownloadInfo } from '@/lib/api-client';

export default function DownloadsPage() {
    const [downloads, setDownloads] = useState<AppDownloadInfo[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [activeFilter, setActiveFilter] = useState<string>('all');

    useEffect(() => {
        loadDownloads();
    }, []);

    const loadDownloads = async () => {
        setIsLoading(true);
        setError(null);
        try {
            const data = await getAvailableDownloads();
            setDownloads(data);
        } catch (err) {
            setError(`Failed to load downloads: ${err instanceof Error ? err.message : String(err)}`);
            console.error("Error loading downloads:", err);
        } finally {
            setIsLoading(false);
        }
    };

    const formatFileSize = (bytes: number): string => {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
        if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
        return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
    };

    const formatDate = (dateString: string): string => {
        const date = new Date(dateString);
        return date.toLocaleDateString(undefined, {
            year: 'numeric',
            month: 'short',
            day: 'numeric'
        });
    };

    // Demo data since we don't have a real API yet
    const demoDownloads: AppDownloadInfo[] = [
        {
            name: 'Clash for Windows',
            version: 'v0.20.39',
            platform: 'windows',
            size: 26314752, // 25.1 MB
            download_url: '',
            release_date: '2023-12-15',
            description: 'A Windows GUI based on Clash.'
        },
        {
            name: 'Clash for Windows',
            version: 'v0.20.39',
            platform: 'macos',
            size: 27262976, // 26 MB
            download_url: '',
            release_date: '2023-12-15',
            description: 'A macOS GUI based on Clash.'
        },
        {
            name: 'ClashX',
            version: 'v1.118.0',
            platform: 'macos',
            size: 15728640, // 15 MB
            download_url: '',
            release_date: '2023-11-30',
            description: 'A rule-based custom proxy for macOS based on Clash.'
        },
        {
            name: 'Clash for Android',
            version: 'v2.5.12',
            platform: 'android',
            size: 10485760, // 10 MB
            download_url: '',
            release_date: '2023-12-10',
            description: 'A rule-based custom proxy for Android based on Clash.'
        },
        {
            name: 'Clash Meta',
            version: 'v1.16.0',
            platform: 'linux',
            size: 8388608, // 8 MB
            download_url: '',
            release_date: '2023-12-05',
            description: 'A rule-based custom proxy with more advanced features.'
        },
        {
            name: 'SingBox Core',
            version: 'v1.7.0',
            platform: 'windows',
            size: 12582912, // 12 MB
            download_url: '',
            release_date: '2023-12-20',
            description: 'Universal proxy platform for Windows.'
        },
        {
            name: 'SingBox Core',
            version: 'v1.7.0',
            platform: 'macos',
            size: 13631488, // 13 MB
            download_url: '',
            release_date: '2023-12-20',
            description: 'Universal proxy platform for macOS.'
        },
        {
            name: 'SingBox Core',
            version: 'v1.7.0',
            platform: 'linux',
            size: 12058624, // 11.5 MB
            download_url: '',
            release_date: '2023-12-20',
            description: 'Universal proxy platform for Linux.'
        },
        {
            name: 'SingBox for Android',
            version: 'v1.7.0',
            platform: 'android',
            size: 10485760, // 10 MB
            download_url: '',
            release_date: '2023-12-20',
            description: 'SingBox client for Android.'
        }
    ];

    const displayDownloads = isLoading ? [] : (downloads.length > 0 ? downloads : demoDownloads);

    const filteredDownloads = activeFilter === 'all'
        ? displayDownloads
        : displayDownloads.filter(d => d.platform === activeFilter);

    const platformGroups = filteredDownloads.reduce((groups, download) => {
        if (!groups[download.name]) {
            groups[download.name] = [];
        }
        groups[download.name].push(download);
        return groups;
    }, {} as Record<string, AppDownloadInfo[]>);

    const getPlatformIcon = (platform: string) => {
        switch (platform) {
            case 'windows':
                return (
                    <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M0 0h24v24H0V0z" fill="none" />
                        <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z" />
                    </svg>
                );
            case 'macos':
                return (
                    <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M21.928 11.607c-.202-.488-.635-.605-.928-.633v-1.898c0-.362-.183-.474-.333-.535-.149-.061-.395-.079-.667.039l-1.273.561c-.393.188-.857.349-1.344.349h-6.7c-.33 0-.595-.437-.781-.86-.781-1.647-1.145-5.007-1.396-6.157-.25-1.15-.908-1.15-1.434-1.15-.511 0-1.07.02-1.07 1.243 0 1.3.993 9.75 4.738 9.75h5.27c.874 0 1.245 1.697.254 1.697h-5.523c-2.689 0-3.387 2.016-3.387 2.016s5.077 3.695 5.077 4.806v1.903c0 .29.34.333.676.333h1.126c.337 0 .676-.043.676-.333v-.862c0-.292.406-.454.676-.454h1.126c.271 0 .677.162.677.454v.862c0 .29.339.333.675.333h1.126c.336 0 .676-.043.676-.333v-.862c0-.292.405-.454.676-.454h1.126c.27 0 .676.162.676.454v.862c0 .29.34.333.676.333h1.126c.337 0 .676-.043.676-.333v-4.85c0-1.263-1.375-3.115-2.071-3.668z" />
                    </svg>
                );
            case 'linux':
                return (
                    <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M21 18v3H3V3h18v3H10v12h11zm-9-2V8H3v8h9z" />
                    </svg>
                );
            case 'android':
                return (
                    <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M17.523 15.3414c-.5511 0-.9993-.4486-.9993-.9997s.4483-.9993.9993-.9993c.5511 0 .9993.4483.9993.9993.0001.5511-.4482.9997-.9993.9997m-11.046 0c-.5511 0-.9993-.4486-.9993-.9997s.4482-.9993.9993-.9993c.5511 0 .9993.4483.9993.9993 0 .5511-.4483.9997-.9993.9997m11.4045-6.02l1.9973-3.4592a.416.416 0 00-.1521-.5676.416.416 0 00-.5676.1521l-2.0223 3.503C15.5902 8.2439 13.8533 7.8508 12 7.8508s-3.5902.3931-5.1367 1.0989L4.84 5.4467a.4161.4161 0 00-.5677-.1521.4157.4157 0 00-.1521.5676l1.9973 3.4592C2.6889 11.1867.3432 14.6589 0 18.761h24c-.3435-4.1021-2.6892-7.5743-6.0775-9.4396" />
                    </svg>
                );
            default:
                return (
                    <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M17,10.5V7A1,1 0 0,0 16,6H4A1,1 0 0,0 3,7V17A1,1 0 0,0 4,18H16A1,1 0 0,0 17,17V13.5L21,17.5V6.5L17,10.5Z" />
                    </svg>
                );
        }
    };

    return (
        <main className="flex min-h-screen flex-col items-center justify-between p-4 md:p-8 lg:p-24">
            <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm">
                <div className="flex flex-col sm:flex-row items-center justify-between mb-6">
                    <h1 className="text-4xl font-bold mb-4 sm:mb-0 text-center">App Downloads</h1>
                    <Link
                        href="/"
                        className="flex items-center gap-2 bg-gray-800 hover:bg-gray-700 text-white px-4 py-2 rounded-lg transition-colors"
                    >
                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path>
                        </svg>
                        <span>Back to Home</span>
                    </Link>
                </div>

                {error && (
                    <div className="mb-4 p-4 border border-red-400 bg-red-50 text-red-700 rounded-md">
                        {error}
                    </div>
                )}

                <div className="bg-white/5 p-6 rounded-lg shadow-md mb-8">
                    <div className="mb-6">
                        <h2 className="text-2xl font-semibold mb-2">Platform Filter</h2>
                        <div className="flex flex-wrap gap-2">
                            <button
                                onClick={() => setActiveFilter('all')}
                                className={`px-4 py-2 rounded-md transition-colors ${activeFilter === 'all'
                                    ? 'bg-blue-600 text-white'
                                    : 'bg-gray-200 hover:bg-gray-300 text-gray-800'
                                    }`}
                            >
                                All Platforms
                            </button>
                            <button
                                onClick={() => setActiveFilter('windows')}
                                className={`flex items-center gap-1 px-4 py-2 rounded-md transition-colors ${activeFilter === 'windows'
                                    ? 'bg-blue-600 text-white'
                                    : 'bg-gray-200 hover:bg-gray-300 text-gray-800'
                                    }`}
                            >
                                {getPlatformIcon('windows')} Windows
                            </button>
                            <button
                                onClick={() => setActiveFilter('macos')}
                                className={`flex items-center gap-1 px-4 py-2 rounded-md transition-colors ${activeFilter === 'macos'
                                    ? 'bg-blue-600 text-white'
                                    : 'bg-gray-200 hover:bg-gray-300 text-gray-800'
                                    }`}
                            >
                                {getPlatformIcon('macos')} macOS
                            </button>
                            <button
                                onClick={() => setActiveFilter('linux')}
                                className={`flex items-center gap-1 px-4 py-2 rounded-md transition-colors ${activeFilter === 'linux'
                                    ? 'bg-blue-600 text-white'
                                    : 'bg-gray-200 hover:bg-gray-300 text-gray-800'
                                    }`}
                            >
                                {getPlatformIcon('linux')} Linux
                            </button>
                            <button
                                onClick={() => setActiveFilter('android')}
                                className={`flex items-center gap-1 px-4 py-2 rounded-md transition-colors ${activeFilter === 'android'
                                    ? 'bg-blue-600 text-white'
                                    : 'bg-gray-200 hover:bg-gray-300 text-gray-800'
                                    }`}
                            >
                                {getPlatformIcon('android')} Android
                            </button>
                        </div>
                    </div>

                    {isLoading ? (
                        <div className="flex justify-center items-center h-64">
                            <div className="text-xl">Loading available downloads...</div>
                        </div>
                    ) : (
                        <div className="space-y-8">
                            {Object.entries(platformGroups).map(([name, apps]) => (
                                <div key={name} className="bg-white/5 rounded-lg p-4">
                                    <h3 className="text-xl font-semibold mb-2">{name}</h3>
                                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                        {apps.map((app, index) => (
                                            <div
                                                key={`${app.name}-${app.platform}-${index}`}
                                                className="border border-gray-300 rounded-lg p-4 transition-all hover:shadow-md hover:border-blue-300"
                                            >
                                                <div className="flex justify-between items-start mb-2">
                                                    <div className="flex items-center gap-2">
                                                        {getPlatformIcon(app.platform)}
                                                        <span className="text-lg font-medium">{app.platform.charAt(0).toUpperCase() + app.platform.slice(1)}</span>
                                                    </div>
                                                    <span className="px-2 py-1 bg-gray-200 text-gray-800 rounded text-xs">{app.version}</span>
                                                </div>
                                                <p className="text-sm text-gray-500 mb-4">{app.description}</p>
                                                <div className="flex justify-between items-center">
                                                    <div className="text-xs text-gray-500">
                                                        <div>{formatFileSize(app.size)}</div>
                                                        <div>Released: {formatDate(app.release_date)}</div>
                                                    </div>
                                                    <a
                                                        href={getDownloadUrl(encodeURIComponent(app.name), app.platform)}
                                                        className="inline-flex items-center gap-1 bg-green-600 hover:bg-green-700 text-white px-3 py-1 rounded transition-colors"
                                                    >
                                                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
                                                        </svg>
                                                        Download
                                                    </a>
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            ))}

                            {filteredDownloads.length === 0 && (
                                <div className="text-center py-12 bg-gray-100 rounded-lg">
                                    <p className="text-xl text-gray-600">No downloads available for the selected platform.</p>
                                </div>
                            )}
                        </div>
                    )}
                </div>

                <div className="bg-white/5 p-6 rounded-lg shadow-md mb-8">
                    <h2 className="text-2xl font-semibold mb-4">About These Downloads</h2>
                    <div className="space-y-4 text-sm">
                        <p>
                            These downloads are provided through our reverse proxy to allow easier access to popular proxy clients in regions where direct GitHub access might be challenging.
                        </p>
                        <p>
                            All clients are fetched directly from their official repositories and verified for integrity before being made available for download.
                        </p>
                        <p className="text-yellow-500">
                            <strong>Note:</strong> For security reasons, always verify the SHA256 hash of downloaded files against the official published values when possible.
                        </p>
                    </div>
                </div>
            </div>
        </main>
    );
} 