const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
  getAppVersion: () => process.versions.electron,
  isElectron:    true,
  platform:      process.platform,
  minimize:      () => ipcRenderer.send('win-minimize'),
  maximize:      () => ipcRenderer.send('win-maximize'),
  close:         () => ipcRenderer.send('win-close'),
});

console.log('Preload loaded — Electron', process.versions.electron);