import { WalletService } from './wallet.service';
export declare class WalletController {
    private readonly walletService;
    constructor(walletService: WalletService);
    getBalance(address: string): Promise<{
        gemBalance: number;
    }>;
    connectWallet(body: {
        walletAddress: string;
    }): Promise<{
        success: boolean;
        message: string;
        walletAddress: string;
    }>;
}
