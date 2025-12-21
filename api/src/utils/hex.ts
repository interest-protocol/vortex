export const hexToDecimal = (hex: string): string => {
    const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
    const reversed = Buffer.from(cleanHex, 'hex').reverse();
    return BigInt('0x' + reversed.toString('hex')).toString();
};
