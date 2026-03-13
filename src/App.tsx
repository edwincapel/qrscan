import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import Onboarding from "./components/Onboarding";
import ResultPanel from "./components/ResultPanel";
import ConfirmDialog from "./components/ConfirmDialog";
import NoQrFound from "./components/NoQrFound";
import HistoryPanel from "./components/HistoryPanel";
import Toast from "./components/Toast";
import { decodeQR } from "./lib/decoder";

interface ScanResult { image_path: string; source_type: string }
interface ActionDef { id: string; label: string; payload: string; requires_confirmation: boolean; confirmation_message?: string }
interface ParsedContent { content_type: string; raw: string; display_text: string; actions: ActionDef[]; fields?: Record<string, string>; warnings?: string[] }

type ViewState =
  | { kind: "idle" }
  | { kind: "result"; content: ParsedContent; sourceType: string }
  | { kind: "no_qr"; sourceType: string }
  | { kind: "error"; message: string; sourceType: string };

function App() {
  const [permOk, setPermOk] = useState<boolean | null>(null);
  const [view, setView] = useState<ViewState>({ kind: "idle" });
  const [confirmAction, setConfirmAction] = useState<ActionDef | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [showHistory, setShowHistory] = useState(false);

  useEffect(() => {
    invoke<boolean>("check_screen_permission").then(setPermOk);
  }, []);

  const triggerScan = useCallback(async (mode: string) => {
    setView({ kind: "idle" });
    setConfirmAction(null);
    try {
      const scan = await invoke<ScanResult>("trigger_scan", { mode });
      const decoded = await decodeQR(scan.image_path);
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
      {view.kind === "no_qr" && (
        <NoQrFound
          onRetry={() => triggerScan(view.sourceType)}
          onDismiss={() => setView({ kind: "idle" })}
        />
      )}
      {view.kind === "error" && (
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
      {toast && <Toast message={toast} onDismiss={() => setToast(null)} />}
    </>
  );
}

export default App;
