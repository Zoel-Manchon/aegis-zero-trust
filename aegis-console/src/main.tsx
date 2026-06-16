import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App";
import { AuthProvider } from "@/lib/auth/AuthContext";

const root = document.getElementById("root");
if (!root) throw new Error("missing #root mount point in index.html");

createRoot(root).render(
    <StrictMode>
        <AuthProvider>
            <App />
        </AuthProvider>
    </StrictMode>,
);