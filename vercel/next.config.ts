import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import type { NextConfig } from "next";
import type { Compiler } from 'webpack';
import withRspack from 'next-rspack';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Define compiler hook types
interface CompilationAssets {
  [key: string]: any;
}

interface Compilation {
  assets: CompilationAssets;
}

// Function to log directory contents for debugging
function logDirectoryContents(dirPath: string, prefix = '') {
  try {
    if (fs.existsSync(dirPath)) {
      console.log(`${prefix}📂 Contents of ${dirPath}:`);
      const items = fs.readdirSync(dirPath);
      items.forEach(item => {
        const itemPath = path.join(dirPath, item);
        const stats = fs.statSync(itemPath);
        if (stats.isDirectory()) {
          console.log(`${prefix}  📁 ${item}/`);
        } else {
          console.log(`${prefix}  📄 ${item} (${stats.size} bytes)`);
        }
      });
    } else {
      console.log(`${prefix}❌ Directory not found: ${dirPath}`);
    }
  } catch (error) {
    console.error(`${prefix}❌ Error reading directory: ${dirPath}`, error);
  }
}

const nextConfig: NextConfig = {
  reactStrictMode: true,
  // Allows importing wasm files from pkg directory
  transpilePackages: ['subconverter-wasm'],
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
        apply: (compiler: Compiler) => {
          compiler.hooks.afterEmit.tap('CopyWasmPlugin', (compilation: Compilation) => {
            // Log current directory for debugging
            console.log('Current working directory:', process.cwd());
            logDirectoryContents(process.cwd(), '  ');

            // Possible source paths for the WASM file
            const possibleSourcePaths = [
              // Local development path
              path.resolve(__dirname, '../pkg/subconverter_bg.wasm'),
              // Public directory path (for Vercel)
              path.resolve(__dirname, 'public/wasm/subconverter_bg.wasm'),
              // Try with process.cwd()
              path.resolve(process.cwd(), 'public/wasm/subconverter_bg.wasm')
            ];

            // Find first existing source path
            let sourcePath = null;
            for (const potentialPath of possibleSourcePaths) {
              console.log(`Checking for WASM at: ${potentialPath}`);
              if (fs.existsSync(potentialPath)) {
                sourcePath = potentialPath;
                console.log(`✅ Found WASM source at: ${sourcePath}`);
                break;
              }
            }

            if (!sourcePath) {
              console.warn('⚠️ Could not find WASM file in any expected location.');
              return;
            }

            // Multiple destination paths for different environments
            const destinations = [
              // For development
              path.resolve(__dirname, '.next/server/subconverter_bg.wasm'),
              // For Vercel production (.output)
              path.resolve(__dirname, '.output/server/subconverter_bg.wasm'),
              // For Vercel serverless functions
              path.resolve(__dirname, '.vercel/output/functions/api/subconverter_bg.wasm'),
              // Alternative Vercel serverless path
              path.resolve(process.cwd(), '.vercel/output/functions/api/subconverter_bg.wasm')
            ];

            // Copy to each destination
            for (const destPath of destinations) {
              const destDir = path.dirname(destPath);
              // Create the directory if it doesn't exist
              if (!fs.existsSync(destDir)) {
                fs.mkdirSync(destDir, { recursive: true });
                console.log(`📁 Created directory: ${destDir}`);
              }

              // Copy the file
              try {
                fs.copyFileSync(sourcePath, destPath);
                console.log(`✅ Copied WASM file from ${sourcePath} to ${destPath}`);

                // Log the file size to confirm copy worked
                const stats = fs.statSync(destPath);
                console.log(`  📊 Copied file size: ${stats.size} bytes`);
              } catch (err: unknown) {
                const errorMessage = err instanceof Error ? err.message : String(err);
                console.error(`❌ Error copying WASM file to ${destPath}: ${errorMessage}`);
              }
            }
          });
        }
      });
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

export default withRspack(nextConfig);
