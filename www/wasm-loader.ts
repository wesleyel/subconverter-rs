import fs from 'fs';
import path from 'path';

/**
 * A helper function to load WASM in various environments including Netlify Functions
 */
export async function initWasm() {
    console.log('🔍 Initializing WASM loader');
    let wasm;

    // Check if we're in a Netlify Function
    const isNetlify = process.env.NETLIFY === 'true' ||
        process.env.CONTEXT === 'production' ||
        process.env.NETLIFY_LOCAL === 'true' ||
        (process.env.DEPLOY_URL && process.env.DEPLOY_URL.includes('netlify'));

    // Log ALL environment variables for debugging
    console.log('Environment variables:', {
        NETLIFY: process.env.NETLIFY,
        CONTEXT: process.env.CONTEXT,
        DEPLOY_URL: process.env.DEPLOY_URL,
        SITE_NAME: process.env.SITE_NAME,
        NODE_ENV: process.env.NODE_ENV,
        LAMBDA_TASK_ROOT: process.env.LAMBDA_TASK_ROOT,
        LAMBDA_RUNTIME_DIR: process.env.LAMBDA_RUNTIME_DIR
    });

    if (isNetlify) {
        console.log('📡 Detected Netlify environment, using specialized loading');
        console.log('Current working directory:', process.cwd());

        // Check for exact Lambda execution directory structure
        let lambdaRoot = process.env.LAMBDA_TASK_ROOT || '/var/task';
        console.log('Lambda root directory:', lambdaRoot);

        // Potential paths in order of likelihood
        const potentialPaths = [
            // Root level (most likely to work in Netlify)
            path.resolve(process.cwd(), 'subconverter_bg.wasm'),
            // Lambda task root
            path.join(lambdaRoot, 'subconverter_bg.wasm'),
            // Various function paths
            path.resolve(process.cwd(), '.netlify/functions/subconverter_bg.wasm'),
            path.resolve(process.cwd(), '.netlify/functions-internal/subconverter_bg.wasm'),
            path.resolve(process.cwd(), 'functions/subconverter_bg.wasm'),
            path.resolve(lambdaRoot, 'functions/subconverter_bg.wasm'),
            // Next.js paths
            path.resolve(process.cwd(), '.next/server/subconverter_bg.wasm'),
            path.resolve(process.cwd(), '.next/server/chunks/subconverter_bg.wasm'),
            path.resolve(lambdaRoot, '.next/server/chunks/subconverter_bg.wasm'),
            // Static paths
            path.resolve(process.cwd(), 'static/wasm/subconverter_bg.wasm'),
            path.resolve(process.cwd(), 'public/subconverter_bg.wasm')
        ];

        // Log available directories and files for debugging
        console.log('📂 Current directory structure in Netlify:');
        try {
            console.log(`Current working directory: ${process.cwd()}`);
            const rootFiles = fs.readdirSync(process.cwd());
            console.log(`Root files: ${rootFiles.join(', ')}`);

            // Try to find WASM file in any subdirectory
            try {
                const findResult = fs.existsSync('/usr/bin/find') ?
                    require('child_process').execSync('find . -name "*.wasm" -type f 2>/dev/null', { encoding: 'utf8' }) :
                    'find command not available';
                console.log('Find results for WASM files:', findResult.trim());
            } catch (e: any) {
                console.log('Error running find command:', e.message);
            }
        } catch (err: any) {
            console.error('Error listing files:', err);
        }

        // Try each path
        for (const wasmPath of potentialPaths) {
            try {
                console.log(`Checking for WASM at: ${wasmPath}`);
                if (fs.existsSync(wasmPath)) {
                    console.log(`✅ Found WASM at: ${wasmPath}`);
                    const stats = fs.statSync(wasmPath);
                    console.log(`WASM file size: ${stats.size} bytes`);

                    // Read the file directly
                    const wasmBuffer = fs.readFileSync(wasmPath);
                    console.log(`Read ${wasmBuffer.length} bytes from WASM file`);

                    // Instantiate WebAssembly from buffer
                    wasm = await WebAssembly.instantiate(wasmBuffer, {});
                    console.log('✅ WASM instantiated from file buffer');
                    return wasm.instance.exports;
                }
            } catch (err: any) {
                console.error(`Error checking/loading WASM from ${wasmPath}:`, err);
            }
        }

        // If we got here, try importing from the NPM package
        try {
            console.log('⚠️ Falling back to NPM package import');

            // Check if package exists
            if (fs.existsSync(path.resolve(process.cwd(), 'node_modules/subconverter-wasm'))) {
                console.log('subconverter-wasm package found in node_modules');
                // List contents of package
                const pkgFiles = fs.readdirSync(path.resolve(process.cwd(), 'node_modules/subconverter-wasm'));
                console.log('Package files:', pkgFiles.join(', '));
            } else {
                console.log('subconverter-wasm package NOT found in node_modules');
            }

            const subconverter = await import('subconverter-wasm');
            console.log('✅ WASM loaded from NPM package');
            return subconverter;
        } catch (err: any) {
            console.error('Error importing from NPM package:', err);
            console.error('Error details:', err.stack);
        }

        console.error('❌ Failed to load WASM in Netlify environment');
        throw new Error('Failed to load WebAssembly module in Netlify environment');
    } else {
        // Non-Netlify environment (development, Vercel, etc.)
        console.log('🔄 Using standard WASM import');
        try {
            wasm = await import('subconverter-wasm');
            console.log('✅ WASM loaded successfully');
            return wasm;
        } catch (err: any) {
            console.error('❌ Error loading WASM:', err);
            throw err;
        }
    }
} 