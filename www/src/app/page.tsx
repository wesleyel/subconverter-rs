"use client";

import Link from "next/link";
import { useState, FormEvent, useCallback, useEffect } from "react";
import { convertSubscription, SubResponseData, ErrorData, createShortUrl, ShortUrlData } from '@/lib/api-client';

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
  const [shortUrlData, setShortUrlData] = useState<ShortUrlData | null>(null);

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

      const shortUrl = await createShortUrl({
        target_url: apiUrl,
        description: description
      });

      setShortUrlData(shortUrl);
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
    'clash', 'singbox', 'surge', 'quan', 'quanx',
    'mellow', 'surfboard', 'loon', 'ss', 'ssr', 'sssub',
    'v2ray', 'trojan', 'trojan-go', 'hysteria', 'hysteria2',
    'ssd', 'mixed', 'clashr'
  ];

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-4 md:p-8 lg:p-24">
      <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm">
        <div className="flex flex-col sm:flex-row items-center justify-between mb-6">
          <h1 className="text-4xl font-bold mb-4 sm:mb-0 text-center">Subconverter Web UI</h1>
          <a
            href="https://github.com/lonelam/subconverter-rs"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 bg-gray-800 hover:bg-gray-700 text-white px-4 py-2 rounded-lg transition-colors"
          >
            <svg height="24" width="24" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z"></path>
            </svg>
            <span>Star on GitHub</span>
          </a>
        </div>

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
                    onClick={() => navigator.clipboard.writeText(shortUrlData && shortUrlCreated ? shortUrlData.short_url : generateApiUrl())}
                    className="text-xs px-2 py-1 bg-gray-600 hover:bg-gray-700 text-white rounded"
                  >
                    Copy
                  </button>
                </div>
                <p className="text-xs break-all font-mono bg-gray-800 p-2 rounded text-white">
                  {shortUrlData && shortUrlCreated ? shortUrlData.short_url : generateApiUrl()}
                </p>
                <p className="text-xs mt-1">
                  You can use this URL directly as a subscription link.
                  {saveApiUrl && !shortUrlCreated && " This URL will be saved for later use."}
                  {shortUrlCreated && " This is your saved short URL that points to the full conversion URL."}
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

          <div className="bg-white/5 p-6 rounded-lg shadow-md">
            <h2 className="text-2xl font-semibold mb-4">Server Settings</h2>
            <div className="flex items-center mb-4">
              <svg className="w-6 h-6 mr-2 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"></path>
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
              </svg>
              <p>Configure server parameters, rule sets, and subscription behavior</p>
            </div>
            <Link
              href="/settings"
              className="block bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded text-center"
            >
              Manage Settings
            </Link>
          </div>

          <div className="bg-white/5 p-6 rounded-lg shadow-md">
            <h2 className="text-2xl font-semibold mb-4">App Downloads</h2>
            <div className="flex items-center mb-4">
              <svg className="w-6 h-6 mr-2 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
              </svg>
              <p>Download client apps for various platforms through our reverse proxy</p>
            </div>
            <div className="grid grid-cols-3 gap-3 mb-4">
              <button className="flex flex-col items-center justify-center p-3 bg-white/10 rounded hover:bg-white/20 transition-colors">
                <svg className="w-8 h-8 mb-1" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M21.928 11.607c-.202-.488-.635-.605-.928-.633v-1.898c0-.362-.183-.474-.333-.535-.149-.061-.395-.079-.667.039l-1.273.561c-.393.188-.857.349-1.344.349h-6.7c-.33 0-.595-.437-.781-.86-.781-1.647-1.145-5.007-1.396-6.157-.25-1.15-.908-1.15-1.434-1.15-.511 0-1.07.02-1.07 1.243 0 1.3.993 9.75 4.738 9.75h5.27c.874 0 1.245 1.697.254 1.697h-5.523c-2.689 0-3.387 2.016-3.387 2.016s5.077 3.695 5.077 4.806v1.903c0 .29.34.333.676.333h1.126c.337 0 .676-.043.676-.333v-.862c0-.292.406-.454.676-.454h1.126c.271 0 .677.162.677.454v.862c0 .29.339.333.675.333h1.126c.336 0 .676-.043.676-.333v-.862c0-.292.405-.454.676-.454h1.126c.27 0 .676.162.676.454v.862c0 .29.34.333.676.333h1.126c.337 0 .676-.043.676-.333v-4.85c0-1.263-1.375-3.115-2.071-3.668z" />
                </svg>
                <span className="text-xs">macOS</span>
              </button>
              <button className="flex flex-col items-center justify-center p-3 bg-white/10 rounded hover:bg-white/20 transition-colors">
                <svg className="w-8 h-8 mb-1" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M0 0h24v24H0V0z" fill="none" />
                  <path d="M3.9 12c0-1.71 1.39-3.1 3.1-3.1h4V7H7c-2.76 0-5 2.24-5 5s2.24 5 5 5h4v-1.9H7c-1.71 0-3.1-1.39-3.1-3.1zM8 13h8v-2H8v2zm9-6h-4v1.9h4c1.71 0 3.1 1.39 3.1 3.1s-1.39 3.1-3.1 3.1h-4V17h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z" />
                </svg>
                <span className="text-xs">Windows</span>
              </button>
              <button className="flex flex-col items-center justify-center p-3 bg-white/10 rounded hover:bg-white/20 transition-colors">
                <svg className="w-8 h-8 mb-1" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M21 18v3H3V3h18v3H10v12h11zm-9-2V8H3v8h9z" />
                </svg>
                <span className="text-xs">Linux</span>
              </button>
            </div>
            <Link
              href="/downloads"
              className="block bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded text-center"
            >
              View All Downloads
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
