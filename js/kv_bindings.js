// Example: kv_bindings.js (or integrate into your Edge Function handler)
// Ensure @vercel/kv is installed in your Vercel project dependencies (package.json)
// Ensures fallback for local development when Vercel KV isn't available

let kv; // Lazy load KV
let localStorageMap = new Map(); // Local in-memory fallback

async function getKv() {
    if (!kv) {
        try {
            // Check if required environment variables are set
            if (typeof process !== 'undefined' &&
                process.env.KV_REST_API_URL &&
                process.env.KV_REST_API_TOKEN) {
                // Dynamic import for environments where top-level await might not be supported
                const module = await import('@vercel/kv');
                kv = module.kv;
                console.log("Using Vercel KV for storage");
            } else {
                // Use local storage fallback
                console.log("Vercel KV environment variables missing, using in-memory fallback");
                // Create an in-memory implementation that mimics the Vercel KV API
                kv = {
                    // Get a value by key
                    get: async (key) => {
                        return localStorageMap.get(key) || null;
                    },
                    // Set a key-value pair
                    set: async (key, value) => {
                        localStorageMap.set(key, value);
                        return "OK";
                    },
                    // Check if a key exists
                    exists: async (key) => {
                        return localStorageMap.has(key) ? 1 : 0;
                    },
                    // Scan keys with pattern matching
                    scan: async (cursor, options = {}) => {
                        const { match = "*", count = 10 } = options;

                        // Convert glob pattern to regex
                        const pattern = match.replace(/\*/g, ".*");
                        const regex = new RegExp(`^${pattern}$`);

                        // Get all keys that match the pattern
                        const allKeys = [...localStorageMap.keys()];
                        const matchingKeys = allKeys.filter(key => regex.test(key));

                        // Implement cursor-based pagination
                        const startIndex = parseInt(cursor) || 0;
                        const endIndex = Math.min(startIndex + count, matchingKeys.length);
                        const keys = matchingKeys.slice(startIndex, endIndex);

                        // Return next cursor or '0' if we're done
                        const nextCursor = endIndex < matchingKeys.length ? String(endIndex) : '0';

                        return [nextCursor, keys];
                    },
                    // Delete a key
                    del: async (key) => {
                        return localStorageMap.delete(key) ? 1 : 0;
                    }
                };
            }
        } catch (error) {
            console.warn("Error initializing Vercel KV, using in-memory fallback:", error);
            // Create an in-memory implementation that mimics the Vercel KV API
            kv = {
                get: async (key) => localStorageMap.get(key) || null,
                set: async (key, value) => {
                    localStorageMap.set(key, value);
                    return "OK";
                },
                exists: async (key) => localStorageMap.has(key) ? 1 : 0,
                scan: async (cursor, options = {}) => {
                    const { match = "*", count = 10 } = options;
                    const pattern = match.replace(/\*/g, ".*");
                    const regex = new RegExp(`^${pattern}$`);
                    const allKeys = [...localStorageMap.keys()];
                    const matchingKeys = allKeys.filter(key => regex.test(key));
                    const startIndex = parseInt(cursor) || 0;
                    const endIndex = Math.min(startIndex + count, matchingKeys.length);
                    const keys = matchingKeys.slice(startIndex, endIndex);
                    const nextCursor = endIndex < matchingKeys.length ? String(endIndex) : '0';
                    return [nextCursor, keys];
                },
                del: async (key) => localStorageMap.delete(key) ? 1 : 0
            };
        }
    }
    return kv;
}

// Helper to handle potential null from kv.get
// Vercel KV stores raw bytes as base64 strings when using the REST API directly,
// but the @vercel/kv SDK might handle JSON/buffers.
// We assume here it returns JS objects/primitives or null.
// If storing raw bytes, you might need base64 encoding/decoding.
// For simplicity, assuming kv_set takes bytes and kv_get returns something
// serde_wasm_bindgen can handle (like null or a JS object/array containing bytes).
export async function kv_get(key) {
    try {
        const kvClient = await getKv();
        const value = await kvClient.get(key);

        if (value instanceof ArrayBuffer) {
            return new Uint8Array(value);
        } else if (ArrayBuffer.isView(value) && !(value instanceof DataView)) {
            return value;
        }

        return value === null ? undefined : value;
    } catch (error) {
        console.error(`KV get error for ${key}:`, error);
        return undefined;
    }
}

// @vercel/kv set expects serializable JSON by default.
// If you want to store raw bytes, you might need to use the REST API
// or see if @vercel/kv has options for raw buffer storage, possibly involving base64.
// This example assumes the value passed from Rust (as &[u8])
// might need conversion before storing, or perhaps kv.set handles Uint8Array directly.
// For simplicity, let's assume kv.set can handle it or you adapt it.
export async function kv_set(key, value /* Uint8Array from Rust */) {
    try {
        const kvClient = await getKv();
        await kvClient.set(key, value);
    } catch (error) {
        console.error(`KV set error for ${key}:`, error);
    }
}

export async function kv_exists(key) {
    try {
        const kvClient = await getKv();
        const exists = await kvClient.exists(key);
        return exists > 0;
    } catch (error) {
        console.error(`KV exists error for ${key}:`, error);
        return false;
    }
}

export async function kv_list(prefix) {
    try {
        const kvClient = await getKv();
        let cursor = 0;
        const keys = [];
        let scanResult;

        do {
            // Use SCAN with MATCH to find keys with the given prefix
            scanResult = await kvClient.scan(cursor, {
                match: `${prefix}*`,
                count: 100 // Limit number of keys per scan
            });

            cursor = scanResult[0]; // Update cursor for next iteration
            const resultKeys = scanResult[1]; // Array of keys from this scan

            if (resultKeys && resultKeys.length > 0) {
                keys.push(...resultKeys);
            }
        } while (cursor !== '0'); // Continue until cursor becomes '0'

        return keys;
    } catch (error) {
        console.error(`KV list error for prefix ${prefix}:`, error);
        return [];
    }
}

export async function kv_del(key) {
    try {
        const kvClient = await getKv();
        await kvClient.del(key);
    } catch (error) {
        console.error(`KV del error for ${key}:`, error);
    }
}

// Use global fetch available in Edge runtime
export async function fetch_url(url) {
    try {
        const response = await fetch(url);
        return response;
    } catch (error) {
        console.error(`Fetch error for ${url}:`, error);
        throw error;
    }
}

// Helper to get status from Response
export async function response_status(response /* Response object */) {
    // Add type check for robustness
    if (!(response instanceof Response)) {
        throw new Error("Input is not a Response object");
    }
    return response.status;
}

// Helper to get body as bytes (Uint8Array) from Response
export async function response_bytes(response /* Response object */) {
    // Add type check for robustness
    if (!(response instanceof Response)) {
        throw new Error("Input is not a Response object");
    }
    try {
        const buffer = await response.arrayBuffer();
        return new Uint8Array(buffer);
    } catch (error) {
        console.error(`Error reading response body:`, error);
        throw error;
    }
} 