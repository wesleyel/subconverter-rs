import { NextRequest, NextResponse } from 'next/server';
import { initWasm } from '@/lib/enhanced-wasm-loader';

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

            if (result.success && result.module) {
                wasmModule = result.module;
                console.log("WASM initialized successfully.");
                resolve(wasmModule);
            } else {
                throw result.error || new Error("Unknown error loading WASM");
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
 * GET handler to read file contents
 */
export async function GET(request: NextRequest) {
    try {
        // Load WASM module
        await loadWasm();

        // Get file path from query parameters
        const { searchParams } = new URL(request.url);
        const filePath = searchParams.get('path');

        if (!filePath) {
            return NextResponse.json({
                success: false,
                error: 'Missing path parameter',
            }, { status: 400 });
        }

        // Check if admin_read_file function is available
        if (typeof wasmModule.admin_read_file !== 'function') {
            throw new Error('admin_read_file function not available in WASM module');
        }

        // Check if file exists first
        const fileExists = await wasmModule.admin_file_exists(filePath);
        if (!fileExists) {
            return NextResponse.json({
                success: false,
                error: `File not found: ${filePath}`,
            }, { status: 404 });
        }

        // Read file content
        const content = await wasmModule.admin_read_file(filePath);

        // Get file attributes if available
        let attributes = null;
        if (typeof wasmModule.admin_get_file_attributes === 'function') {
            try {
                attributes = await wasmModule.admin_get_file_attributes(filePath);
            } catch (error) {
                console.error('Error getting file attributes:', error);
            }
        }

        return NextResponse.json({
            success: true,
            path: filePath,
            content,
            attributes,
        });
    } catch (error) {
        console.error('Error reading file:', error);
        return NextResponse.json({
            success: false,
            error: String(error),
        }, { status: 500 });
    }
}

/**
 * POST handler to write to a file
 */
export async function POST(request: NextRequest) {
    try {
        // Load WASM module
        await loadWasm();

        // Get request body
        const body = await request.json();
        const { path, content } = body;

        if (!path) {
            return NextResponse.json({
                success: false,
                error: 'Missing path parameter',
            }, { status: 400 });
        }

        if (content === undefined) {
            return NextResponse.json({
                success: false,
                error: 'Missing content parameter',
            }, { status: 400 });
        }

        // Check if admin_write_file function is available
        if (typeof wasmModule.admin_write_file !== 'function') {
            throw new Error('admin_write_file function not available in WASM module');
        }

        // Write file content
        const result = await wasmModule.admin_write_file(path, content);

        return NextResponse.json({
            success: true,
            path,
            written: result,
        });
    } catch (error) {
        console.error('Error writing file:', error);
        return NextResponse.json({
            success: false,
            error: String(error),
        }, { status: 500 });
    }
}

/**
 * DELETE handler to delete a file
 */
export async function DELETE(request: NextRequest) {
    try {
        // Load WASM module
        await loadWasm();

        // Get file path from query parameters
        const { searchParams } = new URL(request.url);
        const filePath = searchParams.get('path');

        if (!filePath) {
            return NextResponse.json({
                success: false,
                error: 'Missing path parameter',
            }, { status: 400 });
        }

        // Check if admin_delete_file function is available
        if (typeof wasmModule.admin_delete_file !== 'function') {
            throw new Error('admin_delete_file function not available in WASM module');
        }

        // Check if file exists first
        const fileExists = await wasmModule.admin_file_exists(filePath);
        if (!fileExists) {
            return NextResponse.json({
                success: false,
                error: `File not found: ${filePath}`,
            }, { status: 404 });
        }

        // Delete file
        const result = await wasmModule.admin_delete_file(filePath);

        return NextResponse.json({
            success: true,
            path: filePath,
            deleted: result,
        });
    } catch (error) {
        console.error('Error deleting file:', error);
        return NextResponse.json({
            success: false,
            error: String(error),
        }, { status: 500 });
    }
} 