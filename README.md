<div align="center">
<a href="https://www.webb.tools/">

![Webb Logo](./assets/webb_banner_light.png#gh-light-mode-only)

![Webb Logo](./assets/webb_banner_dark.png#gh-dark-mode-only)
</a>

  </div>

# Webb Faucet

<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents" style=border:0!important> 📖 Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ul>
    <li><a href="#start"> Getting Started</a></li>
    <li><a href="#usage">Usage</a></li>
  </ul>  
</details>

<h2 id="start"> Getting Started  🎉 </h2>

This is a multi-chain faucet for Webb's test bridges. The faucet backend is written in Rust and the frontend is written in React. The backend is responsible for authenticating a twitter user and verifying they follow our
twitter account: [@webbprotocol](https://twitter.com/webbprotocol). Once authenticated, the backend will send a transaction to the user's provided address on the form.

Webb's testnets are currently deployed to EVMs and Substrate chains. If you are an EVM or Substrate chain that wants to integrate with the Webb protocol, please reach out to us on [Discord](https://discord.gg/d88MzS8h)!

### Prerequisites

This repo uses Rust so it is required to have a Rust developer environment set up. First install and configure rustup:

```bash
# Install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Configure
source ~/.cargo/env
```

Configure the Rust toolchain to default to the latest stable version:

```bash
rustup default stable
rustup update
```

Great! Now your Rust environment is ready! 🚀🚀

## Usage

Starting the Rust serve requires you to create a `Rocket.toml` file specified with your Twitter Developer App's credentials. You can find the template in the `Rocket.example.toml` file. You will also need to create a new Twitter Application for development purposes. You can find the instructions [here](https://developer.twitter.com/en/docs/twitter-api/getting-started/getting-access-to-the-twitter-api).

**Notes:**

1. For 'App permissions' you will need to select 'Read' permissions.
2. For 'Type of App' you will need to select 'Native App'.
3. For 'App info' you will need to fill in the 'Website URL' and 'Callback URLs' fields. The 'Callback URLs' field should be set to `http://127.0.0.1:3000` and `http://localhost:3000`.

Once created, you can run the server with the following command:

```rust
cargo run
```

Starting the React application requires you to create a `.env` file specified with some of your Twitter Developer App's credentials. You can find the template in the `./faucet-frontend/.env.example` file. The `.env` should be placed in the `./faucet-frontend` directory.

The frontend can be started with the following commands:

```bash
cd faucet-frontend
yarn start
```

## User Flow

1. User clicks the "Log in with Twitter" button
2. User is redirected and authorizes the app to access their Twitter account.
3. Once the access token is generated and displayed, the user should supply the receiving addressand claim their tokens by clicking the "Claim" button.

There is no actually blockchain logic hooked in yet.
