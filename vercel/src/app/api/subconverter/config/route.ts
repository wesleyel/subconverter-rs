import { NextRequest, NextResponse } from 'next/server';

// Define a more specific type for config
interface ConfigItem {
    id: string;
    name: string;
    target: string;
    subscription: string;
    options: Record<string, unknown>;
    createdAt: string;
}

// In-memory storage for demo purposes
// In a real application, this would be a database
const configStore: Record<string, ConfigItem> = {};

export async function GET(request: NextRequest) {
    try {
        const { searchParams } = new URL(request.url);
        const configId = searchParams.get('id');

        if (!configId) {
            return NextResponse.json(
                { error: 'Missing config id parameter' },
                { status: 400 }
            );
        }

        const config = configStore[configId];

        if (!config) {
            return NextResponse.json(
                { error: 'Config not found' },
                { status: 404 }
            );
        }

        return NextResponse.json(config);
    } catch (error) {
        console.error('Error retrieving config:', error);
        return NextResponse.json(
            { error: 'Failed to retrieve config' },
            { status: 500 }
        );
    }
}

export async function POST(request: NextRequest) {
    try {
        const body = await request.json();

        if (!body.name || !body.target || !body.subscription) {
            return NextResponse.json(
                { error: 'Missing required fields' },
                { status: 400 }
            );
        }

        // Generate a unique ID for the config
        const configId = Math.random().toString(36).substring(2, 15);

        // Store the config with metadata
        configStore[configId] = {
            id: configId,
            name: body.name,
            target: body.target,
            subscription: body.subscription,
            options: body.options || {},
            createdAt: new Date().toISOString(),
        };

        return NextResponse.json({
            id: configId,
            message: 'Config created successfully',
        });
    } catch (error) {
        console.error('Error creating config:', error);
        return NextResponse.json(
            { error: 'Failed to create config' },
            { status: 500 }
        );
    }
}

export async function DELETE(request: NextRequest) {
    try {
        const { searchParams } = new URL(request.url);
        const configId = searchParams.get('id');

        if (!configId) {
            return NextResponse.json(
                { error: 'Missing config id parameter' },
                { status: 400 }
            );
        }

        if (!configStore[configId]) {
            return NextResponse.json(
                { error: 'Config not found' },
                { status: 404 }
            );
        }

        delete configStore[configId];

        return NextResponse.json({
            message: 'Config deleted successfully',
        });
    } catch (error) {
        console.error('Error deleting config:', error);
        return NextResponse.json(
            { error: 'Failed to delete config' },
            { status: 500 }
        );
    }
} 