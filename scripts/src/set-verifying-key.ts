import * as circomlibjs from 'circomlibjs';

async function makePoseidon() {
  const poseidon = await circomlibjs.buildPoseidon();
  const Fp = poseidon.F as any;

  const fieldMod = BigInt(Fp.p.toString());

  const stringToField = (s: string) => {
    const bytes = new TextEncoder().encode(s);
    let acc = 0n;
    for (const b of bytes) acc = (acc << 8n) | BigInt(b);
    return acc % fieldMod;
  };

  // Normalize poseidon result to bigint across implementations/typings
  const toBigint = (x: any): bigint =>
    typeof x === 'bigint'
      ? x
      : Fp.toObjectbut
        ? (Fp.toObject(x) as bigint)
        : BigInt(Fp.toString(x));

  const poseidon1 = (a: bigint) => toBigint(poseidon([a]));
  const poseidon2 = (a: bigint, b: bigint) => toBigint(poseidon([a, b]));

  const toHex32 = (x: bigint) => '0x' + x.toString(16).padStart(64, '0');

  return { poseidon1, poseidon2, stringToField, toHex32, fieldMod };
}

export async function buildZerosAndFilledSubtrees(treeLevels: number) {
  if (treeLevels < 1) throw new Error('treeLevels must be >= 1');

  const { poseidon1, poseidon2, stringToField, toHex32, fieldMod } =
    await makePoseidon();

  // ZERO_VALUE := Poseidon("vortex")
  const ZERO_VALUE = poseidon1(stringToField('vortex'));

  const zeros: bigint[] = [];
  const filledSubtrees: bigint[] = [];

  let currentZero = ZERO_VALUE;
  zeros.push(currentZero);
  filledSubtrees.push(currentZero);

  for (let i = 1; i < treeLevels; i++) {
    currentZero = poseidon2(currentZero, currentZero); // hashLeftRight
    zeros.push(currentZero);
    filledSubtrees.push(currentZero);
  }

  return {
    ZERO_VALUE,
    zeros,
    filledSubtrees,
    zerosHex: zeros.map(toHex32),
    filledSubtreesHex: filledSubtrees.map(toHex32),
    fieldMod,
  };
}

(async () => {
  const {
    ZERO_VALUE,
    zeros,
    filledSubtrees,
    zerosHex,
    filledSubtreesHex,
    fieldMod,
  } = await buildZerosAndFilledSubtrees(26);

  console.log({
    ZERO_VALUE,
    zeros,
    filledSubtrees,
    zerosHex,
    filledSubtreesHex,
    fieldMod,
  });
})();
