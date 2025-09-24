import { TransactionService } from './transaction.service';
export declare class BurnTransactionDto {
    signedTransaction: any;
    gemAmount: number;
    galaAmount: number;
    walletAddress: string;
}
export declare class TransactionController {
    private readonly transactionService;
    constructor(transactionService: TransactionService);
    burnTokens(body: BurnTransactionDto): Promise<{
        transactionId: string;
        success: boolean;
        message: string;
    }>;
    getTransactionHistory(address: string): Promise<{
        transactions: import("../entities/transaction.entity").Transaction[];
        totalGems: number;
        success: boolean;
    }>;
}
