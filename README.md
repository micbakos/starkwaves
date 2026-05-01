# Starkwaves

## Introduction

Starkwaves is a Battleship game built on Starknet. It serves as a personal exploration/introduction project for me into gaming on blockchain, specifically around the problem of trusting your opponent's reporting without revealing private positions, as well as learning how to develop a smart contract on Starknet.

In a traditional Battleship game, both players place ships on a hidden board and take turns attacking coordinates. The defender must honestly report whether a shot was a hit or a miss. In a physical setting, trust is implicit. On a public blockchain, where all state is transparent, a different approach is needed to keep boards private while still guaranteeing honesty.

Starkwaves solves this using a commit-reveal scheme with Merkle proofs and Pedersen hashing. Players commit a cryptographic fingerprint of their board at the start of the game. During gameplay, each defense response is verified against this commitment using Merkle proofs. At the end of the game, both players reveal their full boards, and the contract verifies that everything matches what was committed and reported throughout the game.

## Project Structure

```
starkwaves/
  contract/       Cairo smart contract (Scarb + snforge)
  client/         Rust client application
  patches/        Local crate patches for starknet-rust-core and cainome-cairo-serde
```

## Setup and Running

### Prerequisites
- [starkup](https://github.com/software-mansion/starkup) Starknet toolchain installer
    - [scarb](https://docs.swmansion.com/scarb/) (Cairo package manager)
    - [snforge](https://foundry-rs.github.io/starknet-foundry/) (Starknet Foundry test runner)
    - [starknet-devnet](https://github.com/0xSpaceShard/starknet-devnet-rs) (local Starknet node)
- Rust toolchain (for the client)

### Environment Configuration
Different environments can be set, according to where do you deploy your contract.
1. Create an `.env` file with the following content
    ```
    # This helps the scripts identify which env to be used in order to deploy and run the game
    PRESET=sepolia # This config will use .env.sepolia 
    ```
2. Then create an `.env.<preset>` for each environment

    Copy `.env.example` to `.env.sepolia` (or `.env.devnet` for local development) and fill in the values:
    
    ```
    CHAIN_ID=              # e.g. SN_SEPOLIA
    DEPLOY_RPC_URL=        # RPC endpoint for deployment
    RPC_URL=               # RPC endpoint for gameplay
    WS_URL=                # WebSocket endpoint for events
    DEPLOY_ACCOUNT_NAME=   # sncast account name
    CONTRACT_ADDR=         # Deployed contract address (auto updated by deploy script)
    
    # The rest of the variables are used for testing the game with Player A/B presets.
    PRESET_A_PRIVATE_KEY=  # Player A private key
    PRESET_A_ADDRESS=      # Player A address
    PRESET_B_PRIVATE_KEY=  # Player B private key
    PRESET_B_ADDRESS=      # Player B address
    ```

### Running Contract Tests

Unit tests:

```bash
cd contract
scarb test
```

The e2e script starts a local `starknet-devnet` instance forking from the configured Sepolia RPC, waits for it to be ready, then runs snforge fork tests against it.

### Building and Deploying the Contract

```bash
cd client
# Deploys the contract into the network defined at .env 
cargo run --bin deploy
```

If you are the game owner you can also reset the state of all games. Helpful during developmet:
```bash
cd client
# Resets the contract's storage state to 0 
cargo run --bin reset
```

### Running the Client

```bash
cd client
# Run with private key
cargo run --package starkwaves-client --bin starkwaves-client -- -a 0x<player-address> -k 0x<player-private-key>

# Run from a preset for player A in env
cargo run --package starkwaves-client --bin starkwaves-client -- -p A
# Run from a preset for player B in env
cargo run --package starkwaves-client --bin starkwaves-client -- -p B
```

The client, at this moment is in command line form and accepts the following commands:
```
# Places each ship in a position on the board. 
# This command can be used only before the game starts. 
place <SHIP-ABBREVIATION> <x> <y> <h|v> # e.g. place CR 1 2 h (h: horizontal, v: vertical)

# When it is your turn to attack, use this 
# command to attack a coordinate on the board.
attack <x> <y>                          # e.g. attack 1 0 

# Prints the state of the boards. Your own board 
# and your view to the opponent's board with bomb information. 
boards

# Shows whose the current turn is.
turn

# Quit the game
quit
```

## Board Sizes and Ships

### Board Sizes

The game supports multiple board sizes. The board size determines which ships are available and how many of each.

| Dimensions | Type     |
|------------|----------|
| 6x6        | Smaller  |
| 8x8        | Smaller  |
| 10x10      | Standard |
| 12x12      | Larger   |
| 14x14      | Larger   | 
| 20x20      | Larger   |

### Ships

| Ship         | Abbreviation | Length |
|--------------|--------------|--------|
| Destroyer    | DE           | 2      |
| Cruiser      | CR           | 3      |
| Submarine    | SU           | 3      |
| Battleship   | BA           | 4      |
| Carrier      | CA           | 5      |
| SuperCarrier | SC           | 6      |

### Fleet Composition by Board Size

- **6x6 and 8x8**: 1 Cruiser, 1 Destroyer (5 total cells to hit)
- **10x10 (Standard)**: 1 of each ship except SuperCarrier (17 total cells to hit)
- **12x12, 14x14, and 20x20**: 1 SuperCarrier, 1 Carrier, 1 Battleship, 1 Cruiser, 2 Submarines, 2 Destroyers (27 total cells to hit)

A player wins when they have hit every cell occupied by the opponent's ships, i.e. the total hit count for that board size.

## Game Lifecycle

### 1. Matchmaking

A player calls `request_start_game` with a desired board size. If no opponent is waiting in that lobby, the player enters a queue. When a second player requests the same board size, a game is created and both are paired.

### 2. Board Commitment

Each player arranges their ships locally and computes a Merkle tree over the board. The board is represented as a **flat array of boolean values** (ship present or not) in row-major order. The leaves are hashed with a secret **salt** to produce a Merkle root.

Both players submit their Merkle root on-chain via `commit_board`. Once both roots are committed, the game begins.

### 3. Attack and Defense Cycle

The game alternates between attack and defense:

1. The attacking player calls `attack(game_id, x, y)`, registering the bombed coordinate on-chain.
2. The defending player responds with `defend(game_id, status, proof)`:
   - `status` is either `FireStatus::Miss(salted_hash)` or `FireStatus::Hit(Option<ShipKind>, salted_hash)` where the salted hash is `pedersen(cell_value, salt)`.
   - `proof` is the Merkle proof for the attacked cell against the defender's committed root.
   - If a ship is fully destroyed, the defender includes the `ShipKind` in the hit status.

The contract verifies the Merkle proof against the defender's committed root. If verification fails, **the game ends immediately** with the defender marked as a cheater.

On each confirmed destruction, the contract chains `pedersen(destruction_hash, ship_kind_id)` into a running destruction hash for that player. This hash is later used during reveal to confirm that destruction claims were consistent with the actual board.

The game ends when one player sinks all opponent ships (total hit count reaches the expected number for the board size).

### 4. Reveal Phase

Once the game ends, both players must reveal their boards by calling `reveal(game_id, ships, salt)` with their full ship placements and the salt used during commitment.

The contract performs two verifications for each player:

**Merkle root verification:** The revealed ships are converted back into a boolean board, hashed with the provided salt to recompute a Merkle root, and compared against the originally committed root. This proves the revealed board matches what was committed at the start.

**Destruction hash verification:** The contract replays all bombs thrown at this player's board in chronological order. For each bomb that lands on a ship cell, it tracks cumulative hits per ship. When a ship's hit count reaches its length, it chains `pedersen(hash, ship_kind_id)` into a reconstructed destruction hash. This reconstructed hash is compared against the destruction hash accumulated during gameplay. This proves that destruction claims made during the defend phase were truthful.

### 5. Final Outcome

After both players reveal, the contract determines the final outcome:

- **Fair**: Both players were honest throughout. The player who sank all ships wins.
- **FailedToProvideProof**: One player cheated (failed Merkle proof during gameplay or fake board at reveal). The honest player wins.
- **Null**: Both players cheated. No winner.

### On-Chain Verification Summary

| What is verified                         | When             | How                                        |
|------------------------------------------|------------------|--------------------------------------------|
| Defense response matches committed board | Each defend call | Merkle proof against committed root        |
| Revealed board matches commitment        | Reveal phase     | Recompute Merkle root from ships + salt    |
| Destruction claims were truthful         | Reveal phase     | Replay bombs, reconstruct destruction hash |
| Ship placements are valid (no overlaps)  | Reveal phase     | Collision check during hull construction   |

This design ensures that players cannot lie about hits, misses, or ship destructions without being caught at reveal time. The only information visible on-chain during gameplay is which coordinates were attacked and whether they were hits or misses. The actual ship positions remain private until the game concludes.

## Caveats and Future Considerations

- [ ] **The CLI client is minimal.** It exists as a proof of concept for interacting with the contract. The client can serve as middleware for any game frontend that wants to provide a richer UI on top of the same contract.
- [X] ~~**No time limits are enforced.** The contract does not currently penalize players for inactivity. A player can start a game and stall indefinitely without consequence. A timeout mechanism could be added in the future to forfeit inactive players.~~
- [X] ~~**Contract reset does not notify clients.** When the contract owner calls `reset`, all game state is cleared, but connected clients are not aware of this. In the future, clients should handle this gracefully, either by listening for a reset event or by detecting stale game state.~~
- [ ] **No game resumption after disconnect.** If a player quits, the contract is not notified and there is currently no way to resume the game from the client. This is fairly easily fixable: if the player remembers the salt and reconstructs the board, the Merkle root can be recomputed, verified against the on-chain commitment, and the game can resume since all state is stored on-chain.
- [ ] **Ships are linear only.** All ships occupy a straight line of cells, either horizontal or vertical. Non-linear shapes (e.g. L-shaped ships) are not currently supported but could be added without affecting the core commit-reveal and verification logic.
- [ ] **No indexer, no rankings.** The game relies solely on a direct RPC connection to a Starknet node. There is no indexer involved, so there are no rankings, lobby tables, or historical game data available beyond what can be read from on-chain state and events.

## License

This project is licensed under the [MIT License](LICENSE).
