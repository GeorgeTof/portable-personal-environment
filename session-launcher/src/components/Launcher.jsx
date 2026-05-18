import { convertFileSrc } from '@tauri-apps/api/core';
import React, { useState, useEffect } from 'react';
import { 
    attachTerminal,
    getInstalledApps,
    launchApp,
 } from '../api';
import terminalIcon from '../assets/terminal-1.svg';

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

  // Initialize apps
  useEffect(() => {
    const builtinApps = [
      {
        name: 'Terminal',
        exec: '__builtin_terminal__',
        icon: terminalIcon, 
        isCustom: true
      }
      // TODO add File Explorer entry
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
  }, [refreshTrigger]);

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