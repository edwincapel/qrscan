import { useEffect } from "react";

interface Props {
  message: string;
  onDismiss: () => void;
  durationMs?: number;
}

function Toast({ message, onDismiss, durationMs = 4000 }: Props) {
  useEffect(() => {
    const timer = setTimeout(onDismiss, durationMs);
    return () => clearTimeout(timer);
  }, [onDismiss, durationMs]);

  return (
    <div className="fixed bottom-4 right-4 bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900 text-xs px-4 py-2 rounded-lg shadow-lg">
      {message}
    </div>
  );
}

export default Toast;
