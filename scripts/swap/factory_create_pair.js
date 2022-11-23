const chainConfig = require('./config/chain').defaultChain;

const fs = require('fs');

const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet } = require('@cosmjs/proto-signing');
const { GasPrice } = require('@cosmjs/stargate');

async function CreatePair(_contract) {
    const deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        chainConfig.deployer_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );
    const account = (await deployerWallet.getAccounts())[0];

    // gas price
    const gasPrice = GasPrice.fromString(`0.025${chainConfig.denom}`);
    const client = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, deployerWallet, {gasPrice});
    
    const tokenAAddress = "aura1ecg23djdn0zsdd83nuqd0fpfdfryznmj0w0qf7yv7ay7npsmuucsj29n00";
    const tokenBAddress = "aura1uj77443jlkutrvtteru8p32zkl2vz3nu35pjjsjqcmrfch8hhvms0ysrj4";

    const memo = "create pair";
    // define the take message using the address of deployer, uri of the nft and permitSignature
    const ExecuteCreatePairMsg = {
        "create_pair": {
            "asset_infos": [
                {
                    "token": {
                        "contract_addr": tokenAAddress
                    }
                },
                {
                    "token": {
                        "contract_addr": tokenBAddress
                    }
                }
            ],
        }
    }

    console.log("ExecuteCreatePairMsg: ", ExecuteCreatePairMsg);

    // take a NFT
    const takeResponse = await client.execute(account.address, _contract, ExecuteCreatePairMsg, "auto", memo);

    console.log(takeResponse);
}

const myArgs = process.argv.slice(2);
CreatePair(myArgs[0]);
