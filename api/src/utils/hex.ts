export const hexToDecimal = (hex: string): string => {
    const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
    const bytes = Buffer.from(cleanHex, 'hex');
    const reversed = Buffer.from(bytes).reverse();
    return BigInt('0x' + reversed.toString('hex')).toString();
};
