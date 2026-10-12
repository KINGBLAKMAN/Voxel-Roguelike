const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('native', {
  host: port => ipcRenderer.invoke('host', port),
  stopHost: () => ipcRenderer.invoke('stopHost'),
});
