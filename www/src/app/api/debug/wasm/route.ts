import { NextRequest, NextResponse } from 'next/server';
import { initWasm } from '@/lib/wasm';

export async function GET(_request: NextRequest) {
    // Check if we're in production - don't expose debug endpoints in production
    const isProd = process.env.NODE_ENV === 'production' &&
        process.env.NETLIFY_CONTEXT === 'production';

    if (isProd) {
        return NextResponse.json(
            { error: 'Debug endpoints not available in production' },
            { status: 403 }
        );
    }

    try {
        // Initialize WASM module
        console.log('Debug: Initializing WASM module...');
        const wasmModule = await initWasm();

        const functionResults: Record<string, any> = {};

        // Test admin functions if available
        const adminFunctions = [
            'admin_file_exists',
            'list_directory',
            'admin_read_file',
        ] as const;

        for (const funcName of adminFunctions) {
            if (wasmModule && typeof (wasmModule as any)[funcName] === 'function') {
                try {
                    let result;
                    if (funcName === 'admin_file_exists') {
                        result = await (wasmModule as any)[funcName]('README.md');
                    } else if (funcName === 'list_directory') {
                        result = await (wasmModule as any)[funcName]('/');
                    } else if (funcName === 'admin_read_file') {
                        result = (await (wasmModule as any)[funcName]('README.md')).substring(0, 100) + '...';
                    }

                    functionResults[funcName] = {
                        called: true,
                        result,
                    };
                } catch (error) {
                    functionResults[funcName] = {
                        called: true,
                        error: String(error),
                    };
                }
            }
        }

        // Get list of available functions
        const availableFunctions = Object.keys(wasmModule || {}).filter(
            key => typeof (wasmModule as any)?.[key] === 'function'
        );

        return NextResponse.json({
            wasmInitialized: !!wasmModule,
            environment: {
                nodeEnv: process.env.NODE_ENV,
                netlifyContext: process.env.NETLIFY_CONTEXT || 'unknown',
                netlifyDeployId: process.env.NETLIFY_DEPLOY_ID || 'unknown',
            },
            availableFunctions,
            functionTests: functionResults,
            timestamp: new Date().toISOString(),
        });
    } catch (error) {
        console.error('Error in WASM debug endpoint:', error);
        return NextResponse.json(
            {
                error: 'Failed to initialize or test WASM module',
                details: String(error),
                timestamp: new Date().toISOString(),
            },
            { status: 500 }
        );
    }
} 