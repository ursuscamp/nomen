# How To Get A Name

What you will need:

1. A Bitcoin UTXO
2. A wallet to sign a PSBT
3. A keypair (any schnorr-compatible Bitcoin or Nostr keypair will work)
   * If you need one, use optional step below.

How it works:

You must publish your name on the Bitcoin blockchain, so you need a transaction. The easiest way is to use the Explorer. With the Explorer, you can specify a Bitcoin UTXO and a change address.

It will construct a PSBT which you can sign and broadcast with your wallet. Once it confirms, the indexer will find your name.

## Using the Explorer

1. Visit https://nomenexplorer.com
2. Click `New Name`.
3. Enter the `Txid` and `Vout` of your UTXO.
4. Enter a `Change Address` which will point back to your Bitcoin wallet.
5. Set your fee amount, in `sats/vb`.
6. Enter the name you wish to reserve and the pubkey of the owner.
  * __Note:__ If you have a NIP-07 compatible browser extension, you can click "Use NIP-07" and it will obtain the public key from your browser extension.
7. Click `Submit` and it will build a new, unsigned transaction for you. Copy the transaction to sign and broadcast it with your wallet.
8. After broadcasting the transaction, click `setup your records` to build a new nostr records event.
9. Enter the records you wish to include. Each record must be on its own line and look like this `KEY=value`.
10. Enter you public key again, or use your NIP-07 extension.
11. Click `Create Event` and you will be presented with an unsigned Nostr event.
12. Clicking `Sign and Broadcast` will use your NIP-07 extension to sign the event and broadcast it to relays.