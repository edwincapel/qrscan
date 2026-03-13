interface Props {
  onRetry: () => void;
  onDismiss: () => void;
}

function NoQrFound({ onRetry, onDismiss }: Props) {
  return (
    <div className="fixed top-4 right-4 w-72 bg-white dark:bg-gray-800 rounded-xl shadow-lg border border-gray-200 dark:border-gray-700 p-4">
      <h2 className="text-sm font-semibold text-gray-900 dark:text-white mb-1">
        No QR code found
      </h2>
      <p className="text-xs text-gray-500 dark:text-gray-400 mb-4">
        No QR code was detected in the selected area.
      </p>
      <div className="flex justify-end gap-2">
        <button
          onClick={onDismiss}
          className="text-xs px-3 py-1.5 rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
        >
          Dismiss
        </button>
        <button
          onClick={onRetry}
          className="text-xs px-3 py-1.5 rounded-lg bg-blue-600 text-white hover:bg-blue-700"
        >
          Retry
        </button>
      </div>
    </div>
  );
}

export default NoQrFound;
