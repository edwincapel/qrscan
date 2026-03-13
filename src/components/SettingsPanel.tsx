import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

interface Settings {
  autoCopy: boolean;
  autoDismissSec: number;
  storeWifiPasswords: boolean;
  launchAtLogin: boolean;
  theme: "system" | "light" | "dark";
}

const DEFAULTS: Settings = {
  autoCopy: true,
  autoDismissSec: 15,
  storeWifiPasswords: false,
  launchAtLogin: false,
  theme: "system",
};

interface Props {
  onClose: () => void;
}

function SettingsPanel({ onClose }: Props) {
  const [settings, setSettings] = useState<Settings>(DEFAULTS);

  useEffect(() => {
    invoke<string>("get_settings").then(json => {
      try { setSettings({ ...DEFAULTS, ...JSON.parse(json) }); } catch { /* use defaults */ }
    }).catch(() => {});
  }, []);

  const update = (patch: Partial<Settings>) => {
    const next = { ...settings, ...patch };
    setSettings(next);
    invoke("save_settings", { json: JSON.stringify(next) });
  };

  return (
    <div className="fixed inset-0 bg-white dark:bg-gray-900 z-40 flex flex-col">
      <div className="flex items-center justify-between p-3 border-b border-gray-200 dark:border-gray-700">
        <h2 className="text-sm font-semibold text-gray-900 dark:text-white">Settings</h2>
        <button onClick={onClose} className="text-gray-400 hover:text-gray-600 text-sm">Close</button>
      </div>
      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        <Toggle label="Auto-copy to clipboard" value={settings.autoCopy} onChange={v => update({ autoCopy: v })} />
        <div>
          <label className="text-xs text-gray-600 dark:text-gray-400">Auto-dismiss timeout</label>
          <select
            value={settings.autoDismissSec}
            onChange={e => update({ autoDismissSec: Number(e.target.value) })}
            className="mt-1 block w-full text-xs px-3 py-2 rounded-lg bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-200"
          >
            <option value={5}>5 seconds</option>
            <option value={10}>10 seconds</option>
            <option value={15}>15 seconds</option>
            <option value={30}>30 seconds</option>
            <option value={0}>Never</option>
          </select>
        </div>
        <Toggle label="Store WiFi passwords (Keychain)" value={settings.storeWifiPasswords} onChange={v => update({ storeWifiPasswords: v })} />
        <Toggle label="Launch at login" value={settings.launchAtLogin} onChange={v => update({ launchAtLogin: v })} />
        <div>
          <label className="text-xs text-gray-600 dark:text-gray-400">Theme</label>
          <select
            value={settings.theme}
            onChange={e => update({ theme: e.target.value as Settings["theme"] })}
            className="mt-1 block w-full text-xs px-3 py-2 rounded-lg bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-200"
          >
            <option value="system">System</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </div>
        <p className="text-[10px] text-gray-400 pt-4">
          URL confirmation dialogs are always shown and cannot be disabled.
        </p>
      </div>
    </div>
  );
}

function Toggle({ label, value, onChange }: { label: string; value: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="flex items-center justify-between cursor-pointer">
      <span className="text-xs text-gray-600 dark:text-gray-400">{label}</span>
      <div
        onClick={() => onChange(!value)}
        className={`w-9 h-5 rounded-full transition-colors ${value ? "bg-blue-600" : "bg-gray-300 dark:bg-gray-600"} relative`}
      >
        <div className={`w-4 h-4 bg-white rounded-full absolute top-0.5 transition-transform ${value ? "translate-x-4" : "translate-x-0.5"}`} />
      </div>
    </label>
  );
}

export default SettingsPanel;
