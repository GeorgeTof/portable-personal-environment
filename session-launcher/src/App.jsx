import React, { useState, useEffect } from 'react';
import { 
  checkHostConfig, 
  checkDriveConfiguration, 
  checkContainerIntegrity,
  createEnvironment,
  startContainer,
  checkContainerStarted,
  checkContainerRuntimeIntegrity,
  checkConfiguredForPersonalUse,
  configureUser
} from './api';
import { 
  LoadingView,
  StartingUpView, 
  HostErrorView, 
  EnvironmentActionView, 
  StartView,
  SetupUserView,
  LauncherView,
} from './components/Views';
import { Footer } from './components/Footer';
import './App.css';

function App() {
  const [viewState, setViewState] = useState('loading'); // loading, hostError, missingEnv, corruptEnv, ready, creationFailed
  const [footerState, setFooterState] = useState('none'); // none, unmount, stop
  const [startError, setStartError] = useState(null);

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

  const handleStart = async () => {
    setStartError(null);
    setViewState('startingUp');
    setFooterState('stop'); // Immediately switch footer to stop

    if (!(await startContainer())) {
      setStartError("Failed to issue the start command.");
      setViewState('ready');
      setFooterState('unmount');
      return;
    }
    
    if (!(await checkContainerStarted())) {
      setStartError("Container failed to initialize in time.");
      setViewState('ready');
      setFooterState('unmount');
      return;
    }
    
    if (!(await checkContainerRuntimeIntegrity())) {
      setStartError("Container runtime integrity check failed.");
      setViewState('ready');
      setFooterState('unmount');
      return;
    }
    
    if (!(await checkConfiguredForPersonalUse())) {
      setViewState('setupUser');
      // keep footer state as 'stop' since the container is running
      return;
    }
    
    setViewState('launcher');
  };

  const handleUnmount = () => {
    console.log("Unmounting drive... (TODO workaround script and close app)");
  };

  const handleStop = () => {
    console.log("Stopping container... (API call to be implemented)");
    // TODO: rust command that tries a clean approach with -t <seconds> timeout first, then force it with -K
    // maybe if error, have a popup with the error message and return OR give the user there the option ot force kill the container
    setStartError(null);
    setViewState('ready');
    setFooterState('unmount');
  };

  const handleConfigureUser = async (username, password) => {
    setViewState('loading');
    const success = await configureUser(username, password);
    
    if (success) {
      setViewState('launcher');
    } else {
      alert("Backend Error: Failed to configure the Linux user. Please try again.");
      setViewState('setupUser');
    }
  };

  const handleRefresh = () => {
    console.log("TODO Refresh button pressed!");
  };


  const renderMainView = () => {
    switch (viewState) {
      case 'loading': 
        return <LoadingView />;
      case 'startingUp': 
        return <StartingUpView />;
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
        return <StartView onStart={handleStart} errorMessage={startError} />;
      case 'setupUser':
        return <SetupUserView onSubmit={handleConfigureUser} />;
      case 'launcher':
        return <LauncherView />;
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
        onStop={handleStop}
        showRefresh={viewState === 'launcher'}
        onRefresh={handleRefresh}
      />
    </div>
  );
}

export default App;