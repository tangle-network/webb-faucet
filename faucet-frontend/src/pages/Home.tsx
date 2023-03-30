import React, { useEffect } from "react";
import queryString from "query-string";
import TwitterAuth from "../components/TwitterAuth";
import axios from "axios";
import ClaimFundsForm from "../components/ClaimFundsForm";

const postData = async (code: string) => {
  const client_id = process.env.REACT_APP_TWITTER_CLIENT_ID || "";
  const response = await axios.post(
    // This will be proxied to https:
    // "https://api.twitter.com/2/oauth2/token"
    "/2/oauth2/token",
    {
      code,
      grant_type: "authorization_code",
      client_id: client_id,
      redirect_uri: process.env.PUBLIC_URL,
      code_verifier: "challenge",
    },
    {
      baseURL: process.env.PUBLIC_URL,
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
      }
    }
  );
  return response.data;
};

const authenticate = async (accessToken: string) => {
  const response = await axios.get(`/auth/twitter/?access_token=${accessToken}`, {
    baseURL: process.env.REACT_APP_BACKEND_URL || "http://localhost:8000",
  });
  console.log(response);
  return response.data;
};

const Home: React.FC = () => {
  const [accessToken, setAccessToken] = React.useState<string | null>(null);
  const [refreshToken, setRefreshToken] = React.useState<string | null>(null);
  const [scope, setScope] = React.useState<string | null>(null);
  const query = queryString.parse(window.location.search);
  const code = query.code;
  const state = query.state;

  const onLogin = async () => {
    const backendUrl = process.env.REACT_APP_BACKEND_URL || "http://localhost:8000";
    window.location.assign(`${backendUrl}/login/twitter`);
  };

  useEffect(() => {
    async function fetchData() {
      if (code && state) {
        const data = await postData(code as string);
        setAccessToken(data.access_token);
        setRefreshToken(data.refresh_token);
        setScope(data.scope);
        const authenticateResponse = await authenticate(data.access_token);
        console.log(authenticateResponse);
      }
    }

    fetchData();
  }, [code, state]);

  return (
    <div style={{ margin: "32px", padding: "32px" }}>
      <TwitterAuth onLogin={onLogin} />
      {code && state ? (
        <div style={{ marginTop: "32px" }}>
          <p>
            <strong>Code:</strong> {code}
          </p>
          <p>
            <strong>State:</strong> {state}
          </p>
        </div>
      ) : null}
      {accessToken && refreshToken ? (
        <div style={{ marginTop: "32px" }}>
          <p>
            <strong>Access Token:</strong> {accessToken}
          </p>
          <p>
            <strong>Refresh Token:</strong> {refreshToken}
          </p>
        </div>
      ) : null}
      <ClaimFundsForm accessToken={accessToken || ""} />
    </div>
  );
};

export default Home;
