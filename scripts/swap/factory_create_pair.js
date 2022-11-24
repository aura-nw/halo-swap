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
    
    const tokenAAddress = "aura1smcy8kged97n493cgh6jzrda0hahge4j6qrg0hx2j5tgwy2259hshpxzyv";
    const tokenBAddress = "aura19xm0cnz5j6m8q6jdt2hyldld8wxpd33cym8s6l99kfnfjh6gcstsk4sjua";

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
