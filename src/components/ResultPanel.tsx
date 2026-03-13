import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

interface ActionDef {
  id: string;
  label: string;
  payload: string;
  requires_confirmation: boolean;
  confirmation_message?: string;
}

interface ParsedContent {
  content_type: string;
  raw: string;
  display_text: string;
  actions: ActionDef[];
  fields?: Record<string, string>;
  warnings?: string[];
}

interface Props {
  content: ParsedContent;
  autoDismissMs?: number;
  onDismiss: () => void;
  onConfirmAction: (action: ActionDef) => void;
}

const TYPE_BADGES: Record<string, string> = {
  url: "URL",
  wifi: "WiFi",
  vcard: "Contact",
  event: "Event",
  email: "Email",
  phone: "Phone",
  sms: "SMS",
  geo: "Location",
  text: "Text",
  error: "Error",
};

function ResultPanel({ content, autoDismissMs = 15000, onDismiss, onConfirmAction }: Props) {
  const [showPassword, setShowPassword] = useState(false);

  useEffect(() => {
    if (autoDismissMs <= 0) return;
    const timer = setTimeout(onDismiss, autoDismissMs);
    return () => clearTimeout(timer);
  }, [autoDismissMs, onDismiss]);

  const handleAction = (action: ActionDef) => {
    if (action.requires_confirmation) {
      onConfirmAction(action);
    } else {
      invoke(action.id, { text: action.payload });
    }
  };

  const badge = TYPE_BADGES[content.content_type] ?? "Unknown";
  const isWifi = content.content_type === "wifi";

  return (
    <div className="animate-slide-in fixed top-4 right-4 w-80 bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-4">
      <div className="flex items-center justify-between mb-2">
        <span className="text-xs font-medium px-2 py-0.5 rounded-full bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300">
          {badge}
        </span>
        <button onClick={onDismiss} className="text-gray-400 hover:text-gray-600 text-sm">
          Dismiss
        </button>
      </div>

      {isWifi && content.fields ? (
        <div className="text-sm text-gray-800 dark:text-gray-200 space-y-1 mb-3">
          <div>SSID: {content.fields.ssid}</div>
          <div>Security: {content.fields.auth}</div>
          <div className="flex items-center gap-2">
            Password: {showPassword ? content.actions.find(a => a.label === "Copy Password")?.payload ?? "—" : "•••••••••"}
            <button
              onClick={() => setShowPassword(!showPassword)}
              className="text-xs text-blue-600 dark:text-blue-400"
            >
              {showPassword ? "Hide" : "Show"}
            </button>
          </div>
        </div>
      ) : (
        <p className="text-sm text-gray-800 dark:text-gray-200 break-all mb-3">
          {content.display_text}
        </p>
      )}

      {content.warnings?.map((w, i) => (
        <p key={i} className="text-xs text-amber-600 dark:text-amber-400 mb-2">
          {w}
        </p>
      ))}

      <div className="flex flex-wrap gap-2">
        {content.actions.map((action) => (
          <button
            key={action.id + action.label}
            onClick={() => handleAction(action)}
            className="text-xs px-3 py-1.5 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600"
          >
            {action.label}
          </button>
        ))}
      </div>
    </div>
  );
}

export default ResultPanel;
