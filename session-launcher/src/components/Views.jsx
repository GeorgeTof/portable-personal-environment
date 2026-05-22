import React, { useState, useEffect } from 'react';
import { 
  getBlacklistedUsers,
} from '../api';
import powerIcon from '../assets/on-off-button-icon.svg';

export const LoadingView = () => (
  <div className="view-container">
    <h2>Loading...</h2>
    <div className="spinner"></div>
  </div>
);

export const StartingUpView = () => (
  <div className="view-container">
    <h2>Starting up the environment...</h2>
    <div className="spinner"></div>
  </div>
);

export const LoadingSetupUserView = () => (
  <div className="view-container">
    <h2>Setting up the new user...</h2>
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
      <img src={powerIcon} alt="Power Icon" className="start-icon" />
      <span className="start-text">START</span>
    </button>
  </div>
);

export const SetupUserView = ({ onSubmit }) => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [blacklist, setBlacklist] = useState([]);

  useEffect(() => {
    const fetchBlacklist = async () => {
      const users = await getBlacklistedUsers();
      setBlacklist(users.map(u => u.toLowerCase())); 
    };
    fetchBlacklist();
  }, []);

  const handleSubmit = () => {
    if (!username || !password || !confirmPassword) {
      alert("Error: All fields must be filled out.");
      return;
    }

    if (blacklist.includes(username.toLowerCase())) {
      alert(`Error: The username "${username}" is reserved by the system. Please choose another.`);
      return;
    }

    if (username.includes("'")) {
      alert("Error: Username cannot contain single quotes.");
      return;
    }

    if (password.includes("'")) {
      alert("Error: Password cannot contain single quotes (').");
      return;
    }

    if (password !== confirmPassword) {
      alert("Error: Passwords do not match.");
      return;
    }
    onSubmit(username, password);
  };

  return (
    <div className="view-container">
      <h2>Initial Setup Required</h2>
      <p>Please configure your Linux user</p>
      
      <div className="form-group">
        <input 
          type="text" 
          placeholder="Username" 
          value={username} 
          onChange={e => setUsername(e.target.value)} 
        />
        <input 
          type="password" 
          placeholder="Password" 
          value={password} 
          onChange={e => setPassword(e.target.value)} 
        />
        <input 
          type="password" 
          placeholder="Repeat Password" 
          value={confirmPassword} 
          onChange={e => setConfirmPassword(e.target.value)} 
        />
      </div>

      <button className="btn primary" onClick={handleSubmit}>
        Create User
      </button>
    </div>
  );
};
