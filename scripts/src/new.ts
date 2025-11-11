import { suiClient, keypair } from './utils/keypair';
import { VORTEX_PACKAGE_ID, INIT_CAP } from './utils/constants';
import { Transaction } from '@mysten/sui/transactions';
import { SUI_FRAMEWORK_ADDRESS } from '@mysten/sui/utils';
import { fromHex } from '@mysten/sui/utils';

const VERIFYING_KEY =
  'e57314eb0d2d4acef7a0b56306a4ac1dc99b9a1dc15a34dc549a052171bd981b1e6533af92c383be56b39f0520f0c3f5713f404cc505f4887a88224fbf49562da2f948129e080d367595fcc2f8a6beee2c088f4e77fdfdb9edaaeb4b407d381617e42ded924236cbb7a82ba74b9ae3198aeef633e290a9931ed396a6e6109d1abb8615f6e3dfff68ed2be32ce4035b12e85792795c2f87bc0d2999cf9d2a81224795e7b72c7be59554ce1157ac2db49d2757e81bae97505a0d81ccbe88beb12f3e2e2c16d42cc821a62663a6cd59dcfa621b9aa7472df54a6f13500c1dea611e0800000000000000709473bc071b904d0d5cbe2970a8cea5d455e14bdea8de99e5d8e8a8df959c11f3e9c34758fdb47f005e28a0e5dd98188a255389fde55843716a6e3147c1e01d881b4ddd094049273d579f4de2bf43b55ad2273e66fb71705222b54545ce4125ffba28eada12613ba1ed50292d235af3d2f891dc194f16bf5b1f31751edeb0a7541547b2a4c479930bd40ee947738a49ac0e2fc4f5c8b5a433ca7ace2e144390911e68ec08cc737ea0718e59b727248bafb31315bd6fc95e80ac7dcc029cd8ae1081f54be4f2e745e2d6f34e706a35eb4f067a657694537d30cef0467cd371aab4bf9bda6f95c672b1dd8133e6656b98a29b1b3971faf449e9250afacee63987';

// Main execution
(async () => {
  const tx = new Transaction();

  tx.setSender(keypair.toSuiAddress());

  const curve = tx.moveCall({
    target: `${SUI_FRAMEWORK_ADDRESS}::groth16::bn254`,
  });

  const preparedKey = tx.moveCall({
    target: `${SUI_FRAMEWORK_ADDRESS}::groth16::prepare_verifying_key`,
    arguments: [curve, tx.pure.vector('u8', fromHex(VERIFYING_KEY))],
  });

  const vortex = tx.moveCall({
    target: `${VORTEX_PACKAGE_ID}::vortex::new`,
    arguments: [tx.object(INIT_CAP), preparedKey],
  });

  tx.moveCall({
    target: `${VORTEX_PACKAGE_ID}::vortex::share`,
    arguments: [vortex],
  });

  const result = await keypair.signAndExecuteTransaction({
    transaction: tx,
    client: suiClient,
  });

  console.log(result);

  process.exit(0);
})();
