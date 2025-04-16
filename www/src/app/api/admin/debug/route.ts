import { NextRequest, NextResponse } from 'next/server';
import { loadWasmSingleton } from '@/lib/wasm';

/**
 * Handle debug operations including panic testing
 */
export async function GET(request: NextRequest) {
    let wasmModule;
    try {
        wasmModule = await loadWasmSingleton('Admin');
    } catch (error) {
        console.error("Failed to load WASM module:", error);
        return NextResponse.json(
            {
                error: 'Failed to load WASM module',
                details: error instanceof Error ? error.message : String(error)
            },
            { status: 500 }
        );
    }

    // Get operation from search params
    const operation = request.nextUrl.searchParams.get('op');

    console.log(`Admin debug API request: GET /api/admin/debug?op=${operation}`);

    if (operation === 'panic') {
        // Intentionally trigger a panic to test error handling
        try {
            console.log("Triggering intentional panic for testing...");
            await wasmModule.admin_debug_test_panic();

            // If we get here, the panic was somehow caught and didn't propagate
            return NextResponse.json({
                warning: "Panic test failed - panic did not occur as expected"
            });
        } catch (error: any) {
            // This is expected behavior - the panic should be caught
            console.error("Panic test succeeded - caught expected error:", error);

            return NextResponse.json({
                success: true,
                operation: 'panic-test',
                message: 'Panic test successfully triggered and caught',
                error: typeof error === 'string' ? error : (error.message || String(error))
            });
        }
    } else if (operation === 'init-kv') {
        // Initialize KV bindings (useful if they've become disconnected)
        try {
            console.log("Initializing KV bindings...");
            await wasmModule.admin_init_kv_bindings_js();

            return NextResponse.json({
                success: true,
                operation: 'init-kv',
                message: 'KV bindings initialized successfully'
            });
        } catch (error: any) {
            console.error("Failed to initialize KV bindings:", error);

            return NextResponse.json(
                {
                    error: 'Failed to initialize KV bindings',
                    details: typeof error === 'string' ? error : (error.message || String(error))
                },
                { status: 500 }
            );
        }
    } else {
        // List available debug operations
        return NextResponse.json({
            available_operations: [
                {
                    op: 'panic',
                    description: 'Test panic handling by triggering an intentional panic'
                },
                {
                    op: 'init-kv',
                    description: 'Initialize KV bindings if they have become disconnected'
                }
            ]
        });
    }
} 