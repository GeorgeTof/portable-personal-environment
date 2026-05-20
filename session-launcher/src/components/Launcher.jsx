import { convertFileSrc } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import React, { useState, useEffect } from 'react';
import { 
    attachTerminal,
    openExplorer,
    getInstalledApps,
    launchApp,
    startAppWatcher,
 } from '../api';
import terminalIcon from '../assets/terminal-1.svg';
import filesIcon from '../assets/files-1.png';

// Individual Icon Component
const AppIcon = ({ app, onClick, onContextMenu }) => {
  return (
    <div 
      className="app-icon-container" 
      onClick={() => onClick(app)}
      onContextMenu={(e) => onContextMenu(e, app)}
    >
      <div className="app-icon-image">
        <img src={app.icon} alt={app.name} className={app.isCustom ? "builtin-icon" : ""} />
      </div>
      <span className="app-icon-name">{app.name}</span>
    </div>
  );
};

// Main Launcher View
export const LauncherView = ({ refreshTrigger }) => {
  const [apps, setApps] = useState([]);
  const [contextMenu, setContextMenu] = useState(null);
  const [internalTrigger, setInternalTrigger] = useState(0);

  // Initialize apps
  useEffect(() => {
    const builtinApps = [
      {
        name: 'Terminal',
        exec: '__builtin_terminal__',
        icon: terminalIcon, 
        isCustom: true
      },
      {
        name: 'Files',
        exec: '__builtin_explorer__',
        icon: filesIcon,
        isCustom: true
      }
    ];

    const loadApps = async () => {
      const desktopApps = await getInstalledApps();
      
      // convert absolute file paths to Tauri-safe URLs
      const safeApps = desktopApps.map(app => {
        if (!app.isCustom && app.icon) {
          app.icon = convertFileSrc(app.icon);
        }
        return app;
      });

      const combinedApps = [...builtinApps, ...safeApps];
      combinedApps.sort((a, b) => 
        a.name.toLowerCase().localeCompare(b.name.toLowerCase())
      );

      setApps(combinedApps);
    };

    loadApps();
  }, [refreshTrigger, internalTrigger]);

  useEffect(() => {
    let timeoutId;

    const initWatcher = async () => {
      await startAppWatcher();
    };

    // event emitted by Rust
    const unlistenPromise = listen('apps-changed', () => {
      
      // debouncer - packet managers make multiple fast updates instead of only one write
      clearTimeout(timeoutId);
      timeoutId = setTimeout(() => {
        console.log("Filesystem quiet, refreshing apps...");
        setInternalTrigger(prev => prev + 1);
      }, 1000); 
      
    });

    initWatcher();

    return () => {
      clearTimeout(timeoutId);
      unlistenPromise.then(unlisten => unlisten());
    };
  }, []);

  // Close context menu on any global click
  useEffect(() => {
    const handleGlobalClick = () => setContextMenu(null);
    window.addEventListener('click', handleGlobalClick);
    return () => window.removeEventListener('click', handleGlobalClick);
  }, []);

  // Handle standard left click
  const handleAppClick = async (app) => {
    if (app.isCustom) {
      if (app.exec === '__builtin_terminal__') {
        const success = await attachTerminal();
        if (!success) {
            console.error("Failed to attach terminal.");
        }
      }
      else if (app.exec === '__builtin_explorer__') {
        const success = await openExplorer();
        if (!success) console.error("Failed to open file explorer.");
      }
    } else {
      const success = await launchApp(app.exec, false);
      if (!success) console.error(`Failed to launch ${app.name}`);
    }
  };

  // Handle right click
  const handleContextMenu = (e, app) => {
    e.preventDefault(); 
    if (app.isCustom) return;

    setContextMenu({
      x: e.pageX,
      y: e.pageY,
      app
    });
  };

  const handleLaunchWithTerminal = async () => {
    if (!contextMenu?.app) return;
    const success = await launchApp(contextMenu.app.exec, true);
    if (!success) console.error(`Failed to launch ${contextMenu.app.name} with terminal`);
    setContextMenu(null);
  };

  return (
    <div className="view-container launcher-view">
      <h2>Applications</h2>
      
      <div className="app-grid">
        {apps.map((app, idx) => (
          <AppIcon 
            key={idx} 
            app={app} 
            onClick={handleAppClick} 
            onContextMenu={handleContextMenu} 
          />
        ))}
      </div>

      {/* Dynamic Context Menu */}
      {contextMenu && (
        <div 
          className="context-menu" 
          style={{ top: contextMenu.y, left: contextMenu.x }}
        >
          <div className="context-menu-item" onClick={handleLaunchWithTerminal}>
            Launch with terminal attached
          </div>
          <div className="context-menu-item" onClick={() => {
            console.log("Executable path:", contextMenu.app.exec);
            setContextMenu(null);
          }}>
            Copy executable path
          </div>
        </div>
      )}
    </div>
  );
};