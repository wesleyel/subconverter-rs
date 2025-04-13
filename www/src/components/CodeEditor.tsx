"use client";

import React, { useState, useEffect } from 'react';
import { FileAttributes } from 'subconverter-wasm';
import * as apiClient from '@/lib/api-client';

interface CodeEditorProps {
    filePath: string | null;
}

export default function CodeEditor({ filePath }: CodeEditorProps) {
    const [content, setContent] = useState<string>('');
    const [loading, setLoading] = useState(false);
    const [saving, setSaving] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [fileAttributes, setFileAttributes] = useState<FileAttributes | null>(null);

    // Load file content when filePath changes
    useEffect(() => {
        if (!filePath) {
            setContent('');
            setError(null);
            setFileAttributes(null);
            return;
        }

        const loadFile = async () => {
            setLoading(true);
            setError(null);
            try {
                // Get file content
                const fileContent = await apiClient.readFile(filePath);
                setContent(fileContent || '');

                // Get file attributes if available
                try {
                    const attributes = await apiClient.getFileAttributes(filePath);
                    setFileAttributes(attributes);
                } catch (attrError) {
                    console.error('Error loading file attributes:', attrError);
                    setFileAttributes(null);
                    // Don't block file loading if attributes fail
                }
            } catch (err) {
                setError(err instanceof Error ? err.message : 'Failed to load file');
                console.error('Error loading file:', err);
            } finally {
                setLoading(false);
            }
        };

        loadFile();
    }, [filePath]);

    // Save file content
    const handleSave = async () => {
        if (!filePath) return;

        setSaving(true);
        setError(null);
        try {
            await apiClient.writeFile(filePath, content);

            // Refresh file attributes after save
            try {
                const attributes = await apiClient.getFileAttributes(filePath);
                setFileAttributes(attributes);
            } catch (attrError) {
                console.error('Error refreshing file attributes:', attrError);
            }
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to save file');
            console.error('Error saving file:', err);
        } finally {
            setSaving(false);
        }
    };

    return (
        <div className="h-full flex flex-col">
            {/* Header with file info and save button */}
            <div className="flex justify-between items-center p-2 border-b border-gray-700">
                <div className="flex items-center space-x-2 overflow-hidden">
                    <h3 className="text-sm font-semibold truncate text-gray-200">
                        {filePath || 'No file selected'}
                    </h3>
                    {fileAttributes && (
                        <div className="text-xs bg-gray-700 text-gray-200 px-2 py-0.5 rounded">
                            {apiClient.formatFileSize(fileAttributes.size)}
                        </div>
                    )}
                </div>
                {filePath && (
                    <button
                        className={`px-3 py-1 rounded text-sm ${saving
                            ? 'bg-gray-600 text-gray-300 cursor-not-allowed'
                            : 'bg-blue-600 hover:bg-blue-700 text-white'
                            }`}
                        onClick={handleSave}
                        disabled={saving || loading}
                    >
                        {saving ? 'Saving...' : 'Save'}
                    </button>
                )}
            </div>

            {/* Editor area */}
            <div className="flex-grow relative">
                {loading ? (
                    <div className="absolute inset-0 flex items-center justify-center">
                        <div className="flex flex-col items-center">
                            <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-blue-500 mb-2"></div>
                            <div className="text-gray-300">Loading...</div>
                        </div>
                    </div>
                ) : error ? (
                    <div className="absolute inset-0 flex items-center justify-center text-red-400 p-4 text-center bg-gray-800">
                        {error}
                    </div>
                ) : !filePath ? (
                    <div className="absolute inset-0 flex items-center justify-center text-gray-300 p-4 text-center">
                        Select a file to edit
                    </div>
                ) : (
                    <textarea
                        className="w-full h-full p-4 bg-gray-800 text-gray-200 resize-none font-mono text-sm focus:outline-none border-none"
                        value={content}
                        onChange={(e) => setContent(e.target.value)}
                    />
                )}
            </div>

            {/* File info footer */}
            {fileAttributes && (
                <div className="border-t border-gray-700 px-2 py-1 text-xs text-gray-300 flex justify-between">
                    <div>Type: {fileAttributes.file_type || 'Unknown'}</div>
                    <div>
                        Modified: {apiClient.formatTimestamp(Number(fileAttributes.modified_at))}
                    </div>
                </div>
            )}
        </div>
    );
} 