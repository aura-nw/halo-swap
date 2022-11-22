# cw4973-scripts
Scripts to interact with cw4973 contract.
Before use these scripts, you must modify the mnemonic of deployer account and tester account in the /config/chain.js

To store code of contract:
```bash
node store.js
```

To instantiate the contract:
```bash
node instantiate.js <codeID>
```

To take a nft:
```bash
node --trace-warnings ./scripts/mint.js <contract_address> <metadata_uri>
```

To unequip a nft:
```bash
node --trace-warnings ./scripts/unequip.js <contract_address> <token_id>
```
