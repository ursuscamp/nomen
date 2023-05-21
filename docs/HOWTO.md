# How To Get A Name

What you will need:

1. A Bitcoin UTXO (txid, vout)
2. A destination address
3. A keypair (any schnorr-compatible Bitcoin or Nostr keypair will work)
   * If you need one, use optional step below.


## Using the Explorer

1. Visit https://nomen.directory
2. Click `New Name`.
3. Enter the txid of the Bitcoin UTXO and vout index of the transaction you will be using.
4. Enter the name you wish to reserve, the address of the output to send the Bitcoin, the pubkey that will be associate with the new name, and the fee to pay to the miners to mine the transaction.
  * __Note:__ If you have a NIP-07 compatible browser extension, you can click "Use NIP-07" and it will obtain the public key from your browser extension.
5. Click `Submit` and it will build a new, unsigned transaction for you. Copy the transaction to sign and broadcast it with your wallet.
6. After broadcasting the transaction, click `setup your records` to build a new nostr records event.
7. Enter the records you wish to include. Each record must be on its own line and look like this `KEY=value`.
8. Enter you public key again, or use your NIP-07 extension.
9. Click `Create Event` and you will be presented with an unsigned Nostr event.
10. Clicking `Sign and Broadcast` will use your NIP-07 extension to sign the event and broadcast it to relays.

## Use the CLI

Build the CLI first:

1. `git clone https://github.com/ursuscamp/nomen.git`
2. `cd nomem`
3. `cargo build --release`
4. Put `target/release/nomen` somewhere in your $PATH.

Using Nomen:

1. **OPTIONAL**: `target/release/nomen util generate-keypair` to obtain a new keypair
2. Open your Bitcoin wallet, and create a transaction that pays a Bitcoin UTXO back to you. Save that transaction (unsigned) as PSBT. Slightly overestimate your fees to account for an extra output we will add.
3. `target/release/nomen name new --privkey <PRIVATE KEY> --broadcast --output out.psbt <NAME> <PSBT>`
   * Replace PRIVATE_KEY with the hex-encoded secp256k1 private key or nsec Nostr key.
   * Replace NAME with the desired name you wish you register.
   * Replace PSBT with the path of the PSBT you created.
4. Open `out.psbt` in your Bitcoin wallet. It should now include an extra output. Sign it with your Bitcoin wallet and broadcast it.
5. `target/release/nomen name records --privkey $PRIVATE_KEY KEY1=value1 KEY2=value`
   * Create and broadcast new records to Nostr.
   * Replace key/values with records of your choosing.