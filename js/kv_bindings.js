// Example: kv_bindings.js (or integrate into your Edge Function handler)
// Ensure @vercel/kv is installed in your Vercel project dependencies (package.json)

let kv; // Lazy load KV

async function getKv() {
    if (!kv) {
        // Dynamic import for environments where top-level await might not be supported
        // or to ensure it's only imported when needed.
        const module = await import('@vercel/kv');
        kv = module.kv;
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
        // @vercel/kv might return null if not found
        const value = await kvClient.get(key);
        // console.log(`kv_get ${key}:`, value);
        // We need to return something serde_wasm_bindgen::from_value can handle.
        // If value is raw bytes (e.g., stored via REST API as base64),
        // you might need to decode it first and return a Uint8Array or similar.
        // If @vercel/kv returns JS objects/arrays/buffers directly, this might just work.
        // Returning null/undefined for not found.

        // Check if the value looks like it could be binary data stored directly
        // (e.g., ArrayBuffer or typed array if the SDK supports it)
        if (value instanceof ArrayBuffer) {
            return new Uint8Array(value);
        } else if (ArrayBuffer.isView(value) && !(value instanceof DataView)) {
            // Handles TypedArrays like Uint8Array directly
            return value;
        }

        // Return null/undefined for not found, or the value as is otherwise
        // Let serde_wasm_bindgen handle the conversion for other types
        return value === null ? undefined : value;
    } catch (error) {
        console.error(`KV get error for ${key}:`, error);
        throw error; // Propagate error to Rust
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
        // console.log(`kv_set ${key}:`, value);    
        // Pass the Uint8Array directly. @vercel/kv might handle it
        // If not, conversion to ArrayBuffer or Base64 might be needed:
        // await kvClient.set(key, value.buffer); 
        // await kvClient.set(key, Buffer.from(value).toString('base64'));
        await kvClient.set(key, value);
    } catch (error) {
        console.error(`KV set error for ${key}:`, error);
        throw error;
    }
}

export async function kv_exists(key) {
    try {
        const kvClient = await getKv();
        const exists = await kvClient.exists(key);
        // console.log(`kv_exists ${key}:`, exists);
        // kv.exists returns the number of keys found (0 or 1 for a single key)
        return exists > 0;
    } catch (error) {
        console.error(`KV exists error for ${key}:`, error);
        throw error;
    }
}

export async function kv_list(prefix) {
    try {
        const kvClient = await getKv();
        // console.log(`kv_list prefix: ${prefix}`);

        // Vercel KV doesn't have a native list method with prefix filtering
        // We need to use the scan method instead
        let cursor = 0;
        const keys = [];
        let scanResult;

        do {
            // Use SCAN with MATCH to find keys with the given prefix
            scanResult = await kvClient.scan(cursor, {
                match: `${prefix}*`,
                count: 100, // Limit number of keys per scan
            });

            cursor = scanResult[0]; // Update cursor for next iteration
            const resultKeys = scanResult[1]; // Array of keys from this scan

            if (resultKeys && resultKeys.length > 0) {
                keys.push(...resultKeys);
            }
        } while (cursor !== '0'); // Continue until cursor becomes '0'

        // console.log(`kv_list found ${keys.length} keys with prefix ${prefix}`);
        return keys;
    } catch (error) {
        console.error(`KV list error for prefix ${prefix}:`, error);
        throw error;
    }
}

export async function kv_del(key) {
    try {
        const kvClient = await getKv();
        // console.log(`kv_del ${key}`);
        await kvClient.del(key);
    } catch (error) {
        console.error(`KV del error for ${key}:`, error);
        throw error;
    }
}

// Use global fetch available in Edge runtime
export async function fetch_url(url) {
    try {
        // console.log(`fetch_url: ${url}`);
        const response = await fetch(url);
        // We need to pass the Response object back to Rust
        // wasm-bindgen can handle some JS objects, Response might work
        // Returning the response object directly
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