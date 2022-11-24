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

    // define the hook cw20 message
    const hookMsg = {
        "execute_swap_operations": {
            "operations": [
                {
                    "aura_swap":{
                        "offer_asset_info": {
                            "token": {
                                "contract_addr": "aura1smcy8kged97n493cgh6jzrda0hahge4j6qrg0hx2j5tgwy2259hshpxzyv"
                            }
                        },
                        "ask_asset_info": {
                            "token": {
                                "contract_addr": "aura19xm0cnz5j6m8q6jdt2hyldld8wxpd33cym8s6l99kfnfjh6gcstsk4sjua"
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
            "amount": "1000000000",
            "msg": Buffer.from(JSON.stringify(hookMsg)).toString('base64')
        }
    }

    console.log("executeSendMsg: ", executeSendMsg);

    // send the cw20 token to contract
    const takeResponse = await testerClient.execute(testerAccount.address, "aura1smcy8kged97n493cgh6jzrda0hahge4j6qrg0hx2j5tgwy2259hshpxzyv", executeSendMsg, "auto", memo);

    console.log(takeResponse);
}

const myArgs = process.argv.slice(2);
swap(myArgs[0]);
