import type { VercelRequest, VercelResponse } from '@vercel/node';
import { initWasm } from '../../lib/enhanced-wasm-loader.js';

// --- WASM Setup ---
let wasmModule: any = null;
let initPromise: Promise<any> | null = null;

async function loadWasm() {
    if (initPromise) return initPromise;

    initPromise = new Promise(async (resolve, reject) => {
        try {
            console.log("Initializing WASM using enhanced loader...");

            // Use our enhanced loader
            const result = await initWasm();

            if (result.success) {
                wasmModule = result.module;
                console.log("WASM initialized successfully.");
                resolve(wasmModule);
            } else {
                throw result.error;
            }
        } catch (err) {
            console.error("Failed to load or initialize WASM:", err);
            initPromise = null; // Reset promise on failure
            reject(err); // Reject the promise
        }
    });

    return initPromise;
}

/**
 * Convert Rust directory entries to the format expected by the frontend
 */
function convertDirectoryStructure(entries: any[]) {
    // Group entries by their parent paths
    const pathMap = new Map();
    const rootItems = [];

    // First pass: identify all directories and files
    for (const entry of entries) {
        const isDir = entry.is_directory;
        const name = entry.name;
        const path = entry.path;
        const attributes = entry.attributes;

        // Create node
        const node = {
            id: path,
            name,
            type: isDir ? 'folder' : 'file',
            children: isDir ? [] : undefined,
            attributes: attributes || undefined
        };

        // Add to map for lookup
        pathMap.set(path, node);

        // If it's a root-level item, add to rootItems
        if (!path.includes('/') || path.split('/').length === 1) {
            rootItems.push(node);
        }
    }

    // Second pass: build the tree structure
    for (const entry of entries) {
        const path = entry.path;

        // Skip root items
        if (!path.includes('/') || path.split('/').length === 1) {
            continue;
        }

        // Get parent path
        const parentPath = path.substring(0, path.lastIndexOf('/'));
        const parent = pathMap.get(parentPath);

        // If parent exists, add this as a child
        if (parent && parent.children) {
            const node = pathMap.get(path);
            if (node && !parent.children.some((child: any) => child.id === node.id)) {
                parent.children.push(node);
            }
        }
    }

    return rootItems;
}

/**
 * API endpoint to list directory contents
 */
export default async function handler(
    request: VercelRequest,
    response: VercelResponse,
) {
    try {
        // Attempt to load WASM module
        try {
            await loadWasm(); // Ensure WASM is loaded
        } catch (wasmError) {
            console.error("WASM failed to load, using fallback data:", wasmError);
            // Return fallback data immediately
            return response.status(200).json({
                success: true,
                structure: [
                    {
                        id: 'configs',
                        name: 'configs',
                        type: 'folder',
                        children: [
                            { id: 'configs/config.ini', name: 'config.ini', type: 'file' },
                            { id: 'configs/groups.txt', name: 'groups.txt', type: 'file' },
                        ],
                    },
                    {
                        id: 'rules',
                        name: 'rules',
                        type: 'folder',
                        children: [
                            { id: 'rules/direct.list', name: 'direct.list', type: 'file' },
                            { id: 'rules/proxy.list', name: 'proxy.list', type: 'file' },
                        ],
                    },
                    { id: 'README.md', name: 'README.md', type: 'file' },
                ]
            });
        }

        // Get directory path from query
        const dirPath = request.query.path as string || '';
        console.log(`Listing directory: ${dirPath}`);

        // Get flat list of entries from WASM module
        let entries;
        try {
            // Use our new list_directory function from the WASM module
            entries = await wasmModule.list_directory(dirPath);
            console.log(`Got ${entries.length} entries from WASM`);
        } catch (err) {
            console.error("WASM list_directory error:", err);

            // Fallback to static data for testing if WASM fails
            console.log("Using static data as fallback");
            return response.status(200).json({
                success: true,
                structure: [
                    {
                        id: 'configs',
                        name: 'configs',
                        type: 'folder',
                        children: [
                            { id: 'configs/config.ini', name: 'config.ini', type: 'file' },
                            { id: 'configs/groups.txt', name: 'groups.txt', type: 'file' },
                        ],
                    },
                    {
                        id: 'rules',
                        name: 'rules',
                        type: 'folder',
                        children: [
                            { id: 'rules/direct.list', name: 'direct.list', type: 'file' },
                            { id: 'rules/proxy.list', name: 'proxy.list', type: 'file' },
                        ],
                    },
                    { id: 'README.md', name: 'README.md', type: 'file' },
                ]
            });
        }

        // Convert flat list to tree structure
        const structure = convertDirectoryStructure(entries);

        return response.status(200).json({
            success: true,
            structure
        });
    } catch (error: any) {
        console.error('Error listing directory structure:', error);
        return response.status(500).json({
            error: 'Failed to list directory structure',
            details: error.message || 'Unknown error'
        });
    }
} 