const { app, BrowserWindow, ipcMain, Menu } = require('electron');
const path = require('path');
const os = require('os');
const relay = require('./relay');

// Run like a native game: no GPU blocklist surprises, no throttling when alt-tabbed, raw mouse for pointer lock.
app.commandLine.appendSwitch('ignore-gpu-blocklist');
app.commandLine.appendSwitch('disable-renderer-backgrounding');

let win = null;
let server = null;

function lanAddresses() {
  const out = [];
  for (const list of Object.values(os.networkInterfaces())) {
    for (const a of list || []) if (a.family === 'IPv4' && !a.internal) out.push(a.address);
  }
  return out;
}

function createWindow() {
  win = new BrowserWindow({
    width: 1440, height: 900, minWidth: 960, minHeight: 600, fullscreen: true, show: false,
    backgroundColor: '#0c0b0f', title: 'Emberwild', autoHideMenuBar: true,
    webPreferences: { preload: path.join(__dirname, 'preload.js'), contextIsolation: true, nodeIntegration: false, backgroundThrottling: false },
  });
  Menu.setApplicationMenu(null);
  win.once('ready-to-show', () => win.show());
  win.loadFile(path.join(__dirname, '..', 'index.html'));
  win.webContents.on('before-input-event', (e, input) => {
    if (input.type === 'keyDown' && input.key === 'F11') { win.setFullScreen(!win.isFullScreen()); e.preventDefault(); }
  });
}

ipcMain.handle('host', async (_e, port) => {
  if (!server) server = await relay.start(port || 7777);
  return { port: server.port, ips: lanAddresses() };
});
ipcMain.handle('quit', () => app.quit());
ipcMain.handle('fullscreen', (_e, on) => { if (win) win.setFullScreen(on == null ? !win.isFullScreen() : !!on); return win ? win.isFullScreen() : false; });
ipcMain.handle('stopHost', () => { if (server) { server.close(); server = null; } });

app.whenReady().then(createWindow);
app.on('window-all-closed', () => { if (server) server.close(); app.quit(); });
