import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow, primaryMonitor, LogicalPosition } from "@tauri-apps/api/window";
import { useCallback, useEffect, useState } from "react";
import Onboarding from "./components/Onboarding";
import ResultPanel from "./components/ResultPanel";
import ConfirmDialog from "./components/ConfirmDialog";
import NoQrFound from "./components/NoQrFound";
import HistoryPanel from "./components/HistoryPanel";
import SettingsPanel from "./components/SettingsPanel";
import Toast from "./components/Toast";
import { decodeQR } from "./lib/decoder";

interface ScanResult { image_data: string; source_type: string }
interface ActionDef { id: string; label: string; payload: string; requires_confirmation: boolean; confirmation_message?: string }
interface ParsedContent { content_type: string; raw: string; display_text: string; actions: ActionDef[]; fields?: Record<string, string>; warnings?: string[] }

type ViewState =
  | { kind: "idle" }
  | { kind: "scanning"; sourceType: string }
  | { kind: "result"; content: ParsedContent; sourceType: string }
  | { kind: "no_qr"; sourceType: string }
  | { kind: "error"; message: string; sourceType: string };

async function showPanel() {
  const win = getCurrentWindow();
  try {
    const monitor = await primaryMonitor();
    if (monitor) {
      const scale = monitor.scaleFactor;
      const screenW = monitor.size.width / scale;
      await win.setPosition(new LogicalPosition(screenW - 440, 30));
    }
  } catch (e) {
    console.error("Position error:", e);
  }
  await win.setAlwaysOnTop(true);
  await win.show();
}

async function hidePanel() {
  const win = getCurrentWindow();
  await win.setAlwaysOnTop(false);
  await win.hide();
}

function App() {
  const [permOk, setPermOk] = useState<boolean | null>(null);
  const [view, setView] = useState<ViewState>({ kind: "idle" });
  const [confirmAction, setConfirmAction] = useState<ActionDef | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [showHistory, setShowHistory] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 4000);
  };

  // Check permission — with timeout fallback so we never get stuck loading
  useEffect(() => {
    const timer = setTimeout(() => {
      console.warn("check_screen_permission timed out — assuming granted");
      setPermOk(true);
    }, 3000);
    invoke<boolean>("check_screen_permission")
      .then(ok => { clearTimeout(timer); setPermOk(ok); })
      .catch(e => { clearTimeout(timer); console.error("Permission check failed:", e); setPermOk(true); });
    return () => clearTimeout(timer);
  }, []);

  // Show/hide window based on active panel state
  useEffect(() => {
    const hasPanel =
      view.kind !== "idle" ||
      showHistory ||
      showSettings ||
      confirmAction !== null ||
      permOk === false;
    if (hasPanel) {
      showPanel().catch(e => console.error("showPanel failed:", e));
    } else {
      hidePanel().catch(e => console.error("hidePanel failed:", e));
    }
  }, [view, showHistory, showSettings, confirmAction, permOk]);

  const triggerScan = useCallback(async (mode: string) => {
    setView({ kind: "scanning", sourceType: mode });
    setConfirmAction(null);
    try {
      const scan = await invoke<ScanResult>("trigger_scan", { mode });
      const decoded = await decodeQR(scan.image_data);
      if (!decoded) {
        setView({ kind: "no_qr", sourceType: mode });
        return;
      }
      const parsed = await invoke<ParsedContent>("parse_qr_content", { raw: decoded.text });
      setView({ kind: "result", content: parsed, sourceType: mode });
    } catch (e) {
      const msg = String(e);
      if (msg === "cancelled") { setView({ kind: "idle" }); return; }
      if (msg === "permission_denied") { setPermOk(false); setView({ kind: "idle" }); return; }
      console.error("Scan error:", msg);
      setView({ kind: "error", message: msg, sourceType: mode });
    }
  }, []);

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    listen("scan-region", () => triggerScan("region")).then(u => unsubs.push(u));
    listen("scan-window", () => triggerScan("window")).then(u => unsubs.push(u));
    listen("show-history", () => setShowHistory(true)).then(u => unsubs.push(u));
    listen("show-settings", () => setShowSettings(true)).then(u => unsubs.push(u));
    listen<string>("shortcut-conflict", e => showToast(e.payload)).then(u => unsubs.push(u));
    return () => unsubs.forEach(u => u());
  }, [triggerScan]);

  // Loading state — show spinner instead of blank white
  if (permOk === null) {
    return (
      <div className="flex items-center justify-center h-screen bg-white dark:bg-gray-900">
        <div className="text-sm text-gray-400">Starting QRSnap…</div>
      </div>
    );
  }

  if (!permOk) return <Onboarding onGranted={() => setPermOk(true)} />;

  return (
    <div className="bg-white dark:bg-gray-900 min-h-screen">
      {/* Scanning indicator */}
      {view.kind === "scanning" && (
        <div className="fixed inset-0 flex items-center justify-center">
          <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-6 text-center">
            <div className="text-2xl mb-2">🔍</div>
            <div className="text-sm font-medium text-gray-700 dark:text-gray-300">
              Select a region to scan…
            </div>
            <div className="text-xs text-gray-400 mt-1">Decoding QR code</div>
          </div>
        </div>
      )}

      {view.kind === "result" && (
        <ResultPanel
          content={view.content}
          onDismiss={() => setView({ kind: "idle" })}
          onConfirmAction={setConfirmAction}
        />
      )}
      {(view.kind === "no_qr" || view.kind === "error") && (
        <NoQrFound
          onRetry={() => triggerScan(view.sourceType)}
          onDismiss={() => setView({ kind: "idle" })}
        />
      )}
      {confirmAction && (
        <ConfirmDialog
          action={confirmAction}
          onClose={() => setConfirmAction(null)}
        />
      )}
      {showHistory && (
        <HistoryPanel
          onClose={() => setShowHistory(false)}
          onAction={(id, payload) => {
            setConfirmAction({ id, label: id.replace("open_", "Open "), payload, requires_confirmation: true });
          }}
        />
      )}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
      {toast && <Toast message={toast} onDismiss={() => setToast(null)} />}
    </div>
  );
}

export default App;
