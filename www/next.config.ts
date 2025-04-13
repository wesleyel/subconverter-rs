import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import type { NextConfig } from "next";
import type { Compiler } from 'webpack';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Detect environments
const isNetlify = process.env.NETLIFY === 'true' ||
  process.env.CONTEXT === 'production' ||
  process.env.NETLIFY_LOCAL === 'true' ||
  (process.env.DEPLOY_URL && process.env.DEPLOY_URL.includes('netlify'));

const isVercel = process.env.VERCEL === 'true';
const isDev = process.env.NODE_ENV === 'development';

// Log environment info
console.log('✅ Is Netlify environment:', isNetlify);
console.log('✅ Is Vercel environment:', isVercel);
console.log('✅ Is Development environment:', isDev);

// Define server directories based on Next.js output structure
const serverDirs = [
  '.next/server/chunks',
  '.next/server/pages/api',
  '.next/server/app/api',
  'public/static/wasm'
];

/**
 * Resolve the path to the subconverter-wasm package
 */
function resolveWasmPackagePath() {
  try {
    // Find the installed package path
    const packagePath = require.resolve('subconverter-wasm/package.json');
    const packageDir = path.dirname(packagePath);
    console.log(`📦 Found subconverter-wasm package at: ${packageDir}`);
    return packageDir;
  } catch (error) {
    console.error('❌ Error resolving subconverter-wasm package:', error);
    return null;
  }
}

// WASM integration plugin for Next.js
class WasmIntegrationPlugin {
  apply(compiler: Compiler) {
    const pluginName = 'WasmIntegrationPlugin';

    compiler.hooks.afterEmit.tap(pluginName, (compilation) => {
      console.log('🔌 WasmIntegrationPlugin running after webpack emit');

      try {
        // Resolve the package path
        const packageDir = resolveWasmPackagePath();
        if (!packageDir) {
          console.error('❌ Could not resolve subconverter-wasm package path');
          return;
        }

        // Look for WASM files in the package directory
        let wasmDir = packageDir;

        // Check if there's a specific directory for WASM files
        const possibleWasmDirs = [
          path.join(packageDir, 'dist'),
          path.join(packageDir, 'pkg'),
          packageDir
        ];

        for (const dir of possibleWasmDirs) {
          if (fs.existsSync(dir) &&
            fs.readdirSync(dir).some(file => file.endsWith('.wasm'))) {
            wasmDir = dir;
            console.log(`📁 Found WASM files in: ${wasmDir}`);
            break;
          }
        }

        // Find all WASM files
        const wasmFiles = fs.readdirSync(wasmDir)
          .filter(file => file.endsWith('.wasm'));

        if (wasmFiles.length === 0) {
          console.warn(`⚠️ No WASM files found in ${wasmDir}`);
          return;
        }

        console.log(`Found ${wasmFiles.length} WASM files: ${wasmFiles.join(', ')}`);

        // Integration path: target Next.js server output structure
        for (const wasmFile of wasmFiles) {
          // Integrate WASM files into Next.js server output structure
          integrateToBuildOutput(
            path.join(wasmDir, wasmFile),
            wasmFile,
            serverDirs
          );
        }

        // Special handling for Netlify
        if (isNetlify) {
          // Integrate WASM into Netlify function locations
          const netlifyFunctionDirs = [
            '.netlify/functions-internal/subconverter',
            '.netlify/functions/subconverter',
          ];

          for (const wasmFile of wasmFiles) {
            integrateToBuildOutput(
              path.join(wasmDir, wasmFile),
              wasmFile,
              netlifyFunctionDirs
            );
          }
        }
      } catch (error) {
        console.error('Error integrating WASM files:', error);
      }
    });
  }
}

/**
 * Integrates a file into the Next.js build output directories
 */
function integrateToBuildOutput(sourceFile: string, fileName: string, targetDirs: string[]) {
  if (!fs.existsSync(sourceFile)) {
    console.error(`⚠️ Source file not found: ${sourceFile}`);
    return;
  }

  const sourceSize = fs.statSync(sourceFile).size;
  console.log(`📂 Integrating file: ${sourceFile}, size: ${(sourceSize / 1024).toFixed(2)} KB`);

  let successCount = 0;
  for (const dir of targetDirs) {
    try {
      // Create target directory if it doesn't exist
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
        console.log(`📁 Created directory: ${dir}`);
      }

      // Copy file to target directory
      const targetFile = path.join(dir, fileName);
      fs.copyFileSync(sourceFile, targetFile);

      // Verify copy was successful
      if (fs.existsSync(targetFile)) {
        const targetSize = fs.statSync(targetFile).size;
        console.log(`✅ Integrated to: ${targetFile}, size: ${(targetSize / 1024).toFixed(2)} KB`);

        if (targetSize === sourceSize) {
          successCount++;
        } else {
          console.warn(`⚠️ File size mismatch for ${targetFile}`);
        }
      }
    } catch (error) {
      console.error(`❌ Error integrating to ${dir}:`, error);
    }
  }

  console.log(`📊 Integration summary: ${successCount}/${targetDirs.length} successful`);
}

const nextConfig: NextConfig = {
  reactStrictMode: true,
  // Allows importing wasm files from pkg directory
  transpilePackages: ['subconverter-wasm'],
  // Webpack config to support WASM
  webpack: (config, { isServer, dev }) => {
    console.log(`⚙️ Configuring webpack (isServer: ${isServer}, dev: ${dev})`);

    // Support for WebAssembly
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
      layers: true,
      topLevelAwait: true,
    };

    // Configure WASM output location
    if (config.output) {
      // Ensure WASM is properly emitted to a predictable location
      config.output.webassemblyModuleFilename = 'static/wasm/[modulehash].wasm';

      // Make paths more predictable for Netlify
      if (isNetlify) {
        config.output.webassemblyModuleFilename = 'static/wasm/subconverter_bg.wasm';
      }
    }

    // Apply our integration plugin when building the server
    if (isServer) {
      console.log('🔄 Server-side build detected, setting up WASM file integration');
      config.plugins = config.plugins || [];
      config.plugins.push(new WasmIntegrationPlugin());
    }

    return config;
  },
  async rewrites() {
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
