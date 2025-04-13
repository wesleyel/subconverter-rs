"use client";

import Link from "next/link";
import { useState, FormEvent } from "react";

export default function Home() {
  const [subscriptionUrl, setSubscriptionUrl] = useState("");
  const [targetFormat, setTargetFormat] = useState("clash");
  const [isConverting, setIsConverting] = useState(false);
  const [resultUrl, setResultUrl] = useState("");

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!subscriptionUrl) return;

    setIsConverting(true);
    setResultUrl("");

    try {
      // Create the API URL
      const apiUrl = `/api/subconverter?url=${encodeURIComponent(subscriptionUrl)}&target=${targetFormat}`;

      // In a real implementation, we would redirect to the API URL
      // For now, just set the result URL
      setResultUrl(apiUrl);
    } catch (error) {
      console.error("Error converting subscription:", error);
    } finally {
      setIsConverting(false);
    }
  };

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
      <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm">
        <h1 className="text-4xl font-bold mb-8 text-center">Subconverter Web UI</h1>

        <div className="bg-white/5 p-6 rounded-lg shadow-md mb-8">
          <h2 className="text-2xl font-semibold mb-4">Quick Convert</h2>
          <form className="space-y-4" onSubmit={handleSubmit}>
            <div>
              <label htmlFor="subscriptionUrl" className="block text-sm font-medium mb-1">
                Subscription URL
              </label>
              <input
                type="url"
                id="subscriptionUrl"
                placeholder="https://example.com/subscription"
                className="w-full p-2 border border-gray-300 rounded bg-white/10"
                value={subscriptionUrl}
                onChange={(e) => setSubscriptionUrl(e.target.value)}
                required
              />
            </div>

            <div>
              <label htmlFor="targetFormat" className="block text-sm font-medium mb-1">
                Target Format
              </label>
              <select
                id="targetFormat"
                className="w-full p-2 border border-gray-300 rounded bg-white/10"
                value={targetFormat}
                onChange={(e) => setTargetFormat(e.target.value)}
              >
                <option value="clash">Clash</option>
                <option value="surge">Surge</option>
                <option value="quantumult">Quantumult</option>
                <option value="quanx">Quantumult X</option>
                <option value="loon">Loon</option>
                <option value="ss">SS</option>
                <option value="ssr">SSR</option>
                <option value="v2ray">V2Ray</option>
              </select>
            </div>

            <button
              type="submit"
              disabled={isConverting}
              className="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded disabled:opacity-50"
            >
              {isConverting ? "Converting..." : "Convert"}
            </button>
          </form>

          {resultUrl && (
            <div className="mt-6 p-4 bg-gray-800 rounded">
              <p className="text-sm mb-2">Your converted subscription:</p>
              <div className="flex">
                <input
                  type="text"
                  readOnly
                  value={`${window.location.origin}${resultUrl}`}
                  className="w-full p-2 bg-gray-700 rounded-l"
                />
                <button
                  onClick={() => navigator.clipboard.writeText(`${window.location.origin}${resultUrl}`)}
                  className="bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded-r"
                >
                  Copy
                </button>
              </div>
            </div>
          )}
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="bg-white/5 p-6 rounded-lg shadow-md">
            <h2 className="text-2xl font-semibold mb-4">Advanced Configuration</h2>
            <p className="mb-4">
              Create a custom configuration for your subscription conversion.
            </p>
            <Link
              href="/config"
              className="block bg-purple-600 hover:bg-purple-700 text-white font-bold py-2 px-4 rounded text-center"
            >
              Open Editor
            </Link>
          </div>

          <div className="bg-white/5 p-6 rounded-lg shadow-md">
            <h2 className="text-2xl font-semibold mb-4">My Saved Links</h2>
            <p className="mb-4">
              View and manage your saved subscription conversion links.
            </p>
            <Link
              href="/links"
              className="block bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded text-center"
            >
              Manage Links
            </Link>
          </div>
        </div>
      </div>

      <footer className="w-full text-center mt-16 text-sm text-gray-400">
        <p>
          Powered by <a href="https://github.com/lonelam/subconverter-rs" className="underline">Subconverter-RS</a>
        </p>
      </footer>
    </main>
  );
}
