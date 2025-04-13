"use client";

import dynamic from "next/dynamic";

// Dynamically import the DebugPanel with no SSR to avoid window is not defined errors
const DebugPanel = dynamic(() => import("@/components/DebugPanel"), {
    ssr: false,
});

export default function ClientDebugPanel() {
    return <DebugPanel />;
} 