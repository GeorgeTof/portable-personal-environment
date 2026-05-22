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
          <button className="btn secondary" onClick={onRuntimeCheck}>
            Check Integrity
          </button>
        )}
        {showRefresh && (
          <button className="btn secondary" onClick={onRefresh}>
            Refresh
          </button>
        )}
      </div>
    </div>
  );
};