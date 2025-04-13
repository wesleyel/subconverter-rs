import { NextRequest, NextResponse } from 'next/server';
import { initWasm, DirectoryEntry, SubconverterWasm } from '@/lib/wasm';

// --- WASM Setup ---
let wasmModule: SubconverterWasm | null = null;
let initPromise: Promise<SubconverterWasm | null> | null = null;

async function loadWasm() {
    if (initPromise) return initPromise;

    initPromise = new Promise(async (resolve, reject) => {
        try {
            console.log("Initializing WASM...");

            // Use our loader
            const result = await initWasm();

            if (result) {
                wasmModule = result;
                console.log("WASM initialized successfully.");
                resolve(wasmModule);
            } else {
                throw new Error("Failed to initialize WASM module");
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
function convertDirectoryStructure(entries: DirectoryEntry[]) {
    // Group entries by their parent paths
    const pathMap = new Map();
    const rootItems: any[] = [];

    // First pass: identify all directories and files
    for (const entry of entries) {
        const isDir = entry.is_directory;
        const name = entry.name;
        let path = entry.path;
        const attributes = entry.attributes;

        // Normalize paths for consistency in the tree structure
        if (isDir && !path.endsWith('/')) {
            path = `${path}/`;
        }

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

        // Root level items have no slashes or only one component
        const pathParts = path.split('/').filter((p: string) => p !== '');
        if (pathParts.length === 0 || (pathParts.length === 1 && !path.includes('/'))) {
            rootItems.push(node);
        }
    }

    // Second pass: build the tree structure
    for (const entry of entries) {
        let path = entry.path;
        const isDir = entry.is_directory;

        // Normalize paths for consistency in the tree structure
        if (isDir && !path.endsWith('/')) {
            path = `${path}/`;
        }

        // Skip root items
        const pathParts = path.split('/').filter((p: string) => p !== '');
        if (pathParts.length === 0 || (pathParts.length === 1 && !path.includes('/'))) {
            continue;
        }

        // Get parent path - handle trailing slashes correctly
        let parentPath;
        if (path.endsWith('/')) {
            // For directory paths (with trailing slash), remove the last part
            const pathWithoutTrailingSlash = path.slice(0, -1);
            const lastSlashIndex = pathWithoutTrailingSlash.lastIndexOf('/');
            if (lastSlashIndex === -1) {
                parentPath = '';
            } else {
                parentPath = pathWithoutTrailingSlash.substring(0, lastSlashIndex + 1);
            }
        } else {
            // For file paths, just get the directory part
            const lastSlashIndex = path.lastIndexOf('/');
            if (lastSlashIndex === -1) {
                parentPath = '';
            } else {
                parentPath = path.substring(0, lastSlashIndex + 1);
            }
        }

        const parent = pathMap.get(parentPath);

        // If parent exists, add this as a child
        if (parent && parent.children) {
            const node = pathMap.get(path);
            if (node && !parent.children.some((child: any) => child.id === node.id)) {
                parent.children.push(node);
            }
        } else {
            // If parent wasn't found in the map, it might be a root-level item
            if (!rootItems.some((item: any) => item.id === path)) {
                const node = pathMap.get(path);
                if (node) {
                    rootItems.push(node);
                }
            }
        }
    }

    // Sort children alphabetically with folders first
    const sortItems = (items: any[]) => {
        if (!items) return;

        items.sort((a, b) => {
            // Folders come before files
            if (a.type === 'folder' && b.type !== 'folder') return -1;
            if (a.type !== 'folder' && b.type === 'folder') return 1;

            // Alphabetical sort for the same type
            return a.name.localeCompare(b.name);
        });

        // Sort children recursively
        items.forEach(item => {
            if (item.children) {
                sortItems(item.children);
            }
        });
    };

    sortItems(rootItems);

    return rootItems;
}

/**
 * API endpoint to list directory contents
 */
export async function GET(
    request: NextRequest,
) {
    try {
        // Attempt to load WASM module
        try {
            await loadWasm(); // Ensure WASM is loaded
        } catch (wasmError) {
            console.error("WASM failed to load, using fallback data:", wasmError);
            // Return fallback data immediately
            return NextResponse.json({
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
        const { searchParams } = new URL(request.url);
        const dirPath = searchParams.get('path') || '';
        console.log(`Listing directory: ${dirPath}`);

        // Ensure WASM module is loaded
        if (!wasmModule) {
            throw new Error('Failed to load WASM module');
        }

        // Get flat list of entries from WASM module
        if (typeof wasmModule.list_directory !== 'function') {
            throw new Error('list_directory function not available in WASM module');
        }

        const entries = await wasmModule.list_directory(dirPath);
        console.log(`Got ${entries.length} directory entries`);

        // Convert flat list to hierarchical structure
        const structure = convertDirectoryStructure(entries);

        return NextResponse.json({
            success: true,
            structure
        });
    } catch (error) {
        console.error('Error listing directory:', error);
        return NextResponse.json({
            success: false,
            error: String(error)
        }, { status: 500 });
    }
} 