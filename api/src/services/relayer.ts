import type { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';

export type RelayerService = {
    getAddress: () => string;
};

export const createRelayerService = (keypair: Ed25519Keypair): RelayerService => ({
    getAddress: () => keypair.toSuiAddress(),
});
