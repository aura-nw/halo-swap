const chainConfig = require('./config/chain').defaultChain;

const fs = require('fs');

const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet } = require('@cosmjs/proto-signing');
const { GasPrice } = require('@cosmjs/stargate');
const amino = require('@cosmjs/amino');
const {AminoMsg, StdFee} = require('@cosmjs/amino');
const { toUtf8 } = require("@cosmjs/encoding");

function createMessageToSign(chainID, active, passive, uri) {
    const AGREEMENT = 'Agreement(address active,address passive,string tokenURI)';

    // create message to sign based on concating AGREEMENT, signer, receiver, and uri
    const message = AGREEMENT + active + passive + uri;

    const mess = {
        type: "sign/MsgSignData",
        value: {
            signer: String(passive),
            data: String(message)
        }
    };

    const fee = {
        gas: "0",
        amount: []
    };

    const messageToSign = amino.makeSignDoc(mess, fee, String(chainID), "",  0, 0);
    console.log("amino.serializeSignDoc(messageToSign): ", toUtf8(amino.sortedJsonStringify(messageToSign)));

    return messageToSign;
}

async function getPermitSignatureAmino(messageToSign, mnemonic) {
    const signerWallet = await amino.Secp256k1HdWallet.fromMnemonic(
        mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );

    // const adminAccount = deployerWallet.getAccounts()[0];
    const signerAccount = (await signerWallet.getAccounts())[0];

    // sign message
    const signedDoc = await signerWallet.signAmino(signerAccount.address, messageToSign);
    console.log("signedDoc: ", signedDoc);

    const decodedSignature = amino.decodeSignature(signedDoc.signature);
    console.log(decodedSignature);

    // pubkey must be compressed in base64
    let permitSignature = {
        "hrp": "aura",
        "pub_key": Buffer.from(signerAccount.pubkey).toString('base64'),
        "signature": signedDoc.signature.signature,
    }

    return permitSignature;
}

async function give(_contract, _uri) {
    const deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        chainConfig.deployer_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );
    // get deployer account
    const deployerAccount = (await deployerWallet.getAccounts())[0];

    // gas price
    const gasPrice = GasPrice.fromString(`0.025${chainConfig.denom}`);

    // connect tester wallet to chain
    const deployerClient = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, deployerWallet, {gasPrice});


    const testerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        chainConfig.tester_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );
    // get tester account
    const testerAccount = (await testerWallet.getAccounts())[0];

    // create message to sign
    const messageToSign = createMessageToSign(chainConfig.chainId, deployerAccount.address, testerAccount.address, _uri);
    console.log("messageToSign: ", messageToSign);

    // sign message
    const permitSignature = await getPermitSignatureAmino(messageToSign, chainConfig.tester_mnemonic);
    console.log("permitSignature: ", permitSignature);

    const memo = "give nft";
    // define the take message using the address of deployer, uri of the nft and permitSignature
    const ExecuteTakeMsg = {
        "give": {
            "to": testerAccount.address,
            "uri": _uri,
            "signature": permitSignature,
        }
    }

    console.log("ExecuteTakeMsg: ", ExecuteTakeMsg);

    // take a NFT
    const takeResponse = await deployerClient.execute(deployerAccount.address, _contract, ExecuteTakeMsg, "auto", memo);

    console.log(takeResponse);
}

async function take(_contract, _uri) {
    const deployerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        chainConfig.deployer_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );

    const testerWallet = await DirectSecp256k1HdWallet.fromMnemonic(
        chainConfig.tester_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );

    // get deployer account
    const deployerAccount = (await deployerWallet.getAccounts())[0];

    // gas price
    const gasPrice = GasPrice.fromString(`0.025${chainConfig.denom}`);

    // connect tester wallet to chain
    const testerClient = await SigningCosmWasmClient.connectWithSigner(chainConfig.rpcEndpoint, testerWallet, {gasPrice});

    // get tester account
    const testerAccount = (await testerWallet.getAccounts())[0];

    // create message to sign
    const messageToSign = createMessageToSign(chainConfig.chainId, testerAccount.address, deployerAccount.address, _uri);
    console.log("messageToSign: ", messageToSign);

    // sign message
    const permitSignature = await getPermitSignatureAmino(messageToSign, chainConfig.deployer_mnemonic);
    console.log("permitSignature: ", permitSignature);

    const memo = "take nft";
    // define the take message using the address of deployer, uri of the nft and permitSignature
    const ExecuteTakeMsg = {
        "take": {
            "from": deployerAccount.address,
            "uri": _uri,
            "signature": permitSignature,
        }
    }

    console.log("ExecuteTakeMsg: ", ExecuteTakeMsg);

    // take a NFT
    const takeResponse = await testerClient.execute(testerAccount.address, _contract, ExecuteTakeMsg, "auto", memo);

    console.log(takeResponse);
}

const myArgs = process.argv.slice(2);
take(myArgs[0], myArgs[1]);
