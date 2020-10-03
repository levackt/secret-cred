#!/usr/bin/env node

const fetch = require('node-fetch');
const sc = require("sourcecred").default

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

async function main() {

    START_TS = +new Date("2020-08-25")
    END_TS = +new Date("2020-09-25")

    ledger = await loadLedger("SecretFoundation/SecretPoints")

    G = sc.ledger.grain

    let totalGrain = "0"
    for (const {balance} of ledger.accounts()) {
        totalGrain = G.add(totalGrain, balance)
    }

    G.format(totalGrain)

    const logs = ledger.eventLog();
    distributions = logs.filter(x => x.ledgerTimestamp < END_TS && x.ledgerTimestamp > START_TS)
        .filter(x => x.action.type === "DISTRIBUTE_GRAIN");

    const allocations = [];
    for (const { action } of distributions) {
        const distribution = action.distribution;
        for (const alloc of distribution.allocations) {
            allocations.push(alloc);
        }
    }

    console.log(`${JSON.stringify(allocations)}`)

    const allocationByType = { SPECIAL: "0", BALANCED: 0, IMMEDIATE: 0 };
    for (const { policy } of allocations) {
        allocationByType[policy.policyType] = G.add(allocationByType[policy.policyType], policy.budget);
    }

    G.format(allocationByType["SPECIAL"])

    G.format(allocationByType["IMMEDIATE"])

    console.log(`immediate: ${G.format(allocationByType["IMMEDIATE"])}`);
    console.log(`balanced: ${G.format(allocationByType["BALANCED"])}`);

    specialAllocs = allocations.filter(x => x.policy.policyType === "SPECIAL")

    specialAllocs.map(x => ({
        name: ledger.account(x.policy.recipient).identity.name,
        amt: G.format(x.policy.budget),
        memo: x.policy.memo
    }))
}

main().then(
    () => {
      console.info("Done minting secret cred.");
      process.exit(0);
    },
    error => {
      console.error(error);
      process.exit(1);
    },
  );
