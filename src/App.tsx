import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import Onboarding from "./components/Onboarding";

interface ScanResult {
  image_path: string;
  source_type: string;
}

function App() {
  const [permissionGranted, setPermissionGranted] = useState<boolean | null>(
    null,
  );
  const [lastScan, setLastScan] = useState<ScanResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<boolean>("check_screen_permission").then(setPermissionGranted);
  }, []);

  const triggerScan = useCallback(async (mode: string) => {
    setError(null);
    setLastScan(null);
    try {
      const result = await invoke<ScanResult>("trigger_scan", { mode });
      setLastScan(result);
    } catch (e) {
      const msg = String(e);
      if (msg === "cancelled") return;
      if (msg === "permission_denied") {
        setPermissionGranted(false);
        return;
      }
      setError(msg);
    }
  }, []);

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    listen("scan-region", () => triggerScan("region")).then((u) =>
      unsubs.push(u),
    );
    listen("scan-window", () => triggerScan("window")).then((u) =>
      unsubs.push(u),
    );
    return () => unsubs.forEach((u) => u());
  }, [triggerScan]);

  const handleGranted = useCallback(() => {
    setPermissionGranted(true);
  }, []);

  if (permissionGranted === null) return null;
  if (!permissionGranted) return <Onboarding onGranted={handleGranted} />;

  return (
    <div className="flex flex-col items-center justify-center h-screen p-4 bg-gray-50 dark:bg-gray-900">
      {lastScan && (
        <p className="text-green-600 dark:text-green-400 text-xs mb-2">
          Captured: {lastScan.source_type} → {lastScan.image_path}
        </p>
      )}
      {error && (
        <p className="text-red-600 dark:text-red-400 text-xs mb-2">
          {error}
        </p>
      )}
      <p className="text-gray-600 dark:text-gray-400 text-sm">
        QRSnap is running in the menu bar.
      </p>
    </div>
  );
}

export default App;
