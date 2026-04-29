import React from 'react';

export const Footer = ({ actionState, onUnmount, onStop, showRefresh, onRefresh }) => {
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
        {showRefresh && (
          <button className="btn" onClick={onRefresh} style={{ backgroundColor: '#45475a', color: '#cdd6f4' }}>
            Refresh
          </button>
        )}
      </div>
    </div>
  );
};