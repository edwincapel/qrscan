import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import Onboarding from "./components/Onboarding";

function App() {
  const [permissionGranted, setPermissionGranted] = useState<boolean | null>(
    null,
  );

  useEffect(() => {
    invoke<boolean>("check_screen_permission").then(setPermissionGranted);
  }, []);

  const handleGranted = useCallback(() => {
    setPermissionGranted(true);
  }, []);

  if (permissionGranted === null) {
    return null;
  }

  if (!permissionGranted) {
    return <Onboarding onGranted={handleGranted} />;
  }

  return (
    <div className="flex items-center justify-center h-screen bg-gray-50 dark:bg-gray-900">
      <p className="text-gray-600 dark:text-gray-400 text-sm">
        QRSnap is running in the menu bar.
      </p>
    </div>
  );
}

export default App;
