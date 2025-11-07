pragma circom 2.0.0;

include "../node_modules/circomlib/circuits/poseidon.circom";

// Since we don't use signatures, the keypair can be based on a simple hash
template Keypair() {
    signal input privateKey;

    component hasher = Poseidon(1);
    hasher.inputs[0] <== privateKey;
    signal output publicKey <== hasher.out;
}

template Signature() {
    signal input privateKey;
    signal input commitment;
    signal input merklePath;

    component hasher = Poseidon(3);
    hasher.inputs[0] <== privateKey;
    hasher.inputs[1] <== commitment;
    hasher.inputs[2] <== merklePath;
    signal output out <== hasher.out;
}

component main = Keypair();