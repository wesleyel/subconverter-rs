import fs from 'fs';
import path from 'path';
const __dirname = import.meta.dirname;

/** @type {import('next').NextConfig} */
const nextConfig = {
    reactStrictMode: true,
    // Allows importing wasm files from pkg directory
    transpilePackages: ['subconverter'],
    // Webpack config to support WASM
    webpack: (config, { isServer }) => {
        // Support for WebAssembly
        config.experiments = {
            ...config.experiments,
            asyncWebAssembly: true,
        };

        // Add a copy plugin to copy the WASM file to the output directory
        if (isServer) {
            // For server-side (Node.js environment)
            config.plugins.push({
                apply: (compiler) => {
                    compiler.hooks.afterEmit.tap('CopyWasmPlugin', (compilation) => {

                        // Source WASM file
                        const sourcePath = path.resolve(__dirname, '../pkg/subconverter_bg.wasm');
                        // Destination - where Next.js will look for it
                        const destDir = path.resolve(__dirname, '.next/server');
                        const destPath = path.resolve(destDir, 'subconverter_bg.wasm');

                        // Create the directory if it doesn't exist
                        if (!fs.existsSync(destDir)) {
                            fs.mkdirSync(destDir, { recursive: true });
                        }

                        // Copy the file
                        try {
                            fs.copyFileSync(sourcePath, destPath);
                            console.log(`✅ Copied WASM file from ${sourcePath} to ${destPath}`);
                        } catch (err) {
                            console.error(`❌ Error copying WASM file: ${err.message}`);
                        }
                    });
                }
            });
        }

        return config;
    },
    async rewrites() {
        // In development, rewrite requests to the root /api to our pages/api
        // This is already handled in production by Vercel
        return [
            // Rewrite all API calls to the pages/api directory
            {
                source: '/api/:path*',
                destination: '/api/:path*',
            },
        ];
    },
};

export default nextConfig; 