import axios, { AxiosResponse } from 'axios';
import React, { useState } from 'react';
import './ClaimFundsForm.css';

interface ClaimFundsFormProps {
  accessToken: string;
}

const ClaimFundsForm: React.FC<ClaimFundsFormProps> = ({accessToken }) => {
  const [chain, setChain] = useState<number>(0);
  const [address, setAddress] = useState<string>('');

  const claimFunds = async () => {
    // Implement your claim funds logic here
    let result;
    try {
      const backendUrl = process.env.REACT_APP_BACKEND_URL || "http://localhost:8000";
      result = await axios.post('/faucet',
        JSON.stringify({
          faucet: {
            address: address,
            typed_chain_id: String(chain),
          },
          oauth: {
            access_token: accessToken,
          }
        }), {
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
            'Access-Control-Allow-Origin': '*',
            'Authorization': 'Bearer ' + accessToken,
          },
          baseURL: backendUrl,
        }
      );
    } catch (error) {
      console.error("Error response:");
      console.error((error as any).response.data);    // ***
      console.error((error as any).response.status);  // ***
      console.error((error as any).response.headers); // ***
    }

    // Parse the response and display the result to the user
    console.log('Funds claimed successfully');
    console.log((result as AxiosResponse).data);

    // Reset the form
    setChain(0);
    setAddress('');
  };

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    console.log('Submitting form');
    event.preventDefault();
    console.log(`Claiming funds for chain: ${chain}, address: ${address}`);
    claimFunds();
  };

  return (
    <div className="formWrapper">
      <form onSubmit={handleSubmit}>
        <label htmlFor="chain">Chain:</label>
        <select
          id="chain"
          value={chain}
          onChange={(event) => setChain(Number(event.target.value))}
        >
          <option value={0}>Chain 0</option>
          <option value={1}>Chain 1</option>
          <option value={2}>Chain 2</option>
        </select>

        <label htmlFor="address">Address:</label>
        <input
          type="text"
          id="address"
          value={address}
          onChange={(event) => setAddress(event.target.value)}
        />

        <button type="submit">Claim</button>
      </form>
    </div>
  );
};

export default ClaimFundsForm;
