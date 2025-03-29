import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.jsx";
import "./styles/theme.css";
import "./styles/modern.css";
import "./styles/global.css";

// Import fonts (add to your public/index.html or include via npm packages)
// For production, consider using a font loading strategy for better performance

// Initialize the application
ReactDOM.createRoot(document.getElementById("root")).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);