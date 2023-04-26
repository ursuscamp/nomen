# How To Get A Name

## Use the CLI

What you will need:

1. A Bitcoin UTXO (txid, vout)
2. A destination address
3. A keypair (any schnorr-compatible Bitcoin or hex-encoded Nostr keypair will work)
   * If you need one, use optional step below.

Steps to reserve a name:

1. `git clone https://github.com/ursuscamp/nomen.git`
2. `cd nomem`
3. `cargo build --release`
4. **OPTIONAL**: `target/release/nomen generate-keypair` to obtain a new keypair
5. `target/release/nomen name new --privkey $PRIVATE_KEY $NAME $TXID $VOUT $ADDRESS`
   * Use required data mentioned above in place of variables
6. Copy unsigned transaction, then sign and broadcast with your Bitcoin wallet.
7. `target/release/nomen name records --privkey $PRIVATE_KEY KEY1=value1 KEY2=value`
   * Create and broadcast new records to Nostr
   * Replace key/values with records of your choosing