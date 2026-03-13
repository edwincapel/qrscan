import { invoke } from "@tauri-apps/api/core";

interface ActionDef {
  id: string;
  label: string;
  payload: string;
  requires_confirmation: boolean;
  confirmation_message?: string;
}

interface Props {
  action: ActionDef;
  onClose: () => void;
}

function ConfirmDialog({ action, onClose }: Props) {
  const handleConfirm = async () => {
    try {
      // Each action.id maps to a dedicated Rust command
      // Parse payload based on the command's expected args
      if (action.id === "open_url") {
        await invoke("open_url", { urlStr: action.payload });
      } else if (action.id === "open_mailto") {
        await invoke("open_mailto", { address: action.payload, subject: null });
      } else if (action.id === "open_tel") {
        await invoke("open_tel", { number: action.payload });
      } else if (action.id === "open_sms") {
        await invoke("open_sms", { number: action.payload, body: null });
      } else if (action.id === "open_geo") {
        const [lat, lon] = action.payload.split(",").map(Number);
        await invoke("open_geo", { lat, lon });
      } else if (action.id === "open_calendar_event") {
        await invoke("open_calendar_event", { eventRaw: action.payload });
      }
    } catch (e) {
      console.error("Action failed:", e);
    }
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl p-6 w-80">
        <h2 className="text-base font-semibold text-gray-900 dark:text-white mb-3">
          {action.label}?
        </h2>
        <p className="text-sm text-gray-600 dark:text-gray-400 break-all mb-2">
          {action.payload}
        </p>
        {action.confirmation_message && (
          <p className="text-xs text-amber-600 dark:text-amber-400 mb-4">
            Links from QR codes may lead to untrusted websites.
          </p>
        )}
        <div className="flex justify-end gap-3">
          <button
            onClick={onClose}
            className="text-sm px-4 py-2 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
          >
            Cancel
          </button>
          <button
            onClick={handleConfirm}
            className="text-sm px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700"
          >
            {action.label}
          </button>
        </div>
      </div>
    </div>
  );
}

export default ConfirmDialog;
