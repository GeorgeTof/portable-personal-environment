import { invoke } from '@tauri-apps/api/core';

const runTauriCommand = async (commandName, args = {}) => {
  try {
    const [success, errorMessage] = await invoke(commandName, args);
    
    if (!success && errorMessage) {
      alert(`Error: ${errorMessage}`);
    }
    
    return success;
  } catch (error) {
    alert(`Fatal Application Error: ${error}`);
    return false; 
  }
};

const runTauriCommandWithAlternative = async (commandName, args = {}) => {
  try {
    const [success, errorMessage] = await invoke(commandName, args);
    
    if (!success && errorMessage) {
      // catch expected error for stopping the container
      if (errorMessage === "STOP-TIMEOUT") {
        const force = window.confirm("The container is taking too long to shut down (an app might be hanging). Would you like to force stop it?");
        
        if (force) {
          const [forceSuccess, forceError] = await invoke(commandName, { ...args, force: true });
          if (!forceSuccess && forceError) alert(`Force stop failed: ${forceError}`);
          return forceSuccess;
        }
        
        return false; 
      } else {
        alert(`Error: ${errorMessage}`);
      }
    }
    
    return success;
  } catch (error) {
    alert(`Fatal Application Error: ${error}`);
    return false; 
  }
};


export const checkHostConfig = () => runTauriCommand('check_host_config');
export const checkDriveConfiguration = () => runTauriCommand('check_drive_configuration');
export const checkContainerIntegrity = () => runTauriCommand('check_container_integrity');
export const checkContainerStarted = () => runTauriCommand('check_container_started');
export const checkContainerRuntimeIntegrity = () => runTauriCommand('check_container_runtime_integrity');
export const checkConfiguredForPersonalUse = () => runTauriCommand('check_configured_for_personal_use');


export const createEnvironment = () => runTauriCommand('create_environment');
export const configureUser = (username, password) => 
  runTauriCommand('configure_user', { username, password });


export const startContainer = () => runTauriCommand('start_container');
export const unmountDrive = () => runTauriCommand('unmount_drive');

export const stopContainer = (force = false) => runTauriCommandWithAlternative('stop_container', { force });

export const forceStopForUnmount = async () => {
  try {
    const [success, errorMessage] = await invoke('stop_container', { force: true });
    
    if (success || errorMessage === "NOT_RUNNING") {
      return true;
    }
    
    return window.confirm(
      `Failed to safely stop the container:\n"${errorMessage}"\n\nDo you want to proceed with an unsafe unmount of the drive at your own risk?`
    );
  } catch (error) {
    return window.confirm(
      `Fatal error stopping container:\n"${error}"\n\nProceed with unsafe unmount?`
    );
  }
};