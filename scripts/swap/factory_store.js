const chainConfig = require('./config/chain').defaultChain;

const fs = require('fs');

const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet } = require('@cosmjs/proto-signing');
const { calculateFee, GasPrice } = require('@cosmjs/stargate');

async function store() {
    // Deletes ALL existing entries
    if (process.env.DB_RESET || process.env.NODE_ENV === 'test') {
        await knex('standard_contracts').del();
    }

    const deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        chainConfig.deployer_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );

    const client = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, deployerWallet);
    const gasPrice = GasPrice.fromString(`0.025${chainConfig.denom}`);
    const uploadFee = calculateFee(2500000, gasPrice);
    const account = (await deployerWallet.getAccounts())[0];

    const haloFactory = fs.readFileSync(`${__dirname}/../../artifacts/halo_factory.wasm`);
    const haloFactoryResponse = await client.upload(account.address, haloFactory, uploadFee, 'Upload halo-factory contract code');
    console.log(haloFactoryResponse);
}

store()