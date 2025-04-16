"use client";

import Link from "next/link";
import { useState, FormEvent, useCallback, useEffect } from "react";
import { convertSubscription, SubResponseData, ErrorData, createShortUrl } from '@/lib/api-client';

// Define config presets for easy maintenance
const CONFIG_PRESETS = [
  {
    name: "ACL4SSR Online",
    url: "https://raw.githubusercontent.com/ACL4SSR/ACL4SSR/master/Clash/config/ACL4SSR_Online.ini",
    description: "Basic rules"
  },
  {
    name: "ACL4SSR Full",
    url: "https://raw.githubusercontent.com/ACL4SSR/ACL4SSR/master/Clash/config/ACL4SSR_Online_Full.ini",
    description: "Full rules"
  },
  {
    name: "ACL4SSR Mini",
    url: "https://raw.githubusercontent.com/ACL4SSR/ACL4SSR/master/Clash/config/ACL4SSR_Online_Mini.ini",
    description: "Minimal rules"
  },
  {
    name: "Divine China",
    url: "https://raw.githubusercontent.com/DivineEngine/Profiles/master/Clash/config/China.yaml",
    description: "China rules"
  },
  {
    name: "Loon Simple",
    url: "https://gist.githubusercontent.com/tindy2013/1fa08640a9088ac8652dbd40c5d2715b/raw/loon_simple.conf",
    description: "Simple Loon config"
  }
];

export default function Home() {
  const [subscriptionUrl, setSubscriptionUrl] = useState("");
  const [targetFormat, setTargetFormat] = useState("clash");
  const [configUrl, setConfigUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [result, setResult] = useState<SubResponseData | null>(null);
  const [error, setError] = useState<ErrorData | null>(null);
  const [saveApiUrl, setSaveApiUrl] = useState(true);
  const [shortUrlCreating, setShortUrlCreating] = useState(false);
  const [shortUrlCreated, setShortUrlCreated] = useState(false);

  // Reset shortUrlCreated when form inputs change
  useEffect(() => {
    setShortUrlCreated(false);
  }, [subscriptionUrl, targetFormat, configUrl]);

  // Generate the API URL based on form inputs
  const generateApiUrl = useCallback(() => {
    const baseUrl = window.location.origin + '/api/sub';
    const params = new URLSearchParams();
    params.append('target', targetFormat);
    params.append('url', subscriptionUrl);

    // Add config if set
    if (configUrl) {
      params.append('config', configUrl);
    }

    return `${baseUrl}?${params.toString()}`;
  }, [targetFormat, subscriptionUrl, configUrl]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!subscriptionUrl) return;

    setIsLoading(true);
    setResult(null);
    setError(null);
    setShortUrlCreated(false);

    try {
      // Call the actual conversion API
      const payload: any = {
        target: targetFormat,
        url: subscriptionUrl
      };

      // Add config if set
      if (configUrl) {
        payload.config = configUrl;
      }

      const responseData = await convertSubscription(payload);
      setResult(responseData);

      // If saveApiUrl is enabled, create a short URL
      if (saveApiUrl) {
        await createShortUrlForConversion();
      }
    } catch (err) {
      console.error("Conversion API call failed:", err);
      setError(err as ErrorData || {
        error: 'Failed to connect to the conversion API.',
        details: String(err)
      });
    } finally {
      setIsLoading(false);
    }
  };

  // Create a short URL for the current subscription
  const createShortUrlForConversion = async () => {
    if (!subscriptionUrl) return;

    try {
      setShortUrlCreating(true);
      const apiUrl = generateApiUrl();
      const description = `${targetFormat.toUpperCase()} subscription for ${subscriptionUrl.substring(0, 30)}${subscriptionUrl.length > 30 ? '...' : ''}`;

      await createShortUrl({
        target_url: apiUrl,
        description: description
      });

      setShortUrlCreated(true);
    } catch (err) {
      console.error("Error creating short URL:", err);
      // We don't show this error to the user to avoid confusion
      // The main conversion still succeeded
    } finally {
      setShortUrlCreating(false);
    }
  };

  const handleDownload = useCallback(() => {
    if (!result || !result.content) return;

    const blob = new Blob([result.content], { type: result.content_type || 'text/plain' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `config.${targetFormat === 'clash' ? 'yaml' : 'txt'}`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }, [result, targetFormat]);

  // The supported target formats from the convert page
  const SUPPORTED_TARGETS = [
    'auto', 'clash', 'clashr', 'surge', 'quan', 'quanx',
    'mellow', 'surfboard', 'loon', 'ss', 'ssr', 'sssub',
    'v2ray', 'trojan', 'trojan-go', 'hysteria', 'hysteria2',
    'ssd', 'mixed', 'singbox'
  ];

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-4 md:p-8 lg:p-24">
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
                {SUPPORTED_TARGETS.map(t => <option key={t} value={t}>{t}</option>)}
              </select>
            </div>

            <div>
              <label htmlFor="configUrl" className="block text-sm font-medium mb-1">
                External Config
              </label>
              <div className="flex flex-wrap gap-2 mb-2">
                {CONFIG_PRESETS.map(preset => (
                  <button
                    key={preset.name}
                    type="button"
                    onClick={() => setConfigUrl(preset.url)}
                    className={`px-3 py-1.5 text-xs rounded border transition-colors ${configUrl === preset.url
                      ? 'bg-blue-500 text-white border-blue-600'
                      : 'bg-blue-100 hover:bg-blue-200 border-blue-300 text-blue-800'
                      }`}
                    title={preset.description}
                  >
                    {preset.name}
                  </button>
                ))}
              </div>
              <input
                type="text"
                id="configUrl"
                placeholder="External configuration URL or path"
                className="w-full p-2 border border-gray-300 rounded bg-white/10"
                value={configUrl}
                onChange={(e) => setConfigUrl(e.target.value)}
              />
              <p className="mt-1 text-xs text-gray-400">
                Optional: Use a preset or enter a custom config URL
              </p>
            </div>

            <div className="flex items-center">
              <input
                id="saveApiUrl"
                type="checkbox"
                checked={saveApiUrl}
                onChange={(e) => setSaveApiUrl(e.target.checked)}
                className="h-4 w-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
              />
              <label htmlFor="saveApiUrl" className="ml-2 text-sm">
                Save as subscription
              </label>
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded disabled:opacity-50"
            >
              {isLoading ? "Converting..." : "Convert"}
            </button>
          </form>

          {error && (
            <div className="mt-6 p-4 border border-red-400 bg-red-50 rounded-md">
              <h3 className="text-lg font-semibold text-red-800">Error</h3>
              <p className="text-red-700">{error.error}</p>
              {error.details && <p className="mt-1 text-sm text-red-600">{error.details}</p>}
            </div>
          )}

          {result && !error && (
            <div className="mt-6">
              {/* API URL Display */}
              <div className="mb-4 p-3 bg-white/10 border border-gray-300 rounded-md">
                <div className="flex justify-between items-center mb-2">
                  <h4 className="font-medium">Subscription URL</h4>
                  <button
                    onClick={() => navigator.clipboard.writeText(generateApiUrl())}
                    className="text-xs px-2 py-1 bg-gray-600 hover:bg-gray-700 text-white rounded"
                  >
                    Copy
                  </button>
                </div>
                <p className="text-xs break-all font-mono bg-gray-800 p-2 rounded text-white">
                  {generateApiUrl()}
                </p>
                <p className="text-xs mt-1">
                  You can use this URL directly as a subscription link.
                  {saveApiUrl && " This URL will be saved for later use."}
                </p>
              </div>

              {/* Result preview */}
              <div className="mt-4">
                <div className="flex justify-between items-center mb-2">
                  <h4 className="font-medium">Preview</h4>
                  <div className="text-xs text-gray-400">Content-Type: {result.content_type}</div>
                </div>
                <textarea
                  readOnly
                  value={result.content}
                  rows={8}
                  className="w-full p-2 bg-gray-800 rounded font-mono text-sm text-white"
                />
                <div className="mt-2 flex justify-end">
                  <button
                    onClick={handleDownload}
                    className="bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded"
                  >
                    Download Config
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="bg-white/5 p-6 rounded-lg shadow-md">
            <h2 className="text-2xl font-semibold mb-4">Advanced Configuration</h2>
            <p className="mb-4">
              Need more options? Create a custom configuration with advanced settings.
            </p>
            <Link
              href="/convert"
              className="block bg-purple-600 hover:bg-purple-700 text-white font-bold py-2 px-4 rounded text-center"
            >
              Advanced Options
            </Link>
          </div>

          <div className={`bg-white/5 p-6 rounded-lg shadow-md ${result && (saveApiUrl || shortUrlCreated) ? 'border-2 border-green-500 bg-white/10' : ''}`}>
            <h2 className="text-2xl font-semibold mb-4">My Saved Links</h2>
            <p className="mb-4">
              View and manage your saved subscription conversion links.
              {shortUrlCreating && (
                <span className="block mt-2 text-blue-400 text-sm">
                  Creating short URL for your subscription...
                </span>
              )}
              {shortUrlCreated && (
                <span className="block mt-2 text-green-400 text-sm">
                  Your subscription has been saved as a short URL!
                </span>
              )}
            </p>
            <Link
              href="/links"
              className={`block ${result && (saveApiUrl || shortUrlCreated) ? 'bg-green-500' : 'bg-green-600'} hover:bg-green-700 text-white font-bold py-2 px-4 rounded text-center ${result && (saveApiUrl || shortUrlCreated) ? 'animate-pulse' : ''}`}
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
