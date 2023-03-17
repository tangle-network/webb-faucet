import React, { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

const Auth: React.FC = () => {
  const navigate = useNavigate();

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const oauth_token = params.get('oauth_token');
    const oauth_verifier = params.get('oauth_verifier');

    if (oauth_token && oauth_verifier) {
      // Do something with the tokens, like sending them to your server
      console.log('oauth_token:', oauth_token);
      console.log('oauth_verifier:', oauth_verifier);

      // Redirect the user back to the home page or another authenticated route
      navigate('/');
    } else {
      // Redirect the user back to the login page if authentication fails
      navigate('/');
    }
  }, [navigate]);

  return <div>Authenticating...</div>;
};

export default Auth;
