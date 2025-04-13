import { FileAttributes } from 'subconverter-wasm';

/**
 * Read file content from the server
 */
export async function readFile(path: string): Promise<string> {
    const response = await fetch(`/api/admin/${encodeURIComponent(path)}`);

    if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || `Failed to read file: ${response.statusText}`);
    }

    const data = await response.json();
    return data.content || '';
}

/**
 * Write content to a file on the server
 */
export async function writeFile(path: string, content: string): Promise<void> {
    const response = await fetch(`/api/admin/${encodeURIComponent(path)}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ content }),
    });

    if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || `Failed to write file: ${response.statusText}`);
    }
}

/**
 * Delete a file or directory on the server
 */
export async function deleteFile(path: string): Promise<void> {
    const response = await fetch(`/api/admin/${encodeURIComponent(path)}`, {
        method: 'DELETE',
    });

    if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || `Failed to delete file: ${response.statusText}`);
    }
}

/**
 * Check if a file exists on the server
 */
export async function checkFileExists(path: string): Promise<boolean> {
    const response = await fetch(`/api/admin/${encodeURIComponent(path)}?exists=true`);

    if (!response.ok) {
        return false;
    }

    const data = await response.json();
    return data.exists || false;
}

/**
 * Get file attributes from the server
 */
export async function getFileAttributes(path: string): Promise<FileAttributes> {
    const response = await fetch(`/api/admin/${encodeURIComponent(path)}?attributes=true`);

    if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || `Failed to get file attributes: ${response.statusText}`);
    }

    const data = await response.json();
    return data.attributes;
}

/**
 * Create a directory on the server
 */
export async function createDirectory(path: string): Promise<void> {
    const response = await fetch(`/api/admin/${encodeURIComponent(path)}`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({ is_directory: true }),
    });

    if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || `Failed to create directory: ${response.statusText}`);
    }
}

/**
 * List files in a directory
 */
export async function listDirectory(path: string = ''): Promise<any> {
    const response = await fetch(`/api/admin/list?path=${encodeURIComponent(path)}`);

    if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || `Failed to list directory: ${response.statusText}`);
    }

    return await response.json();
}

/**
 * Load files from a GitHub repository
 */
export async function loadGitHubDirectory(
    path: string,
    shallow: boolean = true,
    recursive: boolean = true
): Promise<any> {
    const response = await fetch(
        `/api/admin/github?path=${encodeURIComponent(path)}&shallow=${shallow}&recursive=${recursive}`
    );

    if (!response.ok) {
        const data = await response.json();
        throw new Error(data.error || `Failed to load GitHub directory: ${response.statusText}`);
    }

    const data = await response.json();
    return data.result;
}

/**
 * Format a file size number to a human-readable string
 */
export function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

/**
 * Format a timestamp (seconds since epoch) to a localized date string
 */
export function formatTimestamp(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
}
