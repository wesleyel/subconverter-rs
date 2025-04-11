// This file provides TypeScript interfaces for the subconverter WASM module
// The actual implementation will be available once the WASM module is properly linked

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

// Interface for the WASM module
export interface SubconverterWasm {
    // Conversion functions
    convert_subscription: (url: string, target?: string) => Promise<string>;

    // Admin functions
    admin_file_exists?: (path: string) => Promise<boolean>;
    admin_read_file?: (path: string) => Promise<string>;
    admin_write_file?: (path: string, content: string) => Promise<boolean>;
    admin_delete_file?: (path: string) => Promise<boolean>;
    admin_get_file_attributes?: (path: string) => Promise<FileAttributes>;
    admin_create_directory?: (path: string) => Promise<boolean>;
    list_directory?: (path: string) => Promise<DirectoryEntry[]>;
    admin_load_github_directory?: (owner: string, repo: string, path: string, token?: string) => Promise<boolean>;

    // Logging functions
    init_wasm_logging?: (level: string) => void;
    init_panic_hook?: () => void;

    // Allow any other properties that might come from the real WASM module
    [key: string]: any;
}

/**
 * Helper function to adapt any WASM module to our expected interface
 */
export function adaptWasmModule(module: any): SubconverterWasm {
    console.log('Adapting WASM module to our interface...');

    // Start with empty object (will be filled with real or mock functions)
    const adaptedModule: SubconverterWasm = {
        convert_subscription: async (url: string, target = 'clash') => {
            // If the real module has this function, use it
            if (typeof module.convert_subscription === 'function') {
                return await module.convert_subscription(url, target);
            }

            // Otherwise use our mock
            console.log(`Mock convert_subscription: ${url} to ${target}`);
            return `Placeholder for conversion from ${url} to ${target}`;
        }
    };

    // Add admin functions if they exist in the real module
    const adminFunctions = [
        'admin_file_exists',
        'admin_read_file',
        'admin_write_file',
        'admin_delete_file',
        'admin_get_file_attributes',
        'admin_create_directory',
        'list_directory',
        'admin_load_github_directory'
    ];

    for (const funcName of adminFunctions) {
        if (typeof module[funcName] === 'function') {
            adaptedModule[funcName] = module[funcName].bind(module);
        }
    }

    // Add mock implementations for missing functions
    if (!adaptedModule.admin_file_exists) {
        adaptedModule.admin_file_exists = async (path: string) => {
            console.log(`Mock file_exists check for: ${path}`);
            return path.includes('README.md');
        };
    }

    if (!adaptedModule.list_directory) {
        adaptedModule.list_directory = async () => {
            console.log('Mock list_directory called');
            return [
                { name: 'README.md', path: 'README.md', is_directory: false },
                { name: 'configs', path: 'configs', is_directory: true },
                { name: 'rules', path: 'rules', is_directory: true },
            ];
        };
    }

    // Copy any other properties from the real module
    for (const prop in module) {
        if (!adaptedModule[prop] && typeof module[prop] !== 'undefined') {
            adaptedModule[prop] = module[prop];
        }
    }

    return adaptedModule;
}

// Function to initialize the WASM module
export async function initWasm(): Promise<SubconverterWasm | null> {
    try {
        // First check if we're in development
        const isDev = process.env.NODE_ENV === 'development';
        console.log(`Initializing mock WASM module (dev: ${isDev})`);

        // In a real implementation, we would import the actual WASM module
        // For now, create and return a mock implementation
        return adaptWasmModule({
            // Mock base module, will be extended with the adapter
            convert_subscription,
        });
    } catch (error) {
        console.error('Failed to initialize WASM module:', error);
        return null;
    }
} 