import { NextRequest, NextResponse } from 'next/server';
import { loadWasmSingleton } from '@/lib/wasm';

// Define a type for the function info
interface FunctionInfo {
    name: string;
    type: string;
}

export async function GET(request: NextRequest) {
    try {
        console.log('Loading WASM module for test...');
        const wasmModule = await loadWasmSingleton('TestAPI');

        // Check some of the key functions we need
        const functions = {
            admin_file_exists: typeof wasmModule.admin_file_exists === 'function',
            admin_read_file: typeof wasmModule.admin_read_file === 'function',
            admin_write_file: typeof wasmModule.admin_write_file === 'function',
            admin_delete_file: typeof wasmModule.admin_delete_file === 'function',
            list_directory: typeof wasmModule.list_directory === 'function',
            admin_load_github_directory: typeof wasmModule.admin_load_github_directory === 'function',
            sub_process_wasm: typeof wasmModule.sub_process_wasm === 'function',
        };

        // List all available functions if requested
        const listAll = request.nextUrl.searchParams.get('list') === 'true';
        let allFunctions: FunctionInfo[] = [];

        if (listAll) {
            allFunctions = Object.getOwnPropertyNames(wasmModule)
                .filter(name => typeof (wasmModule as any)[name] === 'function')
                .map(name => ({ name, type: typeof (wasmModule as any)[name] }));
        }

        // Test a simple function if possible
        let functionTest = null;
        try {
            if (typeof wasmModule.admin_file_exists === 'function') {
                const exists = await wasmModule.admin_file_exists('README.md');
                functionTest = {
                    function: 'admin_file_exists',
                    result: exists
                };
            }
        } catch (fnError) {
            functionTest = {
                function: 'admin_file_exists',
                error: String(fnError)
            };
        }

        return NextResponse.json({
            success: true,
            message: 'WASM module loaded successfully',
            functions,
            functionTest,
            allFunctions: listAll ? allFunctions : undefined,
            time: new Date().toISOString()
        });
    } catch (error) {
        console.error('Failed to load WASM module:', error);

        return NextResponse.json({
            success: false,
            error: 'Failed to load WASM module',
            details: error instanceof Error ? error.message : String(error),
            time: new Date().toISOString()
        }, { status: 500 });
    }
} 