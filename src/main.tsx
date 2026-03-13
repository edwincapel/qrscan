import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";

// No StrictMode — double-fires effects in dev, breaks Tauri event listeners
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<App />);
