const chainConfig = require('./config/chain').defaultChain;

const fs = require('fs');

const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet } = require('@cosmjs/proto-signing');
const { GasPrice } = require('@cosmjs/stargate');

async function ProvideLiquidity(_contract) {
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
    
    // const pairAddress = "aura165m56tck3x86nxvndegj2fe647lke0qgh40832qejqsm2agpwslsrkx77f";

    const tokenAAddress = "aura15pp33amcmxd703szqrtp8fvxdsm9kf4jh9x29920qa6x362g04asf62v23";
    const tokenBAddress = "aura19sqmgzw09qz337wpu7pvu03yvz08s8n4lx89ul657x4kx07mekqs6ysgez";

    // create query message to get the pair infor
    const queryMsg = {
        "pair":{}
    };

    // execute the query
    const pairInfo = await client.queryContractSmart(_contract, queryMsg);
    console.log("pairInfo: ", JSON.stringify(pairInfo));

    // increase allowance of token A for the pair contract
    const increaseAllowanceAMsg = {
        "increase_allowance": {
            "spender": _contract,
            "amount": "10000000001",
        }
    };

    // execute the increase allowance message
    const increaseAllowanceAResponse = await client.execute(account.address, tokenAAddress, increaseAllowanceAMsg, "auto", "Increase allowance of token A for the pair contract");

    // increase allowance of token B for the pair contract
    const increaseAllowanceBMsg = {
        "increase_allowance": {
            "spender": _contract,
            "amount": "1000000001",
        }
    };

    // execute the increase allowance message
    const increaseAllowanceBResponse = await client.execute(account.address, tokenBAddress, increaseAllowanceBMsg, "auto", "Increase allowance of token B for the pair contract");

    const memo = "create pair";
    // define the take message using the address of deployer, uri of the nft and permitSignature
    const ExecuteProvideLiquidityMsg = {
        "provide_liquidity": {
            "assets": [
                {
                    "info": {
                        "token": {
                            "contract_addr": tokenAAddress,
                        }
                    },
                    "amount": "10000000000",
                },
                {
                    "info": {
                        "token": {
                            "contract_addr": tokenBAddress,
                        }
                    },
                    "amount": "1000000000",
                }
            ],
        }
    }

    console.log("ExecuteProvideLiquidityMsg: ", JSON.stringify(ExecuteProvideLiquidityMsg));

    // take a NFT
    const takeResponse = await client.execute(account.address, _contract, ExecuteProvideLiquidityMsg, "auto", memo);

    console.log(takeResponse);
}

const myArgs = process.argv.slice(2);
ProvideLiquidity(myArgs[0]);
