import { invoke } from "@tauri-apps/api/core";

interface Entry {
  id: string;
  scanned_at: string;
  result: string;
  result_type: string;
  parsed_data?: Record<string, string>;
  source_type: string;
  source_name?: string;
  thumbnail_file?: string;
}

const BADGES: Record<string, string> = {
  url: "URL", wifi: "WiFi", vcard: "Contact", event: "Event",
  email: "Email", phone: "Phone", sms: "SMS", geo: "Location", text: "Text",
};

interface Props {
  entry: Entry;
  onDelete: (id: string) => void;
  onAction: (actionId: string, payload: string) => void;
}

function HistoryEntry({ entry, onDelete, onAction }: Props) {
  const badge = BADGES[entry.result_type] ?? "Text";
  const display = entry.result_type === "wifi"
    ? entry.parsed_data?.ssid ?? entry.result
    : entry.result;
  const time = new Date(entry.scanned_at).toLocaleString();

  const handleCopy = () => {
    invoke("copy_to_clipboard", { text: entry.result });
  };

  return (
    <div className="p-3 border-b border-gray-100 dark:border-gray-700">
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300">
              {badge}
            </span>
            <span className="text-[10px] text-gray-400">{entry.source_type}</span>
          </div>
          <p className="text-xs text-gray-800 dark:text-gray-200 break-all line-clamp-2">
            {display}
          </p>
          <p className="text-[10px] text-gray-400 mt-1">{time}</p>
        </div>
      </div>
      <div className="flex gap-1.5 mt-2">
        {entry.result_type === "url" && (
          <button
            onClick={() => onAction("open_url", entry.result)}
            className="text-[10px] px-2 py-1 rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300"
          >
            Open URL
          </button>
        )}
        <button onClick={handleCopy} className="text-[10px] px-2 py-1 rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-300">
          Copy
        </button>
        <button
          onClick={() => onDelete(entry.id)}
          className="text-[10px] px-2 py-1 rounded bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400"
        >
          Delete
        </button>
      </div>
    </div>
  );
}

export default HistoryEntry;
