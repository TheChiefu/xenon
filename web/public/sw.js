self.addEventListener('push', (event) => {
  let data = {};
  if (event.data) {
    try {
      data = event.data.json();
    } catch {
      data = {};
    }
  }

  const author = data.author || 'New message';
  const title = data.room ? `${author} in ${data.room}` : author;

  const options = {
    body: data.body || '',
    tag: data.room_id || 'xenon',
    renotify: Boolean(data.renotify),
    icon: '/icon-192.png',
  };

  event.waitUntil(self.registration.showNotification(title, options));
});
