# Substrate Kitties Pallet 🐱

A decentralized "CryptoKitties" implementation built with Substrate and the Polkadot SDK. This pallet allows users to create, breed, transfer, and trade unique digital cats on-chain.

## 🚀 Overview

This pallet serves as a comprehensive example of how to manage complex state transitions and digital ownership in a Substrate runtime. It leverages cryptographic hashing for unique identity and bitwise operations for genetic inheritance.

## ✨ Core Features

### 🧬 DNA & Breeding

Unique DNA: Every kitty has a [u8; 32] DNA string.

Genetic Crossover: When breeding, the child's DNA is a mix of the parents' DNA, determined by a random hash generated from the block's entropy (parent hash and block number).

### 💰 Marketplace

Internal Economy: Users can set prices for their kitties using the NativeBalance trait.

Secure Purchases: The buy_kitty function ensures atomic transfers: the buyer gets the kitty, the seller gets the funds, and all storage is updated simultaneously.

### 📦 Optimized Storage

Identity Map: Kitties stores the full details of every cat.

Ownership Index: KittiesOwned uses a BoundedVec to quickly retrieve all kitties belonging to a specific account without scanning the whole state.

### 🛠 Technical Implementation

| Feature                    | Status | Description                                                                     |
|----------------------------|--------|---------------------------------------------------------------------------------|
| Storage Items              | -      | -                                                                               |
| CountForKitties            | Ok     | Value query integer tracking total Kitties                                      |
| Kitties                    | Ok     | A StorageMap from DNA [u8; 32] to a Kitty struct                                |
| KittiesOwned               | Ok     | A StorageMap from AccountId to a BoundedVec of DNA hashes (limit: 100 per user) |
| Dispatchables (Extrinsics) | -      | -                                                                               |
| create_kitty               | Ok     | Mints a new kitty with a unique DNA based on block entropy                      |
| breed_kitties              | Ok     | Combines two existing kitties to create a new one                               |
| transfer                   | Ok     | Moves ownership of a kitty to another account                                   |
| set_price                  | Ok     | Lists a kitty for sale or updates its price                                     |
| buy_kitty                  | Ok     | Handles the payment and transfer logic for a sale                               |
| abandon_kitty              | Ok     | Deletes a kitty from the chain and frees up storage (poor kitty)                |

## 💻 Integration

Rust (Pallet Config)

```
impl pallet_kitties::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type NativeBalance = Balances;
}



```
JavaScript (Frontend)
To fetch a user's kitties using @polkadot/api:
```
const dnas = await api.query.kittiesPallet.kittiesOwned(ALICE_ADDRESS);
const details = await api.query.kittiesPallet.kitties.multi(dnas);
console.table(details.map(d => d.unwrap().toJSON()));



```
🛡️ Safety & Security

Checked Math: Uses checked_add to prevent total count overflows.

Panic Protection: Uses ensure! to check for duplicates and ownership before every write operation.

Bounded Collections: KittiesOwned is bounded to prevent storage bloat and "Weight" attacks.

👤 Author

Junior Developer | Rust & Node.js Enthusiast
Currently exploring the Polkadot ecosystem and building custom runtimes. I am open to mentorship, project ideas, and collaboration!

Created as part of the Polkadot Blockchain Training.
