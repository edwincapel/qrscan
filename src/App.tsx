import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
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
  | { kind: "result"; content: ParsedContent; sourceType: string }
  | { kind: "no_qr"; sourceType: string }
  | { kind: "error"; message: string; sourceType: string };

const win = getCurrentWindow();

/** Show window at top-right of screen (below macOS menu bar). */
async function showPanel() {
  await win.setAlwaysOnTop(true);
  await win.show();
  await win.setFocus();
}

/** Hide window and remove always-on-top when no panel is active. */
async function hidePanel() {
  await win.hide();
  await win.setAlwaysOnTop(false);
}

function App() {
  const [permOk, setPermOk] = useState<boolean | null>(null);
  const [view, setView] = useState<ViewState>({ kind: "idle" });
  const [confirmAction, setConfirmAction] = useState<ActionDef | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [showHistory, setShowHistory] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    invoke<boolean>("check_screen_permission").then(setPermOk);
  }, []);

  // Show/hide window based on active panel state
  useEffect(() => {
    const hasPanel = view.kind !== "idle" || showHistory || showSettings || confirmAction !== null || permOk === false;
    if (hasPanel) {
      showPanel();
    } else {
      hidePanel();
    }
  }, [view, showHistory, showSettings, confirmAction, permOk]);

  const triggerScan = useCallback(async (mode: string) => {
    setView({ kind: "idle" });
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
      if (msg === "cancelled") return;
      if (msg === "permission_denied") { setPermOk(false); return; }
      setView({ kind: "error", message: msg, sourceType: mode });
    }
  }, []);

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    listen("scan-region", () => triggerScan("region")).then(u => unsubs.push(u));
    listen("scan-window", () => triggerScan("window")).then(u => unsubs.push(u));
    listen("show-history", () => setShowHistory(true)).then(u => unsubs.push(u));
    listen("show-settings", () => setShowSettings(true)).then(u => unsubs.push(u));
    listen<string>("shortcut-conflict", (e) => setToast(e.payload)).then(u => unsubs.push(u));
    return () => unsubs.forEach(u => u());
  }, [triggerScan]);

  if (permOk === null) return null;
  if (!permOk) return <Onboarding onGranted={() => setPermOk(true)} />;

  return (
    <>
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
          onClose={() => { setShowHistory(false); }}
          onAction={(id, payload) => {
            setConfirmAction({ id, label: id.replace("open_", "Open "), payload, requires_confirmation: true });
          }}
        />
      )}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
      {toast && <Toast message={toast} onDismiss={() => setToast(null)} />}
    </>
  );
}

export default App;
