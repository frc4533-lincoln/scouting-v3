self.addEventListener("install", (e) => {
  console.log("[Service Worker] Install");
});

const resources = [
  "/",
  "/scouting-v3.js",
  "/scouting-v3_bg.wasm",
];

self.addEventListener("install", (e) => {
  console.log("[Service Worker] Install");
  e.waitUntil(
    (async () => {
      const cache = await caches.open("scouting-v3");
      console.log("[Service Worker] Caching all: app shell and content");
      await cache.addAll(resources);
    })(),
  );
});

self.addEventListener("fetch", (e) => {
  e.respondWith(
    (async () => {
      const r = await caches.match(e.request);
      console.log(`[Service Worker] Fetching resource: ${e.request.url}`);
      if (r) {
        return r;
      } else {
        return await fetch(e.request);
      }
    })(),
  );
});

