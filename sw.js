var cacheName = 'epe';
var filesToCache = [
  './',
  './index.html',
'./easy_protobuf_edit-709b4ee15f3742b8.js',
'./easy_protobuf_edit-709b4ee15f3742b8_bg.wasm',
];

/* Start the service worker and cache all of the app's content */
self.addEventListener('install', function (e) {
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    })
  );
});

/* Serve cached content when offline */
self.addEventListener('fetch', function (e) {
  e.respondWith(
    caches.match(e.request).then(function (response) {
      return response || fetch(e.request);
    })
  );
});
