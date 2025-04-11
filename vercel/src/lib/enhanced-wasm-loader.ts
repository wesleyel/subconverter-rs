/**
 * Enhanced WASM loading helper for Next.js API routes
 */

import fs from 'fs';
import path from 'path';
import { SubconverterWasm, adaptWasmModule } from './wasm';

// Array of potential relative paths to look for the WASM file
const possiblePaths = [
    // Default location when imported directly 
    '../pkg/subconverter_bg.wasm',
    // Location when running in .next/server
    './subconverter_bg.wasm',
    // Root of the project
    '../../pkg/subconverter_bg.wasm',
    // Absolute path based on current file
    path.resolve(process.cwd(), 'pkg', 'subconverter_bg.wasm'),
    // From node_modules
    path.resolve(process.cwd(), 'node_modules', 'subconverter-wasm', 'subconverter_bg.wasm'),
    // Public directory (for Vercel)
    path.resolve(process.cwd(), 'public', 'wasm', 'subconverter_bg.wasm'),
    // Vercel serverless function path
    '/var/task/public/wasm/subconverter_bg.wasm',
    // Another possible Vercel path
    path.resolve(process.cwd(), '.next', 'server', 'subconverter_bg.wasm'),
];

interface WasmInitResult {
    success: boolean;
    module?: SubconverterWasm;
    error?: Error;
}

/**
 * Attempt to load the WASM file from any of the possible locations
 */
export function findWasmFile(): string | null {
    console.log('Current working directory:', process.cwd());
    console.log('Searching for WASM file in possible locations...');

    for (const relativePath of possiblePaths) {
        try {
            const absolutePath = path.isAbsolute(relativePath)
                ? relativePath
                : path.resolve(__dirname, relativePath);

            console.log(`Checking path: ${absolutePath}`);

            // Check if file exists
            if (fs.existsSync(absolutePath)) {
                console.log(`✅ Found WASM file at ${absolutePath}`);
                return absolutePath;
            }
        } catch (error: any) {
            console.error(`❌ Error finding WASM file at ${relativePath}: ${error.message}`);
        }
    }

    console.error('❌ Could not find WASM file in any of the possible locations');
    return null;
}

/**
 * Print available modules and functions in a WASM module
 */
function inspectWasmModule(module: any): void {
    console.log('Inspecting WASM module content:');
    console.log('- Module type:', typeof module);

    if (typeof module !== 'object' || module === null) {
        console.log('- Module is not an object');
        return;
    }

    const properties = Object.getOwnPropertyNames(module).sort();
    console.log(`- Module has ${properties.length} properties:`);

    for (const prop of properties) {
        try {
            const value = module[prop];
            const type = typeof value;
            const isFunction = type === 'function';
            console.log(`  - ${prop}: ${type}${isFunction ? ' (function)' : ''}`);
        } catch (e: any) {
            console.log(`  - ${prop}: [Error accessing: ${e.message}]`);
        }
    }

    // Look specifically for the functions we need
    const criticalFunctions = [
        'admin_load_github_directory',
        'admin_read_file',
        'admin_write_file',
        'admin_delete_file',
        'admin_file_exists',
        'admin_get_file_attributes',
        'admin_create_directory',
        'list_directory',
        'convert_subscription'
    ];

    console.log('- Checking critical functions:');
    for (const funcName of criticalFunctions) {
        const exists = typeof module[funcName] === 'function';
        console.log(`  - ${funcName}: ${exists ? '✅ Available' : '❌ Missing'}`);
    }
}

/**
 * Initialize the WASM module with proper error handling
 */
export async function initWasm(): Promise<WasmInitResult> {
    try {
        console.log('🔄 Trying to load WASM module...');

        // First check if we're in a development or production environment
        const isDev = process.env.NODE_ENV === 'development';
        console.log(`Environment: ${isDev ? 'Development' : 'Production'}`);

        // In development, try dynamic import
        let wasmModule = null;

        if (isDev) {
            try {
                // This will likely fail in production/Vercel
                const importedModule = await import('subconverter-wasm');
                wasmModule = adaptWasmModule(importedModule);
                console.log('WASM module loaded successfully via import in dev mode.');
            } catch (importError: any) {
                console.error('Error loading WASM via import in dev mode:', importError.message);
            }
        }

        // If we still don't have a module, fall back to our mock implementation
        if (!wasmModule) {
            console.log('Using mock WASM implementation.');
            const { initWasm: mockInit } = await import('./wasm');
            wasmModule = await mockInit();

            if (!wasmModule) {
                throw new Error('Failed to initialize mock WASM module');
            }
        }

        // Inspect module content to help diagnose issues
        inspectWasmModule(wasmModule);

        return {
            success: true,
            module: wasmModule
        };
    } catch (error) {
        console.error('❌ Error loading WASM module:', error);
        console.error('Stack trace:', error instanceof Error ? error.stack : undefined);

        return {
            success: false,
            error: error instanceof Error ? error : new Error(String(error))
        };
    }
} 