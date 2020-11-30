# Secret Cred contract

Keeps track of allocations in [Secret Points](https://secretfoundation.github.io/SecretPoints)

Users should be registered by an admin of the contract, thereafter an oracle
adds any allocations due to the user periodically, or as part of the Secret Points github workflow.

### Build contract

```bash
    git clone https://github.com/levackt/secret-cred

    make
```

### Upload and instantiate the contract

```bash
    secretcli tx compute store contract.wasm.gz --from <key alias> --source "https://github.com/levackt/secret-cred" -y --gas 20000000
    CODE_ID=<result of upload TX>
    INIT="{\"denom\": \"uscrt\"}"
    secretcli tx compute instantiate $CODE_ID "$INIT" --from <key alias> --label "something unique" -y
```

### Contract Admin
The account used to instantiate the contract is the owner, it has permission to register users and allocate cred.

#### install the client
```bash
    cd client
    yarn
```

#### Copy or edit .env.defaults
```bash
    cp .env.defaults .env
```

#### User registration

To register a user, you need their github username, as well as an address on the Secret Network, eg
```bash
yarn run register-user --github_name=levackt --scrt_address=secret12345...
```

#### Allocate cred
By default allocation runs for the previous month, you can also specify a date range.

```bash
    yarn run allocate --start_date=[Start date] --end_date=[End date]
```

### As a contributor

To register as a contributor, submit your secret address in a GitHub issue of this repo.

A contract admin will register your secret account for any future allocations.
You can then view your DevToken balance with secretcli

```bash
# query the contract for DevToken info
CONTRACT=<secret creds contract address / label>
secretcli query compute query $CONTRACT "{\"config\": {}}" | jq

TOKEN_CONTRACT=<token_contract.address output above>

# create viewing key
secretcli tx snip20 create-viewing-key $TOKEN_CONTRACT --from <your account alias> -y

HASH=<output>

# query viewing key with hash output above
secretcli q compute tx <HASH>

VIEWING_KEY=<api key...>

# query balance
secretcli q snip20 balance $TOKEN_CONTRACT <your account address> $VIEWING_KEY
```