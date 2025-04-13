// This file provides TypeScript interfaces for the subconverter WASM module
// with enhanced loading logic for Next.js API routes

// For browser environments (will be undefined in Node)
declare global {
    interface Window {
        WebAssembly: typeof WebAssembly;
    }
}

// For fs and path operations in Node.js environments
import fs from 'fs';
import path from 'path';

// Placeholder function that will be replaced with actual WASM function
export function convert_subscription(url: string, target = 'clash'): Promise<string> {
    // In development/demo mode, return a placeholder
    return Promise.resolve(`Placeholder for conversion from ${url} to ${target}`);
}

// Interface for file attributes
export interface FileAttributes {
    size?: number;
    modified?: string;
    is_directory: boolean;
    name: string;
    path: string;
}

// Interface for directory entries
export interface DirectoryEntry {
    name: string;
    path: string;
    is_directory: boolean;
    attributes?: Record<string, unknown>;
}

// Interface for the WASM module with more flexible return types
export interface SubconverterWasm {
    // Conversion functions
    convert_subscription: (url: string, target?: string) => Promise<string>;

    // Admin functions
    admin_file_exists?: (path: string) => Promise<boolean | any>;
    admin_read_file?: (path: string) => Promise<string | any>;
    admin_write_file?: (path: string, content: string) => Promise<boolean | void | any>;
    admin_delete_file?: (path: string) => Promise<boolean | void | any>;
    admin_get_file_attributes?: (path: string) => Promise<FileAttributes | any>;
    admin_create_directory?: (path: string) => Promise<boolean | void | any>;
    list_directory?: (path: string) => Promise<DirectoryEntry[] | any>;
    admin_load_github_directory?: (owner: string, repo: string, path: string, token?: string) => Promise<boolean | void | any>;

    // Logging functions
    init_wasm_logging?: (level: string) => void;
    init_panic_hook?: () => void;

    // This could be used for processing subscription URLs with more parameters
    sub_process_wasm?: (json_config: string) => Promise<any>;

    // Allow any other properties that might come from the real WASM module
    [key: string]: any;
}

/**
 * Print available modules and functions in a WASM module for debugging
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
        'convert_subscription',
        'sub_process_wasm',
        'admin_file_exists',
        'admin_read_file',
        'admin_write_file',
        'admin_delete_file',
        'admin_get_file_attributes',
        'admin_create_directory',
        'list_directory',
        'admin_load_github_directory'
    ];

    console.log('- Checking critical functions:');
    for (const funcName of criticalFunctions) {
        const exists = typeof module[funcName] === 'function';
        console.log(`  - ${funcName}: ${exists ? '✅ Available' : '❌ Missing'}`);
    }
}

/**
 * Check if we're running in the Netlify environment
 */
function isNetlifyEnvironment(): boolean {
    return typeof process !== 'undefined' &&
        process.env.NETLIFY === 'true' ||
        (process.cwd && process.cwd() === '/var/task');
}

/**
 * Direct Node.js import for WASM modules in Netlify
 */
async function loadWasmInNetlify(): Promise<any> {
    console.log('Attempting direct WASM loading for Netlify environment...');

    // Known path from previous logs where the file was found
    const netlifyWasmPath = '/var/task/subconverter_bg.wasm';

    try {
        // Check if file exists
        if (fs.existsSync(netlifyWasmPath)) {
            console.log(`Found WASM file at ${netlifyWasmPath}`);

            // In Netlify, try multiple possible paths for the JS module
            const possibleJsPaths = [
                // Try module paths
                'subconverter-wasm',
                'subconverter-wasm/subconverter',
                // Try absolute paths if needed (less likely to work)
                '/var/task/node_modules/subconverter-wasm/subconverter'
            ];

            // Try each path
            for (const jsPath of possibleJsPaths) {
                try {
                    console.log(`Trying to import from: ${jsPath}`);
                    // Use dynamic import instead of require
                    const wasmModule = await import(jsPath);

                    if (wasmModule) {
                        console.log(`Successfully loaded module from ${jsPath}`);

                        // Initialize panic hook if available
                        if (typeof wasmModule.init_panic_hook === 'function') {
                            wasmModule.init_panic_hook();
                        }

                        return wasmModule;
                    }
                } catch (importError: any) {
                    console.log(`Import failed for ${jsPath}: ${importError.message}`);
                    // Continue to next path
                }
            }

            console.log('All import paths failed, will try alternative loading methods');
        } else {
            console.error(`WASM file not found at ${netlifyWasmPath}`);
        }
    } catch (error) {
        console.error('Error during Netlify WASM loading:', error);
    }

    return null;
}

/**
 * Initialize the WASM module using the pre-generated wasm-bindgen bindings
 */
export async function initWasm(): Promise<SubconverterWasm | null> {
    try {
        console.log('🔄 Initializing WASM module using wasm-bindgen bindings...');

        // Check environment
        const isDev = process.env.NODE_ENV === 'development';
        console.log(`Environment: ${isDev ? 'Development' : 'Production'}`);

        let moduleWithInterface: SubconverterWasm | null = null;

        // For Netlify, try a specialized loading approach
        if (isNetlifyEnvironment()) {
            console.log('Detected Netlify environment, using specialized loading');
            const netlifyModule = await loadWasmInNetlify();

            if (netlifyModule) {
                moduleWithInterface = adaptModuleToInterface(netlifyModule);
            }
        }

        // If we don't have a module yet, try the standard approaches
        if (!moduleWithInterface) {
            // First try: Dynamic import
            try {
                console.log('Importing subconverter-wasm module via dynamic import...');
                const wasmModule = await import('subconverter-wasm');

                // Initialize the module
                if (typeof wasmModule.init_panic_hook === 'function') {
                    console.log('Initializing panic hook...');
                    wasmModule.init_panic_hook();
                }

                if (typeof wasmModule.init_wasm_logging === 'function') {
                    console.log('Initializing WASM logging...');
                    wasmModule.init_wasm_logging('info');
                }

                console.log('WASM module loaded via dynamic import');

                // Create interface adapter
                moduleWithInterface = adaptModuleToInterface(wasmModule);
            } catch (importError: any) {
                console.error('Dynamic import failed:', importError.message);

                // Second try: Alternative dynamic import
                try {
                    console.log('Trying alternative import approach...');
                    // Try a different import approach - better than using require()
                    const wasmModulePromise = import('subconverter-wasm/subconverter');
                    const wasmModule = await wasmModulePromise;

                    if (wasmModule) {
                        if (typeof wasmModule.init_panic_hook === 'function') {
                            wasmModule.init_panic_hook();
                        }

                        console.log('WASM module loaded via alternative import');

                        // Create interface adapter
                        moduleWithInterface = adaptModuleToInterface(wasmModule);
                    }
                } catch (altImportError: any) {
                    console.error('Alternative import also failed:', altImportError.message);
                }
            }
        }

        if (moduleWithInterface) {
            console.log('Successfully initialized WASM module');
            return moduleWithInterface;
        }

        console.error('❌ All WASM loading methods failed');
        return null;
    } catch (error) {
        console.error('❌ Unhandled error in initWasm:', error);
        return null;
    }
}

/**
 * Adapt any WASM module to our expected interface
 */
function adaptModuleToInterface(wasmModule: any): SubconverterWasm {
    inspectWasmModule(wasmModule);

    // Start with the module itself
    const adaptedModule: SubconverterWasm = {
        ...wasmModule,

        // Ensure convert_subscription exists (required by interface)
        convert_subscription: async (url: string, target = 'clash') => {
            // Case 1: Module has convert_subscription directly
            if (typeof wasmModule.convert_subscription === 'function') {
                return wasmModule.convert_subscription(url, target);
            }

            // Case 2: Module has sub_process_wasm (could be used as alternative)
            if (typeof wasmModule.sub_process_wasm === 'function') {
                console.log('Using sub_process_wasm as alternative to convert_subscription');
                try {
                    // Format the parameters as JSON that the function expects
                    const jsonConfig = JSON.stringify({
                        url: url,
                        target: target,
                        // Add any other required parameters
                    });
                    return wasmModule.sub_process_wasm(jsonConfig);
                } catch (error) {
                    console.error('Error using sub_process_wasm:', error);
                }
            }

            // Fallback if nothing is available
            console.log(`Fallback convert_subscription: ${url} to ${target}`);
            return `Placeholder for conversion from ${url} to ${target} (Module API mismatch)`;
        }
    };

    return adaptedModule;
} 