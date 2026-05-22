import React from 'react';
import powerIcon from '../assets/on-off-button-icon.svg';
import ejectIcon from '../assets/eject-icon.svg';

export const Footer = ({ actionState, onUnmount, onStop, showRefresh, onRefresh, showRuntimeCheck, onRuntimeCheck }) => {
  return (
    <div className="footer">
      <div className="footer-left">
        {actionState === 'unmount' && (
          <button className="btn danger flex-btn" onClick={onUnmount}>
            <img src={ejectIcon} alt="Eject Icon" className="small-btn-icon" />
            <span>Unmount Drive</span>
          </button>
        )}
        {actionState === 'stop' && (
          <button className="btn warning flex-btn" onClick={onStop}>
            <img src={powerIcon} alt="Power Icon" className="small-btn-icon" />
            <span>Stop</span>
          </button>
        )}
      </div>
      <div className="footer-right">
        {showRuntimeCheck && (
          <button className="btn" onClick={onRuntimeCheck} style={{ backgroundColor: '#45475a', color: '#cdd6f4', marginRight: '10px' }}>
            Check Integrity
          </button>
        )}
        {showRefresh && (
          <button className="btn" onClick={onRefresh} style={{ backgroundColor: '#45475a', color: '#cdd6f4' }}>
            Refresh
          </button>
        )}
      </div>
    </div>
  );
};