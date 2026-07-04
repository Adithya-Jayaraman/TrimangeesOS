const { app, BrowserWindow, session, ipcMain, shell } = require('electron');
const path = require('path');

let mainWindow;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1300,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    frame: false,               // Use the custom HTML title bar
    titleBarStyle: 'hidden',
    autoHideMenuBar: true,
    webPreferences: {
      webviewTag: true,         // CRITICAL: enables <webview> so Google/YouTube work
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'),
    },
    backgroundColor: '#0a0a0f',
    show: false,
  });

  // Avoid white flash on load
  mainWindow.once('ready-to-show', () => mainWindow.show());
  mainWindow.loadFile('browser.html');

  // Spoof User-Agent on every request so Google/YouTube serve the full site
  session.defaultSession.webRequest.onBeforeSendHeaders((details, callback) => {
    details.requestHeaders['User-Agent'] =
      'Mozilla/5.0 (Windows NT 10.0; Win64; x64) ' +
      'AppleWebKit/537.36 (KHTML, like Gecko) ' +
      'Chrome/124.0.0.0 Safari/537.36';
    callback({ requestHeaders: details.requestHeaders });
  });

  // Grant all permissions (mic, camera, geolocation, fullscreen etc.)
  session.defaultSession.setPermissionRequestHandler((_wc, permission, callback) => {
    callback(true);
  });
  session.defaultSession.setPermissionCheckHandler(() => true);

  // IPC: HTML title bar dots -> window controls
  ipcMain.on('win-minimize', () => mainWindow.minimize());
  ipcMain.on('win-maximize', () =>
    mainWindow.isMaximized() ? mainWindow.unmaximize() : mainWindow.maximize()
  );
  ipcMain.on('win-close', () => mainWindow.close());

  // When a webview opens a new window, send it to a new browser tab instead
  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    if (url.startsWith('mailto:') || url.startsWith('tel:')) {
      shell.openExternal(url);
      return { action: 'deny' };
    }
    mainWindow.webContents.executeJavaScript(
      `typeof newTab === 'function' && newTab(${JSON.stringify(url)})`
    );
    return { action: 'deny' };
  });
}

// Accept any certificate (useful for intranet/dev sites)
app.on('certificate-error', (event, _wc, _url, _err, _cert, callback) => {
  event.preventDefault();
  callback(true);
});

app.whenReady().then(() => {
  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});