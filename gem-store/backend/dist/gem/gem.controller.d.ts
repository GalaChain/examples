import { GemService } from './gem.service';
export declare class GemController {
    private readonly gemService;
    constructor(gemService: GemService);
    getGemPackages(): {
        success: boolean;
        packages: import("./gem.service").GemPackage[];
        exchangeRate: number;
    };
    getExchangeRate(): {
        success: boolean;
        exchangeRate: number;
        description: string;
    };
}
