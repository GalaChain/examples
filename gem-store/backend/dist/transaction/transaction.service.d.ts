import { Repository } from 'typeorm';
import { Transaction } from '../entities/transaction.entity';
import { User } from '../entities/user.entity';
import { WalletService } from '../wallet/wallet.service';
export declare class TransactionService {
    private transactionRepository;
    private userRepository;
    private walletService;
    constructor(transactionRepository: Repository<Transaction>, userRepository: Repository<User>, walletService: WalletService);
    processBurnTransaction(signedTransaction: any, gemAmount: number, galaAmount: number, walletAddress: string): Promise<{
        transactionId: string;
    }>;
    getTransactionHistory(walletAddress: string): Promise<{
        transactions: Transaction[];
        totalGems: number;
    }>;
    private submitToGalaChain;
}
