import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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

// Window management via Rust commands — more reliable than JS window APIs
async function showPanel() {
  try { await invoke("show_panel_window"); }
  catch (e) { console.error("[QRSnap] showPanel failed:", e); }
}
async function hidePanel() {
  try { await invoke("hide_panel_window"); }
  catch (e) { console.error("[QRSnap] hidePanel failed:", e); }
}

function App() {
  const [permOk, setPermOk] = useState<boolean | null>(null);
  const [view, setView] = useState<ViewState>({ kind: "idle" });
  const [confirmAction, setConfirmAction] = useState<ActionDef | null>(null);
  const [toast, setToast] = useState<string | null>(null);
  const [showHistory, setShowHistory] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  const showToast = useCallback((msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 4000);
  }, []);

  // Permission check — 3s timeout, never gets stuck
  useEffect(() => {
    const timer = setTimeout(() => { console.warn("[QRSnap] perm check timeout"); setPermOk(true); }, 3000);
    invoke<boolean>("check_screen_permission")
      .then(ok => { clearTimeout(timer); console.log("[QRSnap] perm:", ok); setPermOk(ok); })
      .catch(e => { clearTimeout(timer); console.error("[QRSnap] perm error:", e); setPermOk(true); });
    return () => clearTimeout(timer);
  }, []);

  // Show/hide window based on active panel state
  useEffect(() => {
    const hasPanel = view.kind !== "idle" || showHistory || showSettings || confirmAction !== null || permOk === false;
    console.log("[QRSnap] hasPanel:", hasPanel, "view:", view.kind, "history:", showHistory);
    if (hasPanel) showPanel(); else hidePanel();
  }, [view, showHistory, showSettings, confirmAction, permOk]);

  const triggerScan = useCallback(async (mode: string) => {
    console.log("[QRSnap] triggerScan:", mode);
    setView({ kind: "scanning", sourceType: mode });
    setConfirmAction(null);
    try {
      const scan = await invoke<ScanResult>("trigger_scan", { mode });
      console.log("[QRSnap] capture ok, image_data length:", scan.image_data.length);
      const decoded = await decodeQR(scan.image_data);
      console.log("[QRSnap] decode result:", decoded?.text ?? "null");
      if (!decoded) {
        setView({ kind: "no_qr", sourceType: mode });
        return;
      }
      const parsed = await invoke<ParsedContent>("parse_qr_content", { raw: decoded.text });
      console.log("[QRSnap] parsed type:", parsed.content_type);
      setView({ kind: "result", content: parsed, sourceType: mode });
      // Save to history
      const entry = {
        id: crypto.randomUUID(),
        scanned_at: new Date().toISOString(),
        result: decoded.text,
        result_type: parsed.content_type,
        parsed_data: parsed.fields ?? null,
        source_type: mode,
        source_name: mode === "region" ? "Screen Region" : "Window",
        thumbnail_file: null,
      };
      invoke("save_scan", { entry }).catch(e => console.error("[QRSnap] save_scan:", e));
    } catch (e) {
      const msg = String(e);
      console.error("[QRSnap] scan error:", msg);
      if (msg === "cancelled") { setView({ kind: "idle" }); return; }
      if (msg === "permission_denied") { setPermOk(false); setView({ kind: "idle" }); return; }
      setView({ kind: "error", message: msg, sourceType: mode });
    }
  }, []);

  useEffect(() => {
    console.log("[QRSnap] setting up event listeners");
    const unsubs: Array<() => void> = [];
    listen("scan-region", () => { console.log("[QRSnap] got scan-region"); triggerScan("region"); }).then(u => unsubs.push(u));
    listen("scan-window", () => { console.log("[QRSnap] got scan-window"); triggerScan("window"); }).then(u => unsubs.push(u));
    listen("show-history", () => { console.log("[QRSnap] got show-history"); setShowHistory(true); }).then(u => unsubs.push(u));
    listen("show-settings", () => { console.log("[QRSnap] got show-settings"); setShowSettings(true); }).then(u => unsubs.push(u));
    listen<string>("shortcut-conflict", e => showToast(e.payload)).then(u => unsubs.push(u));
    return () => { console.log("[QRSnap] cleanup listeners"); unsubs.forEach(u => u()); };
  }, [triggerScan, showToast]);

  if (permOk === null) {
    return <div className="flex items-center justify-center h-screen bg-white"><div className="text-sm text-gray-400">Starting…</div></div>;
  }
  if (!permOk) return <Onboarding onGranted={() => setPermOk(true)} />;

  return (
    <div className="bg-white dark:bg-gray-900 min-h-screen">
      {view.kind === "scanning" && (
        <div className="fixed inset-0 flex items-center justify-center bg-white dark:bg-gray-900">
          <div className="rounded-xl shadow-lg border border-gray-200 p-6 text-center">
            <div className="text-2xl mb-2">🔍</div>
            <div className="text-sm font-medium text-gray-700">Select a region…</div>
            <div className="text-xs text-gray-400 mt-1">Decoding QR code</div>
          </div>
        </div>
      )}
      {view.kind === "result" && (
        <ResultPanel content={view.content} onDismiss={() => setView({ kind: "idle" })} onConfirmAction={setConfirmAction} />
      )}
      {(view.kind === "no_qr" || view.kind === "error") && (
        <NoQrFound onRetry={() => triggerScan(view.sourceType)} onDismiss={() => setView({ kind: "idle" })} />
      )}
      {confirmAction && <ConfirmDialog action={confirmAction} onClose={() => setConfirmAction(null)} />}
      {showHistory && (
        <HistoryPanel onClose={() => setShowHistory(false)} onAction={(id, payload) =>
          setConfirmAction({ id, label: id.replace("open_", "Open "), payload, requires_confirmation: true })} />
      )}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
      {toast && <Toast message={toast} onDismiss={() => setToast(null)} />}
    </div>
  );
}

export default App;
