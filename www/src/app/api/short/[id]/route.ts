import { NextRequest, NextResponse } from 'next/server';
import { loadWasmSingleton } from '@/lib/wasm';

// Define correct types for route parameters according to Next.js 15
type RouteParams = any;

/**
 * Handle getting a specific short URL
 */
export async function GET(
    request: NextRequest,
    { params }: RouteParams
) {
    const id = params.id;

    if (!id) {
        return NextResponse.json(
            { error: 'Missing short URL ID' },
            { status: 400 }
        );
    }

    try {
        // Load the WASM module
        const wasmModule = await loadWasmSingleton('ShortURL');

        // Get short URLs list
        const response = await wasmModule.short_url_list();
        const data = JSON.parse(response);

        // Find the specific short URL
        const shortUrl = data.urls.find((url: any) => url.id === id);

        if (!shortUrl) {
            return NextResponse.json(
                { error: 'Short URL not found' },
                { status: 404 }
            );
        }

        // Add full URL with the base URL from the request
        const baseUrl = new URL(request.url).origin;
        shortUrl.short_url = `${baseUrl}/api/s/${shortUrl.id}`;

        return NextResponse.json(shortUrl);
    } catch (error: any) {
        console.error(`Error getting short URL:`, error);
        const errorMessage = typeof error === 'string' ? error : (error.message || 'Unknown error');

        return NextResponse.json(
            { error: 'Failed to get short URL', details: errorMessage },
            { status: 500 }
        );
    }
}

/**
 * Handle updating a specific short URL
 */
export async function PUT(
    request: NextRequest,
    { params }: RouteParams
) {
    const id = params.id;

    if (!id) {
        return NextResponse.json(
            { error: 'Missing short URL ID' },
            { status: 400 }
        );
    }

    try {
        // Load the WASM module
        const wasmModule = await loadWasmSingleton('ShortURL');

        // Parse the request body
        const body = await request.text();

        // Call the WASM function to update the short URL
        const response = await wasmModule.short_url_update(id, body);

        // Parse the response
        const data = JSON.parse(response);

        // Add full URL with the base URL from the request
        const baseUrl = new URL(request.url).origin;
        data.short_url = `${baseUrl}/api/s/${id}`;

        return NextResponse.json(data);
    } catch (error: any) {
        console.error(`Error updating short URL:`, error);
        const errorMessage = typeof error === 'string' ? error : (error.message || 'Unknown error');

        return NextResponse.json(
            { error: 'Failed to update short URL', details: errorMessage },
            { status: 500 }
        );
    }
}

/**
 * Handle deleting a specific short URL
 */
export async function DELETE(
    request: NextRequest,
    { params }: RouteParams
) {
    const id = params.id;

    if (!id) {
        return NextResponse.json(
            { error: 'Missing short URL ID' },
            { status: 400 }
        );
    }

    try {
        // Load the WASM module
        const wasmModule = await loadWasmSingleton('ShortURL');

        // Call the WASM function to delete the short URL
        const response = await wasmModule.short_url_delete(id);

        // Parse the response
        const data = JSON.parse(response);

        return NextResponse.json(data);
    } catch (error: any) {
        console.error(`Error deleting short URL:`, error);
        const errorMessage = typeof error === 'string' ? error : (error.message || 'Unknown error');

        return NextResponse.json(
            { error: 'Failed to delete short URL', details: errorMessage },
            { status: 500 }
        );
    }
} 