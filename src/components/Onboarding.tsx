import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { useEffect, useState } from "react";

const SETTINGS_URL =
  "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture";

function Onboarding({ onGranted }: { onGranted: () => void }) {
  const [checking, setChecking] = useState(false);

  useEffect(() => {
    const onFocus = async () => {
      setChecking(true);
      try {
        const granted = await invoke<boolean>("check_screen_permission");
        if (granted) onGranted();
      } finally {
        setChecking(false);
      }
    };
    window.addEventListener("focus", onFocus);
    return () => window.removeEventListener("focus", onFocus);
  }, [onGranted]);

  const openSettings = async () => {
    try {
      await open(SETTINGS_URL);
    } catch {
      // Fallback: user must navigate manually
    }
  };

  return (
    <div className="flex flex-col items-center justify-center h-screen p-8 bg-white dark:bg-gray-900">
      <h1 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
        QRSnap needs Screen Recording
      </h1>
      <p className="text-sm text-gray-600 dark:text-gray-400 text-center mb-6 max-w-xs">
        To scan QR codes, QRSnap needs permission to capture what's on your
        screen. Open System Settings, find QRSnap, and toggle it on.
      </p>
      <button
        onClick={openSettings}
        disabled={checking}
        className="px-4 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 disabled:opacity-50"
      >
        {checking ? "Checking..." : "Open System Settings"}
      </button>
    </div>
  );
}

export default Onboarding;
