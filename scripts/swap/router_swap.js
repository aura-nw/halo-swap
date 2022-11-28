const chainConfig = require('./config/chain').defaultChain;

const fs = require('fs');

const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet } = require('@cosmjs/proto-signing');
const { GasPrice } = require('@cosmjs/stargate');

async function swap(_contract) {
    const testerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        chainConfig.tester_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );

    // gas price
    const gasPrice = GasPrice.fromString(`0.025${chainConfig.denom}`);

    // connect tester wallet to chain
    const testerClient = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, testerWallet, {gasPrice});

    // get tester account
    const testerAccount = (await testerWallet.getAccounts())[0];

    const memo = "swap";

    const tokenAAddress = "aura15pp33amcmxd703szqrtp8fvxdsm9kf4jh9x29920qa6x362g04asf62v23";
    const tokenBAddress = "aura19sqmgzw09qz337wpu7pvu03yvz08s8n4lx89ul657x4kx07mekqs6ysgez";

    // define the hook cw20 message
    const hookMsg = {
        "execute_swap_operations": {
            "operations": [
                {
                    "halo_swap":{
                        "offer_asset_info": {
                            "token": {
                                "contract_addr": tokenAAddress
                            }
                        },
                        "ask_asset_info": {
                            "token": {
                                "contract_addr": tokenBAddress
                            }
                        }
                    }
                }
            ],
            // "minimum_receive": Option<Uint128>,
            // "to": Option<String>,
        }
    }

    // define the exectute send for cw20
    const executeSendMsg = {
        "send": {
            "contract": _contract,
            "amount": "100000000",
            "msg": Buffer.from(JSON.stringify(hookMsg)).toString('base64')
        }
    }

    console.log("executeSendMsg: ", executeSendMsg);

    console.log("testerAccount.address: ", testerAccount.address);
    // send the cw20 token to contract
    const takeResponse = await testerClient.execute(testerAccount.address, tokenAAddress, executeSendMsg, "auto", memo);

    console.log(takeResponse);
}

const myArgs = process.argv.slice(2);
swap(myArgs[0]);
