import React from 'react';

export const LoadingView = () => (
  <div className="view-container">
    <h2>Initializing...</h2>
    <div className="spinner"></div>
  </div>
);

export const StartingUpView = () => (
  <div className="view-container">
    <h2>Starting up the environment...</h2>
    <div className="spinner"></div>
  </div>
);

export const HostErrorView = () => (
  <div className="view-container">
    <h2>Host Configuration Error</h2>
    <p>The host is not properly configured! Please run the configuration script as sudo and try again.</p>
  </div>
);

export const EnvironmentActionView = ({ title, description, buttonText = "Create a new environment", onAction }) => (
  <div className="view-container">
    <h2>{title}</h2>
    <p>{description}</p>
    <button className="btn primary" onClick={onAction}>
      {buttonText}
    </button>
  </div>
);

export const StartView = ({ onStart, errorMessage }) => (
  <div className="view-container">
    {errorMessage && <div className="error-message">{errorMessage}</div>}
    <button className="btn start-btn" onClick={onStart}>
      START
    </button>
  </div>
);

export const SetupUserView = () => (
  <div className="view-container">
    <h2>Initial Setup Required</h2>
    <p>(TODO Configuration...)</p>
  </div>
);

export const LauncherView = () => (
  <div className="view-container">
    <h2>Launcher</h2>
  </div>
);