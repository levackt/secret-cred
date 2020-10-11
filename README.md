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

### Usage
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

#### Register user
To register a user, you need their github username, as well as an address on the Secret Network.
```bash
 yarn run register-user --github_name=levackt --scrt_address=secret1yq04cf889ka4fmplypytq04mgkdj693tu4tn72
```

#### Allocate cred
By default allocation runs for the previous month, you can also specify a date range.

```bash
    yarn run allocate --start_date=[Start date] --end_date=[End date]
```

