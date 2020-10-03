#!/usr/bin/env node

const {argv} = require('yargs')

/* eslint-disable @typescript-eslint/camelcase */
const { Encoding } = require("@iov/encoding");
const { coin } = require("@cosmjs/sdk38");

/* eslint-disable @typescript-eslint/camelcase */
const {
  BroadcastMode, EnigmaUtils, Secp256k1Pen, SigningCosmWasmClient, pubkeyToAddress, encodeSecp256k1Pubkey, makeSignBytes
} = require("secretjs");

const fs = require("fs");

const httpUrl = "https://bootstrap.secrettestnet.io";
require('dotenv-defaults').config();
const fetch = require('node-fetch');
const sc = require("sourcecred").default

MNEMONIC = process.env.MNEMONIC;
contractAddress = process.env.CONTRACT;

var startTs = process.env.START_TS;
var endTs = process.env.END_TS;

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
    amount: [{ amount: "250000", denom: "uscrt" }],
    gas: "250000",
  },
  send: {
    amount: [{ amount: "80000", denom: "uscrt" }],
    gas: "80000",
  },
}

async function loadCredView(repo) {
    const credResultFile = `https://raw.githubusercontent.com/${repo}/gh-pages/output/credResult.json`;
    const credResultRaw = await (await fetch(credResultFile)).json()
    const credResult = sc.analysis.credResult.fromJSON(credResultRaw)
    const credView = new sc.analysis.credView.CredView(credResult)
    return credView;
}

async function loadLedger(repo) {
    const ledgerFile = `https://raw.githubusercontent.com/${repo}/gh-pages/data/ledger.json`;
    const ledgerRaw = await (await fetch(ledgerFile)).text();
    return sc.ledger.ledger.Ledger.parse(ledgerRaw);
}

function usage() {
  console.log("yarn run register-user --cred_id=[Source Cred ID] --scrt_address=[Secret Network address]")
}

async function main() {

  const cred_id = argv.cred_id;
  const scrt_address = argv.scrt_address;

  if (!cred_id) {
    throw "cred_id expected"
  }
  if (!scrt_address) {
    throw "scrt_address expected"
  }

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

  const result = await client.queryContractSmart(contractAddress, { is_cred_registered: { cred_id } });

  if (result.registered) {
    console.log(`cred_id=${cred_id} is already registered`)
    return;
  } else {
    const registerMsg = {
      cred_id,
      scrt_address
    }
    console.log(`register message=${JSON.stringify(registerMsg)}`)

    let result = await client.execute(
      contractAddress, {register_user: registerMsg}
    );

    console.info(`Register result: ${JSON.stringify(result)}`);
  }
}

main().then(
  () => {
    console.info("Done register");
    process.exit(0);
  },
  error => {
    console.error(error);
    process.exit(1);
  },
);
