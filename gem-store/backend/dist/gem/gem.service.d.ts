export interface GemPackage {
    id: number;
    gems: number;
    gala: number;
    description?: string;
}
export declare class GemService {
    private readonly exchangeRate;
    constructor();
    getGemPackages(): GemPackage[];
    getExchangeRate(): number;
    calculateGemsFromGala(galaAmount: number): number;
    calculateGalaFromGems(gemAmount: number): number;
    validateGemPackage(packageId: number, galaAmount: number, gemAmount: number): boolean;
}
