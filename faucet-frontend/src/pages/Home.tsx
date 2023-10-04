import React, { useEffect } from "react";
import queryString from "query-string";
import TwitterAuth from "../components/TwitterAuth";
import axios from "axios";
import ClaimFundsForm from "../components/ClaimFundsForm";

const postData = async (code: string) => {
  const client_id = process.env.REACT_APP_TWITTER_CLIENT_ID || "";
  const client_secret = process.env.REACT_APP_TWITTER_CLIENT_SECRET || "";
  const token = btoa(`${client_id}:${client_secret}`);
  const publicUrl = process.env.PUBLIC_URL || "http://localhost:3000";
  const response = await axios.post(
    // This will be proxied to https:
    // "https://api.twitter.com/oauth2/token"
    "/2/oauth2/token",
    {
      code,
      grant_type: "authorization_code",
      client_id: client_id,
      redirect_uri: publicUrl,
      code_verifier: "challenge",
    },
    {
      baseURL: publicUrl,
      headers: {
        "Content-Type": "application/x-www-form-urlencoded",
        "Authorization": `Basic ${token}`,
      },
    }
  );
  return response.data;
};

const Home: React.FC = () => {
  const [accessToken, setAccessToken] = React.useState<string | null>(null);
  const [refreshToken, setRefreshToken] = React.useState<string | null>(null);
  const query = queryString.parse(window.location.search);
  const code = query.code;

  const onLogin = async () => {
    const backendUrl = process.env.REACT_APP_BACKEND_URL || "http://localhost:8000";
    window.location.assign(`${backendUrl}/login/twitter`);
  };

  useEffect(() => {
    async function fetchData() {
      if (code && !accessToken && !refreshToken) {
        const data = await postData(code as string);
        setAccessToken(data.access_token);
        setRefreshToken(data.refresh_token);
      }
    }

    fetchData();
  }, [code, accessToken, refreshToken]);

  return (
    <div style={{ margin: "32px", padding: "32px" }}>
      {!accessToken && !refreshToken ? (
        <TwitterAuth onLogin={onLogin} />
      ) : null}
      <ClaimFundsForm accessToken={accessToken} />
    </div>
  );
};

export default Home;
