"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { ShortUrlData, listShortUrls, deleteShortUrl, createShortUrl, updateShortUrl } from "@/lib/api-client";
import { useRouter } from "next/navigation";

export default function SavedLinks() {
    const [links, setLinks] = useState<ShortUrlData[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [showNewLinkForm, setShowNewLinkForm] = useState(false);
    const [newLink, setNewLink] = useState({ target_url: "", custom_id: "", description: "" });
    const [editingLink, setEditingLink] = useState<ShortUrlData | null>(null);
    const router = useRouter();

    // Load links on component mount
    useEffect(() => {
        loadLinks();
    }, []);

    const loadLinks = async () => {
        try {
            setLoading(true);
            const data = await listShortUrls();
            setLinks(data);
            setError(null);
        } catch (err: any) {
            setError(err.error || "Failed to load links");
            console.error("Error loading links:", err);
        } finally {
            setLoading(false);
        }
    };

    const handleDelete = async (id: string) => {
        if (!confirm("Are you sure you want to delete this short URL?")) {
            return;
        }

        try {
            await deleteShortUrl(id);
            setLinks(links.filter(link => link.id !== id));
        } catch (err: any) {
            setError(err.error || "Failed to delete link");
            console.error("Error deleting link:", err);
        }
    };

    const handleCreateSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!newLink.target_url) {
            setError("Target URL is required");
            return;
        }

        try {
            const createdLink = await createShortUrl({
                target_url: newLink.target_url,
                custom_id: newLink.custom_id || undefined,
                description: newLink.description || undefined
            });

            setLinks([createdLink, ...links]);
            setNewLink({ target_url: "", custom_id: "", description: "" });
            setShowNewLinkForm(false);
            setError(null);
        } catch (err: any) {
            setError(err.error || "Failed to create short URL");
            console.error("Error creating short URL:", err);
        }
    };

    const handleEditSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!editingLink) return;

        try {
            const updatedLink = await updateShortUrl(editingLink.id, {
                target_url: editingLink.target_url,
                description: editingLink.description
            });

            setLinks(links.map(link =>
                link.id === updatedLink.id ? updatedLink : link
            ));
            setEditingLink(null);
            setError(null);
        } catch (err: any) {
            setError(err.error || "Failed to update short URL");
            console.error("Error updating short URL:", err);
        }
    };

    // Format timestamp to human-readable date
    const formatDate = (timestamp: number) => {
        return new Date(timestamp * 1000).toLocaleString();
    };

    return (
        <main className="flex min-h-screen flex-col items-center p-8">
            <div className="z-10 max-w-5xl w-full items-center font-mono text-sm">
                <div className="flex justify-between items-center mb-8">
                    <h1 className="text-3xl font-bold">My Short URLs</h1>
                    <div className="flex gap-2">
                        <button
                            onClick={() => setShowNewLinkForm(!showNewLinkForm)}
                            className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                        >
                            {showNewLinkForm ? "Cancel" : "Create New Short URL"}
                        </button>
                        <Link
                            href="/"
                            className="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded"
                        >
                            Back to Home
                        </Link>
                    </div>
                </div>

                {error && (
                    <div className="bg-red-500/20 border border-red-500 text-red-100 p-4 mb-6 rounded-lg">
                        {error}
                    </div>
                )}

                {showNewLinkForm && (
                    <div className="bg-white/10 p-6 rounded-lg shadow-md mb-6">
                        <h2 className="text-xl font-bold mb-4">Create New Short URL</h2>
                        <form onSubmit={handleCreateSubmit}>
                            <div className="mb-4">
                                <label className="block mb-2">Target URL*</label>
                                <input
                                    type="url"
                                    required
                                    className="w-full p-2 bg-black/30 border border-gray-700 rounded"
                                    value={newLink.target_url}
                                    onChange={(e) => setNewLink({ ...newLink, target_url: e.target.value })}
                                    placeholder="https://example.com"
                                />
                            </div>
                            <div className="mb-4">
                                <label className="block mb-2">Custom ID (optional)</label>
                                <input
                                    type="text"
                                    className="w-full p-2 bg-black/30 border border-gray-700 rounded"
                                    value={newLink.custom_id}
                                    onChange={(e) => setNewLink({ ...newLink, custom_id: e.target.value })}
                                    placeholder="my-custom-id"
                                />
                            </div>
                            <div className="mb-4">
                                <label className="block mb-2">Description (optional)</label>
                                <input
                                    type="text"
                                    className="w-full p-2 bg-black/30 border border-gray-700 rounded"
                                    value={newLink.description}
                                    onChange={(e) => setNewLink({ ...newLink, description: e.target.value })}
                                    placeholder="Description for this URL"
                                />
                            </div>
                            <button
                                type="submit"
                                className="bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded"
                            >
                                Create Short URL
                            </button>
                        </form>
                    </div>
                )}

                {editingLink && (
                    <div className="bg-white/10 p-6 rounded-lg shadow-md mb-6">
                        <h2 className="text-xl font-bold mb-4">Edit Short URL</h2>
                        <form onSubmit={handleEditSubmit}>
                            <div className="mb-4">
                                <label className="block mb-2">Target URL*</label>
                                <input
                                    type="url"
                                    required
                                    className="w-full p-2 bg-black/30 border border-gray-700 rounded"
                                    value={editingLink.target_url}
                                    onChange={(e) => setEditingLink({ ...editingLink, target_url: e.target.value })}
                                />
                            </div>
                            <div className="mb-4">
                                <label className="block mb-2">Description</label>
                                <input
                                    type="text"
                                    className="w-full p-2 bg-black/30 border border-gray-700 rounded"
                                    value={editingLink.description || ""}
                                    onChange={(e) => setEditingLink({ ...editingLink, description: e.target.value || undefined })}
                                />
                            </div>
                            <div className="flex gap-2">
                                <button
                                    type="submit"
                                    className="bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded"
                                >
                                    Save Changes
                                </button>
                                <button
                                    type="button"
                                    className="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded"
                                    onClick={() => setEditingLink(null)}
                                >
                                    Cancel
                                </button>
                            </div>
                        </form>
                    </div>
                )}

                <div className="bg-white/5 p-6 rounded-lg shadow-md">
                    {loading ? (
                        <div className="text-center py-8">
                            <p className="text-lg">Loading short URLs...</p>
                        </div>
                    ) : links.length > 0 ? (
                        <div className="space-y-4">
                            {links.map((link) => (
                                <div
                                    key={link.id}
                                    className="border border-gray-700 rounded-lg p-4 flex flex-col md:flex-row md:items-center justify-between"
                                >
                                    <div className="mb-4 md:mb-0">
                                        <h3 className="text-xl font-semibold">
                                            {link.description || link.id}
                                        </h3>
                                        <p className="text-sm text-gray-400 mt-2">
                                            Target: {link.target_url}
                                        </p>
                                        <p className="text-sm text-gray-400">
                                            Created: {formatDate(link.created_at)} •
                                            Used: {link.use_count} times
                                            {link.last_used ? ` • Last used: ${formatDate(link.last_used)}` : ''}
                                        </p>
                                    </div>
                                    <div className="flex flex-col md:flex-row gap-2">
                                        <button
                                            onClick={() => navigator.clipboard.writeText(link.short_url.startsWith('http') ? link.short_url : `${window.location.origin}${link.short_url}`)}
                                            className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                                        >
                                            Copy Link
                                        </button>
                                        <button
                                            onClick={() => setEditingLink(link)}
                                            className="bg-yellow-600 hover:bg-yellow-700 text-white font-bold py-2 px-4 rounded"
                                        >
                                            Edit
                                        </button>
                                        <button
                                            onClick={() => handleDelete(link.id)}
                                            className="bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded"
                                        >
                                            Delete
                                        </button>
                                    </div>
                                </div>
                            ))}
                        </div>
                    ) : (
                        <div className="text-center py-8">
                            <p className="text-lg mb-4">You don't have any short URLs yet.</p>
                            <button
                                onClick={() => setShowNewLinkForm(true)}
                                className="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                            >
                                Create Your First Short URL
                            </button>
                        </div>
                    )}
                </div>
            </div>
        </main>
    );
} 