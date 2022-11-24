const chainConfig = require('./config/chain').defaultChain;

const fs = require('fs');

const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet } = require('@cosmjs/proto-signing');
// const { calculateFee, GasPrice } = require('@cosmjs/stargate');

async function instantiate(_codeID) {
    const deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        chainConfig.deployer_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );
    const client = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, deployerWallet, {
        gasPrice: "0.025uaura",
    });

    const account = (await deployerWallet.getAccounts())[0];

    const defaultFee = { amount: [{amount: "300000", denom: chainConfig.denom,},], gas: "auto",};

    const codeId = _codeID;
    //Define the instantiate message
    const instantiateMsg = {"asset_infos": [
                                {
                                    "token": {
                                        "contract_addr": "aura1smcy8kged97n493cgh6jzrda0hahge4j6qrg0hx2j5tgwy2259hshpxzyv"},
                                },
                                {
                                    "token": {
                                        "contract_addr": "aura19xm0cnz5j6m8q6jdt2hyldld8wxpd33cym8s6l99kfnfjh6gcstsk4sjua"},
                                }
                            ],
                            "token_code_id": 713,
                            "asset_decimals": [6, 6]
                        };


    //Instantiate the contract
    const instantiateResponse = await client.instantiate(account.address, Number(_codeID), instantiateMsg, "Instantiate contract", "auto");
    console.log(instantiateResponse);

    // print out the address of the newly created contract
    const contracts = await client.getContracts(_codeID);
    console.log(contracts);
}

const myArgs = process.argv.slice(2);
instantiate(myArgs[0]);
