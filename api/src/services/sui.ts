import { GasStationClient, createSuiClient, buildGaslessTransaction } from '@shinami/clients/sui';
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import type { Transaction } from '@mysten/sui/transactions';
import { fromBase64 } from '@mysten/sui/utils';
import { env } from '@/config/env.ts';
import { logger } from '@/utils/logger.ts';

export const nodeClient = createSuiClient(env.SHINAMI_RPC_KEY);
export const gasClient = new GasStationClient(env.SHINAMI_RPC_KEY);
export const keypair = Ed25519Keypair.fromSecretKey(env.SUI_PRIVATE_KEY);

export async function sponsorAndExecuteTransaction(tx: Transaction) {
    const sender = keypair.toSuiAddress();

    const gaslessTx = await buildGaslessTransaction(tx, { sui: nodeClient, sender });

    const { txBytes, signature: sponsorSignature } = await gasClient.sponsorTransaction(gaslessTx);

    const txBytesArray = typeof txBytes === 'string' ? fromBase64(txBytes) : txBytes;
    const { signature: senderSignature } = await keypair.signTransaction(txBytesArray);

    const result = await nodeClient.executeTransactionBlock({
        transactionBlock: txBytes,
        signature: [senderSignature, sponsorSignature],
        options: { showEffects: true, showEvents: true, showObjectChanges: true },
    });

    if (result.effects?.status.status !== 'success') {
        logger.error({ result }, 'Transaction failed');
        throw new Error(`Transaction failed: ${result.effects?.status.error ?? 'unknown error'}`);
    }

    logger.info({ digest: result.digest }, 'Transaction executed');
    return result;
}
