import React from 'react';

export const Footer = ({ actionState, onUnmount, onStop }) => {
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
        {/* Future utilities like settings or status indicators can go here */}
      </div>
    </div>
  );
};