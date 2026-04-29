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
export const stopContainer = () => runTauriCommand('stop_container');
export const unmountDrive = () => runTauriCommand('unmount_drive');