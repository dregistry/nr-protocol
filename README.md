# Near Registry

This smart contract is for creation and management of on-chain registries.

## Basic
<details>
<summary>3-Step Rust Installation.</summary>
<p>

1. Install Rustup:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

[_(Taken from official installation guide)_](https://www.rust-lang.org/tools/install)

2. Configure your current shell:

```
source $HOME/.cargo/env
```

3. Add Wasm target to your toolchain:

```
rustup target add wasm32-unknown-unknown
```

</p>
</details>

After, you need to change your version to nightly

```rustup toolchain install nightly-2022-08-14 && rustup override set nightly-2022-08-14 && rustup target add wasm32-unknown-unknown --toolchain nightly-2022-08-14```

After, build the contract itself

```make build```

To send a contract to the network, you need to build it and create a folder for the file that will be sent to the blockchain

```mkdir out && cp target/wasm32-unkown-unkown/release/near_registry.wasm out/```

After, send the contract itself to the desired environment (by default, testnet)

if testnet:

```near dev-deploy --wasm_file out/near_registry.wasm```

if mainnet:

```near deploy --wasm_file out/near_registry.wasm --account_id YOUR_ACCOUNT_ID```


Next, after deploying the contract, you need to initialize it

```near call YOUR_CONTRACT_ADDRESS init '{"owner_id": "YOUR_ACCOUNT_ID", "dao": "ASTRO_DAO_CONTRACT"}' --account_id YOUR_ACCOUNT_ID```

## Create your first registry

```near call YOUR_CONTRACT_ADDRESS new_registry '{"owner": "YOUR_ADDRESS", "column_data": "['{"SOME_VALUE":"value"}']", "row_data": "['{"SOME_VALUE": "value"}']", "name": "SOME_NAME"}' --account_id YOUR_ADDRESS```

## Voting prosess

First you need to go and get tokens for voting on AstroDAO, then, if you wanna send your proposal for change registry, you need to send a method

```near call YOUR_CONTRACT_ID --account_id YOUR_ACCOUNT_ID add_proposal '{"proposal": {"owner": "OWNER_THIS_REGISTRY", "description": "Some description", "kind": "Vote", "registry_data": {"name": "John Doe","age": 44, "name": "test"}, "unique_identifier": "test.testnet"}}'```

### First of all, compare all data for adding proposal

For voting on some proposal need send method

```near call YOUR_CONTRACT_ID --account YOUR_ADDRESS act_proposal '{"id": "ID_OF_PROPOSAL", "action": "VoteApprove", "amount": "YOUR_AMOUNT"}'```