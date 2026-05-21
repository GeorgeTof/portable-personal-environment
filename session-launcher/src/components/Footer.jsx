import React from 'react';

export const Footer = ({ actionState, onUnmount, onStop, showRefresh, onRefresh, showRuntimeCheck, onRuntimeCheck }) => {
  return (
    <div className="footer">
      <div className="footer-left">
        {actionState === 'unmount' && (
          <button className="btn danger" onClick={onUnmount}>
            Unmount Drive
          </button>
        )}
        {actionState === 'stop' && (
          <button className="btn warning" onClick={onStop}>
            Stop Container
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