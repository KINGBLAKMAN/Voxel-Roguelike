// Emberwild co-op relay. The first player to connect becomes the host (it runs enemies and the world clock);
// everyone else's messages are forwarded to the host or broadcast. Run standalone with `npm run relay [port]`.
const { WebSocketServer } = require('ws');

function start(port = 7777) {
  return new Promise((resolve, reject) => {
    const wss = new WebSocketServer({ port, maxPayload: 1 << 20 });
    const clients = new Map();
    let nextId = 1, host = 0;
    const sendTo = (ws, msg) => { if (ws.readyState === 1) ws.send(msg); };

    wss.on('connection', ws => {
      const id = nextId++;
      clients.set(id, ws);
      if (!host || !clients.has(host)) host = id;
      sendTo(ws, JSON.stringify({ t: 'id', id, host }));
      const joined = JSON.stringify({ t: 'join', id });
      for (const [i, c] of clients) if (i !== id) sendTo(c, joined);

      ws.on('message', data => {
        let m;
        try { m = JSON.parse(data); } catch { return; }
        if (!m || typeof m !== 'object') return;
        m.from = id;
        const out = JSON.stringify(m);
        if (m.to) { const c = clients.get(m.to); if (c) sendTo(c, out); }
        else for (const [i, c] of clients) if (i !== id) sendTo(c, out);
      });

      ws.on('close', () => {
        clients.delete(id);
        const msg = JSON.stringify(id === host ? { t: 'hostleft' } : { t: 'leave', id });
        for (const c of clients.values()) sendTo(c, msg);
        if (id === host) host = 0;
      });
    });

    wss.once('listening', () => resolve({ port, close: () => { for (const c of clients.values()) c.close(); wss.close(); } }));
    wss.once('error', reject);
  });
}

module.exports = { start };

if (require.main === module) {
  const port = Number(process.argv[2]) || 7777;
  start(port).then(() => console.log(`Emberwild relay listening on port ${port}`), e => { console.error(e.message); process.exit(1); });
}
