"use client";

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { getAvailableDownloads, detectUserOS, AppDownloadInfo } from '@/lib/api-client';

export default function DownloadsPage() {
    const [downloads, setDownloads] = useState<AppDownloadInfo[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [userOS, setUserOS] = useState('');

    useEffect(() => {
        async function loadDownloads() {
            try {
                setIsLoading(true);
                const downloadsList = await getAvailableDownloads();
                setDownloads(downloadsList);
            } catch (err) {
                console.error('Failed to load downloads:', err);
                setError('Failed to load available downloads. Please try again later.');
            } finally {
                setIsLoading(false);
            }
        }

        // Detect user OS
        setUserOS(detectUserOS());

        loadDownloads();
    }, []);

    // Group downloads by application name
    const downloadsByApp = downloads.reduce((acc, download) => {
        if (!acc[download.name]) {
            acc[download.name] = [];
        }
        acc[download.name].push(download);
        return acc;
    }, {} as Record<string, AppDownloadInfo[]>);

    return (
        <main className="flex min-h-screen flex-col items-center p-4 md:p-8 lg:p-24">
            <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm">
                <div className="flex flex-col sm:flex-row items-center justify-between mb-6">
                    <h1 className="text-4xl font-bold mb-4 sm:mb-0">Client Downloads</h1>
                    <Link
                        href="/"
                        className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                    >
                        Back to Home
                    </Link>
                </div>

                {isLoading ? (
                    <div className="flex justify-center items-center h-64">
                        <div className="text-xl">Loading available downloads...</div>
                    </div>
                ) : error ? (
                    <div className="bg-red-100 border-l-4 border-red-500 text-red-700 p-4 mb-6">
                        <p>{error}</p>
                    </div>
                ) : (
                    <>
                        <div className="mb-8">
                            <p className="mb-4">
                                Download and install the appropriate client application for your platform. We automatically detect your system as <strong className="font-bold">{userOS.charAt(0).toUpperCase() + userOS.slice(1)}</strong>.
                            </p>
                        </div>

                        {Object.entries(downloadsByApp).map(([appName, appDownloads]) => (
                            <div key={appName} className="bg-white/5 p-6 rounded-lg shadow-md mb-8">
                                <h2 className="text-2xl font-semibold mb-4">{appName}</h2>
                                <p className="mb-4">{appDownloads[0]?.description || 'Proxy client application'}</p>

                                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                    {appDownloads.map((download) => (
                                        <div
                                            key={`${download.name}-${download.platform}`}
                                            className={`p-4 border rounded-lg flex flex-col ${userOS === download.platform
                                                ? 'border-green-500 bg-green-50/10'
                                                : 'border-gray-300 bg-white/5'
                                                }`}
                                        >
                                            <div className="flex-1">
                                                <h3 className="font-medium text-lg mb-1">
                                                    {download.platform.charAt(0).toUpperCase() + download.platform.slice(1)}
                                                    {userOS === download.platform && (
                                                        <span className="ml-2 bg-green-100 text-green-800 text-xs px-2 py-1 rounded">
                                                            Recommended
                                                        </span>
                                                    )}
                                                </h3>
                                                <p className="text-sm mb-4">Version: {download.version}</p>
                                            </div>
                                            <a
                                                href={download.download_url}
                                                className={`
                                                    w-full py-2 px-4 rounded text-white text-center font-medium
                                                    ${userOS === download.platform
                                                        ? 'bg-green-600 hover:bg-green-700'
                                                        : 'bg-blue-600 hover:bg-blue-700'
                                                    }
                                                `}
                                            >
                                                Download
                                            </a>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        ))}

                        <div className="bg-blue-50/10 p-6 rounded-lg shadow-md mb-8 border border-blue-200">
                            <h2 className="text-xl font-semibold mb-2">Using Subconverter with Client Apps</h2>
                            <p className="mb-4">
                                After downloading and installing the app, use your subscription URL from the home page as the subscription link in the application.
                            </p>
                            <div className="flex flex-col gap-2">
                                <div className="font-medium">Quick Steps:</div>
                                <ol className="list-decimal pl-5 space-y-1">
                                    <li>Install the application for your platform</li>
                                    <li>Open the application and navigate to profiles/subscriptions</li>
                                    <li>Add a new subscription with your generated URL</li>
                                    <li>Update the subscription to fetch the latest configuration</li>
                                </ol>
                            </div>
                        </div>
                    </>
                )}
            </div>
        </main>
    );
} 