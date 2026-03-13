import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import HistoryEntry from "./HistoryEntry";

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

interface Props {
  onClose: () => void;
  onAction: (actionId: string, payload: string) => void;
}

function HistoryPanel({ onClose, onAction }: Props) {
  const [entries, setEntries] = useState<Entry[]>([]);
  const [search, setSearch] = useState("");

  const loadHistory = useCallback(async () => {
    try {
      const data = await invoke<Entry[]>("get_history");
      setEntries(data);
    } catch (e) {
      console.error("Load history:", e);
    }
  }, []);

  useEffect(() => { loadHistory(); }, [loadHistory]);

  const handleDelete = async (id: string) => {
    await invoke("delete_history", { id });
    loadHistory();
  };

  const handleClearAll = async () => {
    await invoke("clear_history");
    setEntries([]);
  };

  const filtered = search
    ? entries.filter(e =>
        e.result.toLowerCase().includes(search.toLowerCase()) ||
        e.result_type.toLowerCase().includes(search.toLowerCase())
      )
    : entries;

  return (
    <div className="fixed inset-0 bg-white dark:bg-gray-900 z-40 flex flex-col">
      <div className="flex items-center justify-between p-3 border-b border-gray-200 dark:border-gray-700">
        <h2 className="text-sm font-semibold text-gray-900 dark:text-white">Scan History</h2>
        <div className="flex gap-2">
          {entries.length > 0 && (
            <button onClick={handleClearAll} className="text-[10px] px-2 py-1 text-red-600 dark:text-red-400">
              Clear All
            </button>
          )}
          <button onClick={onClose} className="text-gray-400 hover:text-gray-600 text-sm">
            Close
          </button>
        </div>
      </div>
      <div className="p-2">
        <input
          type="text"
          placeholder="Search results..."
          value={search}
          onChange={e => setSearch(e.target.value)}
          className="w-full text-xs px-3 py-2 rounded-lg bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-200 outline-none"
        />
      </div>
      <div className="flex-1 overflow-y-auto">
        {filtered.length === 0 ? (
          <p className="text-xs text-gray-400 text-center py-8">
            {search ? "No matching results" : "No scans yet"}
          </p>
        ) : (
          filtered.map(entry => (
            <HistoryEntry
              key={entry.id}
              entry={entry}
              onDelete={handleDelete}
              onAction={onAction}
            />
          ))
        )}
      </div>
    </div>
  );
}

export default HistoryPanel;
