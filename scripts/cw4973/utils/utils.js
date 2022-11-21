const chainConfig = require('../config/chain').defaultChain;

const fs = require('fs');

const { SigningCosmWasmClient } = require('@cosmjs/cosmwasm-stargate');
const { DirectSecp256k1HdWallet } = require('@cosmjs/proto-signing');
const { GasPrice } = require('@cosmjs/stargate');
const amino = require('@cosmjs/amino');
const {AminoMsg, StdFee} = require('@cosmjs/amino');

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

    return messageToSign;
}

async function getPermitSignatureAmino(messageToSign, signer_mnemonic) {
    const deployerWallet = await amino.Secp256k1HdWallet.fromMnemonic(
        signer_mnemonic,
        {
            prefix: chainConfig.prefix
        }
    );

    // const adminAccount = deployerWallet.getAccounts()[0];
    const adminAccount = (await deployerWallet.getAccounts())[0];

    // sign message
    const signedDoc = await deployerWallet.signAmino(adminAccount.address, messageToSign);

    // pubkey must be compressed in base64
    let permitSignature = {
        "hrp": "aura",
        "pub_key": Buffer.from(adminAccount.pubkey).toString('base64'),
        "signature": signedDoc.signature.signature,
    }

    // console.log("signature: ", signatureBase64.signature);

    return permitSignature;
}

module.exports = { createMessageToSign, getPermitSignatureAmino };