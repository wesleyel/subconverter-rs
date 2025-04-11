import { kv } from '@vercel/kv';
import { NextRequest, NextResponse } from 'next/server';
import { initWasm } from '@/lib/enhanced-wasm-loader';

// WASM setup for testing
let wasmModule: any = null;
let wasmInitialized = false;

async function loadWasm() {
    if (wasmInitialized) return { success: true, message: "WASM already loaded" };

    try {
        console.log("Loading WASM module via enhanced loader...");

        // Use our enhanced loader
        const result = await initWasm();

        if (result.success && result.module) {
            wasmModule = result.module;
            wasmInitialized = true;
            console.log("WASM initialized successfully.");
            return { success: true, message: "WASM loaded successfully" };
        } else {
            throw result.error || new Error("Unknown error loading WASM");
        }
    } catch (err) {
        console.error("Failed to load or initialize WASM:", err);
        return {
            success: false,
            error: err instanceof Error ? err.message : String(err)
        };
    }
}

export async function GET(request: NextRequest) {
    // Check if query has wasm=true parameter
    const { searchParams } = new URL(request.url);
    const checkWasm = searchParams.get('wasm') === 'true';

    if (checkWasm) {
        try {
            const wasmStatus = await loadWasm();

            // Try to call a WASM function if loaded
            let wasmFunctionResult = null;
            if (wasmStatus.success && wasmModule) {
                try {
                    // Check if admin_file_exists is available
                    if (typeof wasmModule.admin_file_exists === 'function') {
                        wasmFunctionResult = {
                            tested: true,
                            function: 'admin_file_exists',
                            result: await wasmModule.admin_file_exists('README.md')
                        };
                    }
                } catch (fnError) {
                    wasmFunctionResult = {
                        tested: true,
                        error: String(fnError)
                    };
                }
            }

            return NextResponse.json({
                test: 'wasm',
                status: wasmStatus,
                wasmFunction: wasmFunctionResult,
                time: new Date().toISOString()
            });
        } catch (error) {
            return NextResponse.json({
                test: 'wasm',
                error: String(error),
                time: new Date().toISOString()
            }, { status: 500 });
        }
    }

    // Regular KV test
    const key = searchParams.get('key') || 'defaultKey';
    try {
        const value = await kv.get(key);
        return NextResponse.json({
            test: 'kv',
            key,
            value,
            time: new Date().toISOString()
        });
    } catch (error) {
        console.error('Error getting value from KV:', error);
        return NextResponse.json({
            test: 'kv',
            error: 'Failed to get value from KV',
            details: String(error),
            time: new Date().toISOString()
        }, { status: 500 });
    }
}

export async function POST(request: NextRequest) {
    // Example: Set a value
    try {
        const body = await request.json();
        const { key, value } = body;

        if (!key || value === undefined) {
            return NextResponse.json({ error: 'Missing key or value in request body' }, { status: 400 });
        }

        await kv.set(key, value);
        return NextResponse.json({ success: true, key, value });
    } catch (error) {
        console.error('Error setting value in KV:', error);
        return NextResponse.json({ error: 'Failed to set value in KV' }, { status: 500 });
    }
}

export async function DELETE(request: NextRequest) {
    // Example: Delete a value
    const { searchParams } = new URL(request.url);
    const key = searchParams.get('key');

    if (!key) {
        return NextResponse.json({ error: 'Missing key in query parameters' }, { status: 400 });
    }

    try {
        const result = await kv.del(key);
        return NextResponse.json({ success: result > 0, key, deletedCount: result });
    } catch (error) {
        console.error('Error deleting value from KV:', error);
        return NextResponse.json({ error: 'Failed to delete value from KV' }, { status: 500 });
    }
} 