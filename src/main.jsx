import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.jsx";
// Wichtig: Hier Tauri API importieren, wenn es irgendwo in der App verwendet wird
import { invoke } from "@tauri-apps/api/core";

// Import CSS in the correct order
// 1. Variables first (design tokens)
import "./styles/variables.css";
// 2. Base/reset styles
import "./styles/global.css";
// 3. Theme styles for light/dark mode
import "./styles/theme.css";
// 4. Component styles
import "./styles/components.css";
// 5. Modern enhancements and effects
import "./styles/modern.css";

// Import font (Inter)
// For production, consider using a font loading strategy or self-hosting the font
const fontLink = document.createElement('link');
fontLink.rel = 'stylesheet';
fontLink.href = 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap';
document.head.appendChild(fontLink);

// Stelle sicher, dass window.invoke verfügbar ist, falls du es global verwendest
window.__TAURI_INVOKE__ = invoke;

// Initialize the application
ReactDOM.createRoot(document.getElementById("root")).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);