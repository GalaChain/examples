import { Repository } from 'typeorm';
import { User } from '../entities/user.entity';
export declare class WalletService {
    private userRepository;
    constructor(userRepository: Repository<User>);
    findOrCreateUser(walletAddress: string): Promise<User>;
    getUserBalance(walletAddress: string): Promise<{
        gemBalance: number;
    }>;
    addGems(walletAddress: string, gemAmount: number): Promise<void>;
    validateWalletAddress(walletAddress: string): Promise<boolean>;
}
