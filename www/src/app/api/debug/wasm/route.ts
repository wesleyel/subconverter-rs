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

        // Test conversion function if available
        if (wasmModule && typeof wasmModule.convert_subscription === 'function') {
            try {
                const testUrl = 'https://example.com/test-subscription';
                functionResults.convert_subscription = {
                    called: true,
                    result: (await wasmModule.convert_subscription(testUrl, 'clash')).substring(0, 100) + '...',
                };
            } catch (error) {
                functionResults.convert_subscription = {
                    called: true,
                    error: String(error),
                };
            }
        }

        // Test admin functions if available
        const adminFunctions = [
            'admin_file_exists',
            'list_directory',
            'admin_read_file',
        ];

        for (const funcName of adminFunctions) {
            if (wasmModule && typeof wasmModule[funcName] === 'function') {
                try {
                    let result;
                    if (funcName === 'admin_file_exists') {
                        result = await wasmModule[funcName]('README.md');
                    } else if (funcName === 'list_directory') {
                        result = await wasmModule[funcName]('/');
                    } else if (funcName === 'admin_read_file') {
                        result = (await wasmModule[funcName]('README.md')).substring(0, 100) + '...';
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
            key => typeof wasmModule?.[key] === 'function'
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