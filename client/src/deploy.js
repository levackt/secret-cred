#!/usr/bin/env node

const {argv} = require('yargs');
require('dotenv-defaults').config();

/* eslint-disable @typescript-eslint/camelcase */
const { EnigmaUtils, Secp256k1Pen, SigningCosmWasmClient, pubkeyToAddress, encodeSecp256k1Pubkey } = require("secretjs");
const fs = require("fs");

//const httpUrl = "http://localhost:1317";
const httpUrl = "https://bootstrap.secrettestnet.io";

MNEMONIC = process.env.MNEMONIC;

const customFees = {
  upload: {
    amount: [{ amount: "2000000", denom: "uscrt" }],
    gas: "2000000",
  },
  init: {
    amount: [{ amount: "500000", denom: "uscrt" }],
    gas: "500000",
  },
  exec: {
    amount: [{ amount: "500000", denom: "uscrt" }],
    gas: "500000",
  },
  send: {
    amount: [{ amount: "80000", denom: "uscrt" }],
    gas: "80000",
  },
}

async function main() {
  const signingPen = await Secp256k1Pen.fromMnemonic(MNEMONIC);
  const myWalletAddress = pubkeyToAddress(
    encodeSecp256k1Pubkey(signingPen.pubkey),
    "secret"
  );
  const txEncryptionSeed = EnigmaUtils.GenerateNewSeed();
  const client = new SigningCosmWasmClient(
    httpUrl,
    myWalletAddress,
    (signBytes) => signingPen.sign(signBytes),
    txEncryptionSeed, customFees
  );
  console.log(myWalletAddress)


  // upload token contract
  let wasm = fs.readFileSync(__dirname + "/../contracts/token/contract.wasm");
  let uploadReceipt = await client.upload(wasm, {})
  console.info(`Token Upload succeeded. Receipt: ${JSON.stringify(uploadReceipt)}`);
  let codeId = uploadReceipt.codeId;
  //init token
  const tokenInit = {
    "name":"secretdev",
    "symbol":"SDEV",
    "decimals": 18,
    "config": {
      "public_total_supply": true
    },
    "prng_seed": Buffer.from("coffee and cupcakes").toString('base64')
  }
  const codes = await client.getCodes();
  let label = "token" + (codes.length + 1);
  const { contractAddress } = await client.instantiate(codeId, tokenInit, label);
  console.log(`tokenContractAddress=${contractAddress}`)


  wasm = fs.readFileSync(__dirname + "/../../contract.wasm");
  uploadReceipt = await client.upload(wasm, {})
  console.info(`Upload succeeded. Receipt: ${JSON.stringify(uploadReceipt)}`);

  // init secret cred
  label = "secret-cred-" + (codes.length + 2);
  const hashStr = String(fs.readFileSync(__dirname + "/../contracts/token/hash.txt"));
  console.log(`hashStr=${hashStr}`)

  const initMsg = {"token_contract": {
    "address": contractAddress,
    "code_hash": hashStr.substring(0, hashStr.indexOf(' ')),
  }}

  const secretCredInit = await client.instantiate(uploadReceipt.codeId, initMsg, label);
  console.info(`Contract instantiated at ${secretCredInit.contractAddress}`);

//  allow secret cred contract to mint
  let result = await client.execute(contractAddress, { add_minters: { minters: [secretCredInit.contractAddress] } });
  console.log('added minters')
  console.log(result)

}

main().then(
  () => {
    console.info("Secret Cred contract deployed.");
    process.exit(0);
  },
  error => {
    console.error(error);
    process.exit(1);
  },
);


