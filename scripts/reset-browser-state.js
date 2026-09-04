// =============================================================================
// Reset every trace the console left in this browser, for this origin.
//
// Paste into the DevTools console **while https://localhost is open** (F12 →
// Console) and press Enter. It reports what it removed, then reloads.
//
// WHAT IT CLEARS
//   sessionStorage   the console keeps the refresh token and session jti here
//                    (`zt.rt`, `zt.jti`) — this is the part that matters after
//                    a database wipe, because those point at a session that no
//                    longer exists
//   localStorage     nothing today, cleared anyway so a future key cannot
//                    survive a reset and be blamed for something else
//   cookies          every cookie readable from this origin
//   IndexedDB        every database
//   Cache Storage    every cache
//   service workers  unregistered
//
// WHAT IT CANNOT CLEAR — and no page script can
//   * The imported Caddy root CA. Certificates live in the browser's trust
//     store, which is deliberately out of reach of any web page. Remove it by
//     hand: Firefox → Settings → Privacy & Security → Certificates → View
//     Certificates → Authorities.
//   * The HSTS entry for localhost. Also out of reach by design — the whole
//     point of HSTS is that a page cannot talk the browser out of it. Use
//     "Forget About This Site" on localhost from the History sidebar.
//   * Passkeys. They live in the platform authenticator (Windows Hello, Touch
//     ID, a security key), not in the page. Remove them where they were
//     created; a virtual authenticator's credentials disappear with its
//     DevTools panel.
// =============================================================================

(async () => {
  const done = [];
  const failed = [];

  const attempt = async (label, work) => {
    try {
      const detail = await work();
      done.push(detail ? `${label}: ${detail}` : label);
    } catch (error) {
      failed.push(`${label}: ${error && error.message ? error.message : error}`);
    }
  };

  await attempt("sessionStorage", () => {
    const keys = Object.keys(sessionStorage);
    sessionStorage.clear();
    return keys.length ? `cleared ${keys.length} (${keys.join(", ")})` : "already empty";
  });

  await attempt("localStorage", () => {
    const keys = Object.keys(localStorage);
    localStorage.clear();
    return keys.length ? `cleared ${keys.length} (${keys.join(", ")})` : "already empty";
  });

  await attempt("cookies", () => {
    const cookies = document.cookie ? document.cookie.split(";") : [];
    for (const cookie of cookies) {
      const name = cookie.split("=")[0].trim();
      // Expire it on every path/domain combination the page can reach; a
      // cookie set on a parent path is not removed by clearing the current one.
      for (const path of ["/", location.pathname]) {
        document.cookie = `${name}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=${path}`;
        document.cookie =
          `${name}=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=${path}; domain=${location.hostname}`;
      }
    }
    return cookies.length ? `expired ${cookies.length}` : "none readable";
  });

  await attempt("IndexedDB", async () => {
    if (!indexedDB.databases) return "not enumerable in this browser, skipped";
    const databases = await indexedDB.databases();
    await Promise.all(
      databases
        .filter((database) => database.name)
        .map(
          (database) =>
            new Promise((resolve) => {
              const request = indexedDB.deleteDatabase(database.name);
              request.onsuccess = request.onerror = request.onblocked = () => resolve();
            })
        )
    );
    return databases.length ? `deleted ${databases.length}` : "none";
  });

  await attempt("Cache Storage", async () => {
    if (!window.caches) return "unavailable, skipped";
    const names = await caches.keys();
    await Promise.all(names.map((name) => caches.delete(name)));
    return names.length ? `deleted ${names.length}` : "none";
  });

  await attempt("service workers", async () => {
    if (!navigator.serviceWorker) return "unavailable, skipped";
    const registrations = await navigator.serviceWorker.getRegistrations();
    await Promise.all(registrations.map((registration) => registration.unregister()));
    return registrations.length ? `unregistered ${registrations.length}` : "none";
  });

  console.log(`%cAegis — browser state reset for ${location.origin}`, "font-weight:bold");
  done.forEach((line) => console.log("  ✓ " + line));
  failed.forEach((line) => console.warn("  ✗ " + line));

  console.log(
    "%cNot cleared (no page script can): the imported Caddy CA, the HSTS entry " +
      "for localhost, and any passkeys. See the header of this script.",
    "color:#b8860b"
  );

  console.log("Reloading in 2s…");
  setTimeout(() => location.reload(), 2000);
})();
