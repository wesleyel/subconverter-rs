import { NextRequest, NextResponse } from 'next/server';
import { loadWasmSingleton } from '@/lib/wasm';

const DOWNLOADS_CACHE_FILE = 'downloads/available_downloads.json';

interface PlatformConfig {
    repo: string;
    asset_pattern: string;
    fallback_url: string;
}

interface AppDownload {
    name: string;
    description: string;
    platforms: Record<string, PlatformConfig>;
}

async function getDownloadInfo(appId: string, platform: string): Promise<PlatformConfig | null> {
    try {
        const wasmModule = await loadWasmSingleton('Admin');
        const exists = await wasmModule.admin_file_exists(DOWNLOADS_CACHE_FILE);

        if (!exists) {
            return null;
        }

        const cacheData = await wasmModule.admin_read_file(DOWNLOADS_CACHE_FILE);
        const parsedCache = JSON.parse(cacheData);

        if (!Array.isArray(parsedCache.downloads)) {
            return null;
        }

        // Find the app and platform in the cached data
        const app = parsedCache.downloads.find((app: AppDownload) =>
            app.name.toLowerCase() === appId.toLowerCase()
        );

        if (!app || !app.platforms || !app.platforms[platform]) {
            console.error(`Download info not found for ${appId} on ${platform}`);
            return null;
        }

        return app.platforms[platform];
    } catch (error) {
        console.error('Error getting download info:', error);
        return null;
    }
}

async function fetchLatestReleaseAsset(repo: string, assetPattern: string): Promise<string | null> {
    try {
        // GitHub API URL for the latest release
        const apiUrl = `https://api.github.com/repos/${repo}/releases/latest`;

        // GitHub API requires a User-Agent header
        const response = await fetch(apiUrl, {
            headers: {
                'User-Agent': 'Subconverter-RS-ProxyDownloader/1.0',
                'Accept': 'application/vnd.github.v3+json'
            }
        });

        if (!response.ok) {
            throw new Error(`GitHub API error: ${response.status} ${response.statusText}`);
        }

        const release = await response.json();

        // Find the matching asset
        const regex = new RegExp(assetPattern);
        const asset = release.assets.find((asset: any) => regex.test(asset.name));

        if (!asset) {
            return null;
        }

        return asset.browser_download_url;
    } catch (error) {
        console.error('Error fetching latest release:', error);
        return null;
    }
}

export async function GET(
    request: NextRequest,
    { params }: any
) {
    try {
        const { appId, platform } = (await params) as { appId: string; platform: string };

        if (!appId || !platform) {
            return NextResponse.json(
                { error: 'App ID and platform are required' },
                { status: 400 }
            );
        }

        // Get download configuration for the requested app and platform
        const downloadInfo = await getDownloadInfo(decodeURIComponent(appId), decodeURIComponent(platform));

        if (!downloadInfo) {
            return NextResponse.json(
                { error: 'Download not found for the specified app and platform' },
                { status: 404 }
            );
        }

        // Try to get the latest release asset URL
        let downloadUrl = await fetchLatestReleaseAsset(
            downloadInfo.repo,
            downloadInfo.asset_pattern
        );

        // If we couldn't get a URL from the GitHub API, use the fallback URL
        if (!downloadUrl) {
            downloadUrl = downloadInfo.fallback_url;
        }

        // Redirect to the actual download URL
        return NextResponse.redirect(downloadUrl);
    } catch (error) {
        console.error('Error processing download request:', error);
        return NextResponse.json(
            { error: 'Failed to process download request' },
            { status: 500 }
        );
    }
} 