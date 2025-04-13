import { NextRequest, NextResponse } from 'next/server';
import { initWasm } from '@/lib/wasm';

export async function GET(request: NextRequest) {
    try {
        const { searchParams } = new URL(request.url);
        const target = searchParams.get('target') || 'clash';
        const url = searchParams.get('url');

        if (!url) {
            return NextResponse.json(
                { error: 'Missing url parameter' },
                { status: 400 }
            );
        }

        // Initialize WASM module and use it for conversion
        const wasm = await initWasm();
        let result: string;

        if (wasm) {
            result = await wasm.convert_subscription(url, target);
        } else {
            // Fallback if WASM failed to initialize
            result = `Placeholder for conversion from ${url} to ${target}`;
        }

        return new NextResponse(result, {
            status: 200,
            headers: {
                'Content-Type': 'text/plain',
            },
        });
    } catch (error) {
        console.error('Error in subscription conversion:', error);
        return NextResponse.json(
            { error: 'Failed to convert subscription' },
            { status: 500 }
        );
    }
} 