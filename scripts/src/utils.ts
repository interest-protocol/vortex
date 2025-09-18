import { getFullnodeUrl, SuiClient } from '@mysten/sui/client';
import { Transaction } from '@mysten/sui/transactions';
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import invariant from 'tiny-invariant';

export const suiClient = new SuiClient({
  url: getFullnodeUrl('devnet'),
});

invariant(process.env.KEY, 'Private key missing');

export const keypair = Ed25519Keypair.fromSecretKey(
  Uint8Array.from(Buffer.from(process.env.KEY, 'base64')).slice(1)
);

export const executeTx = async (tx: Transaction) => {
  const result = await suiClient.signAndExecuteTransaction({
    signer: keypair,
    transaction: tx,
    options: { showEffects: true },
  });

  // return if the tx hasn't succeed
  if (result.effects?.status?.status !== 'success') {
    console.error('tx-failed', result.errors);
    return;
  }

  console.log('tx-success', result.digest);

  if (result.effects.created) {
    console.log(result.effects.created);
  }
};
