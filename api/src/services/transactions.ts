import type { SuiClient } from '@mysten/sui/client';
import type { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import { Transaction } from '@mysten/sui/transactions';
import { fromBase64 } from '@mysten/sui/utils';
import { buildGaslessTransaction, type GasStationClient } from '@shinami/clients/sui';
import { logger } from '@/utils/logger.ts';
import { fromHex } from '@mysten/sui/utils';

export type TransactionsService = {
    execute: (txBytes: string) => Promise<string>;
};

export const createTransactionsService = (
    nodeClient: SuiClient,
    gasClient: GasStationClient,
    keypair: Ed25519Keypair
): TransactionsService => ({
    execute: async (txBytes) => {
        const sender = keypair.toSuiAddress();
        const rebuiltTransaction = Transaction.from(fromHex(txBytes));

        rebuiltTransaction.setSender(sender);

        const gaslessTx = await buildGaslessTransaction(rebuiltTransaction, {
            sui: nodeClient,
            sender,
        });
        const { txBytes: sponsoredTxBytes, signature: sponsorSignature } =
            await gasClient.sponsorTransaction(gaslessTx);

        const signedTxBytes =
            typeof sponsoredTxBytes === 'string' ? fromBase64(sponsoredTxBytes) : sponsoredTxBytes;
        const { signature: senderSignature } = await keypair.signTransaction(signedTxBytes);

        const result = await nodeClient.executeTransactionBlock({
            transactionBlock: sponsoredTxBytes,
            signature: [senderSignature, sponsorSignature],
            options: { showEffects: true },
        });

        if (result.effects?.status.status !== 'success') {
            logger.error({ result }, 'Transaction failed');
            throw new Error('Transaction failed');
        }

        logger.info({ digest: result.digest }, 'Sponsored transaction executed');
        return result.digest;
    },
});
