import { AppRouter } from "@/app/router";

/**
 * Top-level App — just mounts the router. All real layout lives in the
 * individual pages (LoginPage, Dashboard, etc.) so each can have its own
 * frame without fighting a global shell.
 */
export default function App() {
    return <AppRouter />;
}