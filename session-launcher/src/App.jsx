import React, { useState, useEffect } from 'react';
import { 
  checkHostConfig, 
  checkDriveConfiguration, 
  checkContainerIntegrity,
  createEnvironment
} from './api';
import { 
  LoadingView, 
  HostErrorView, 
  EnvironmentActionView,
  StartView 
} from './components/Views';
import { Footer } from './components/Footer';
import './App.css';

function App() {
  const [viewState, setViewState] = useState('loading'); // loading, hostError, missingEnv, corruptEnv, ready, creationFailed
  const [footerState, setFooterState] = useState('none'); // none, unmount, stop

  useEffect(() => {
    runStartupChecks();
  }, []);

  const runStartupChecks = async () => {
    setViewState('loading');
    setFooterState('none');

    const hostOk = await checkHostConfig();
    if (!hostOk) {
      setViewState('hostError');
      setFooterState('unmount');
      return;
    }

    const driveOk = await checkDriveConfiguration();
    if (!driveOk) {
      setViewState('missingEnv');
      setFooterState('unmount'); 
      return;
    }

    const containerOk = await checkContainerIntegrity();
    if (!containerOk) {
      setViewState('corruptEnv');
      setFooterState('unmount');
      return;
    }

    setViewState('ready');
    setFooterState('unmount');
  };

  const handleCreateEnvironment = async () => {
    setViewState('loading');
    const success = await createEnvironment();
    if (success) {
      runStartupChecks(); 
    }
    else {
      setViewState('creationFailed');
      setFooterState('unmount');
    }
  };

  const handleStart = () => {
    console.log("Start button pressed! (To be implemented)");
    // Todo: transition to app grid
  };

  const handleUnmount = () => {
    console.log("Unmounting drive... (Trigger workaround script and close app)");
  };

  const renderMainView = () => {
    switch (viewState) {
      case 'loading': 
        return <LoadingView />;
      case 'hostError': 
        return <HostErrorView />;
      case 'missingEnv': 
        return (
          <EnvironmentActionView 
            title="Environment Not Found"
            description="You don't have a personal environment."
            onAction={handleCreateEnvironment}
          />
        );
      case 'corruptEnv': 
        return (
          <EnvironmentActionView 
            title="Environment Corrupted"
            description="Your environment is corrupt. Please create a new one."
            onAction={handleCreateEnvironment}
          />
        );
      case 'creationFailed':
        return (
          <EnvironmentActionView 
            title="Creation Failed"
            description="An error occurred while creating the environment. Please try again."
            buttonText="Try Again"
            onAction={handleCreateEnvironment}
          />
        );
      case 'ready': 
        return <StartView onStart={handleStart} />;
      default: 
        return null;
    }
  };

  return (
    <div className="app-layout">
      <main className="main-container">
        {renderMainView()}
      </main>
      
      <Footer 
        actionState={footerState} 
        onUnmount={handleUnmount} 
      />
    </div>
  );
}

export default App;