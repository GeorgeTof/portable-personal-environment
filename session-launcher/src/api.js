const delay = (ms) => new Promise(resolve => setTimeout(resolve, ms));

export const checkHostConfig = async () => {
  await delay(500);
  return true; // Dummy return values, TODO change
};

export const checkDriveConfiguration = async () => {
  await delay(500);
  return true;
};

export const checkContainerIntegrity = async () => {
  await delay(500);
  return true;
};

export const createEnvironment = async () => {
  await delay(1500);
  return true; 
};

export const startContainer = async () => { 
  await delay(500); 
  return true;
};

export const checkContainerStarted = async () => { 
  await delay(500); 
  return true;
};

export const checkContainerRuntimeIntegrity = async () => { 
  await delay(500); 
  return true;
};

export const checkConfiguredForPersonalUse = async () => { 
  await delay(500); 
  return false;
};

export const configureUser = async (username, password) => {
  await delay(500); 
  return false;
};