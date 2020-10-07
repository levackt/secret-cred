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
  console.log("yarn run allocate --start_date=[Start date] --end_date=[End date]")
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
  console.log(await client.getAccount())

  const config = await client.queryContractSmart(contractAddress, { config: { } })
  console.log(`contract config=${JSON.stringify(config)}`)

  ledger = await loadLedger("SecretFoundation/SecretPoints")

  G = sc.ledger.grain

  let totalGrain = "0"
  for (const {balance} of ledger.accounts()) {
      totalGrain = G.add(totalGrain, balance)
  }

  G.format(totalGrain)


  const logs = ledger.eventLog();


  const startDateInput = argv.start_date;
  const endDateInput = argv.end_date;

  var today = new Date();
  if (startDateInput) {
    startDate = new Date(startDateInput)
  } else {
    startDate = new Date(today.getFullYear(), today.getMonth() - 1, 1);
  }

  if (endDateInput) {
    endDate = new Date(endDateInput)
  } else {
    endDate = new Date(today.getFullYear(), today.getMonth(), 0);
  }

  if (endDate < startDate) {
    throw "END_DATE cannot be before START_DATE"
  }

  console.log(`Distributing from start=${startDate} to end=${endDate}`)

  distributions = logs.filter(x => x.ledgerTimestamp < endDate && x.ledgerTimestamp > startDate)
      .filter(x => x.action.type === "DISTRIBUTE_GRAIN");

  const allocations = [];
  for (const { action } of distributions) {
      const distribution = action.distribution;
      for (const alloc of distribution.allocations) {
          allocations.push(alloc);
      }
  }
  const policies = {
    "IMMEDIATE": "Immediate",
    "BALANCED": "Balanced"
  }

  for (const allocation of allocations) {

    console.log(`Distributing policyType=${allocation.policy.policyType}, budget=${allocation.policy.budget}`);
    for (const receipt of allocation.receipts) {

      // allocate if user is registered
      let result = await client.queryContractSmart(contractAddress, { is_cred_registered: { cred_id: receipt.id } });

      if (result.registered) {
        let allocResult = await client.queryContractSmart(contractAddress, { is_allocated: { cred_id: receipt.id, allocation_id: allocation.id} });

        if (allocResult.allocated) {
          console.log(`Already allocated=${allocation.id} to id=${receipt.id}`)
          continue
        }

        console.log(`allocating ${receipt.amount} to ${receipt.id}`);

        const allocateMsg = {
          allocation_id: allocation.id,
          cred_id: receipt.id,
          amount: receipt.amount,
          policy_type: policies[allocation.policy.policyType],
        }
        console.log(`allocation message=${JSON.stringify(allocateMsg)}`)

        let result = await client.execute(
          contractAddress, {allocate: allocateMsg}
        );

        console.info(`Allocate result: ${JSON.stringify(result)}`);
      }
    }
  }
}

main().then(
  () => {
    console.info("Done allocating");
    process.exit(0);
  },
  error => {
    console.error(error);
    process.exit(1);
  },
);
