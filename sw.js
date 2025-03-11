var cacheName = 'epe';
var filesToCache = [
  './',
  './index.html',
'./easy_protobuf_edit-2e67823a9957e6ec.js',
'./easy_protobuf_edit-2e67823a9957e6ec_bg.wasm',
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
