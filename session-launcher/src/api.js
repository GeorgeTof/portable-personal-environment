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
  return false;
};

export const createEnvironment = async () => {
  await delay(1500);
  return false; 
};