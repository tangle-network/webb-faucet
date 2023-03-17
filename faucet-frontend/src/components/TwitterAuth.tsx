import React from 'react';
import './TwitterAuth.css';

interface TwitterAuthProps {
  onLogin: () => void;
}

const TwitterAuth: React.FC<TwitterAuthProps> = ({ onLogin }) => {
  return (
    <button className="twitter-auth-button" onClick={onLogin}>
      <img
        src={`${process.env.PUBLIC_URL}/TwitterIconWhite.png`}
        alt="Twitter Logo"
        className="twitter-logo"
      />
      Log in with Twitter
    </button>
  );
};

export default TwitterAuth;
