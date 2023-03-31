import axios, { AxiosResponse } from "axios";
import React, { useState } from "react";
import "./ClaimFundsForm.css";

interface ClaimFundsFormProps {
  accessToken: string | null;
}

type EvmChain = { Evm: number };
type SubstrateChain = { Substrate: number };
type Chain = EvmChain | SubstrateChain;

const ClaimFundsForm: React.FC<ClaimFundsFormProps> = ({ accessToken }) => {
  const [chain, setChain] = useState<Chain>({ Evm: 5 });
  const [address, setAddress] = useState<string>("");

  const claimFunds = async () => {
    // Implement your claim funds logic here
    let result;
    try {
      const backendUrl = process.env.REACT_APP_BACKEND_URL || "http://localhost:8000";
      result = await axios.post(
        "/faucet",
        JSON.stringify({
          faucet: {
            walletAddress: {
              type: (chain as EvmChain).Evm
                ? "ethereum"
                : (chain as SubstrateChain).Substrate
                ? "substrate"
                : "Unknown",
              value: address,
            },
            typedChainId: {
              type: (chain as EvmChain).Evm ? "Evm" : "Substrate",
              id: (chain as EvmChain).Evm || (chain as SubstrateChain).Substrate,
            },
          },
        }),
        {
          headers: {
            "Content-Type": "application/x-www-form-urlencoded",
            "Access-Control-Allow-Origin": "*",
            Authorization: "Bearer " + accessToken,
          },
          baseURL: backendUrl,
        }
      );
    } catch (error) {
      console.error("Error response:");
      console.error((error as any).response.data); // ***
      console.error((error as any).response.status); // ***
      console.error((error as any).response.headers); // ***
      alert("Error claiming funds: check the console for more details");
      return;
    }

    // Parse the response and display the result to the user
    console.log("Funds claimed successfully");
    console.log((result as AxiosResponse).data);

    alert("Funds claimed successfully");
  };

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    if (accessToken === null) {
      console.log("Not logged in");
      return;
    }
    console.log("Submitting form");
    event.preventDefault();
    console.log(`Claiming funds for chain: ${chain}, address: ${address}`);
    claimFunds();
  };

  return (
    <form className="claim-funds-form" onSubmit={handleSubmit}>
      <label htmlFor="chain">Chain:</label>
      <select
        className="chain-selector"
        id="chain"
        multiple={false}
        value={JSON.stringify(chain)}
        onChange={(event) => setChain(JSON.parse(event.target.value) as Chain)}
      >
        <option value={JSON.stringify({ Evm: 5 })}>Gorli</option>
        <option value={JSON.stringify({ Evm: 80001 })}>Mumbai</option>
        <option value={JSON.stringify({ Substrate: 1080 })}>Tangle Standalone</option>
      </select>

      <label htmlFor="address">Address:</label>
      <input
        type="text"
        id="address"
        value={address}
        onChange={(event) => setAddress(event.target.value)}
      />

      <button
        className="submit-button"
        disabled={accessToken === null ? true : false}
        type="submit"
      >
        Claim
      </button>
    </form>
  );
};

export default ClaimFundsForm;
