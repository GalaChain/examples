import { Transaction } from './transaction.entity';
export declare class User {
    id: number;
    walletAddress: string;
    gemBalance: number;
    createdAt: Date;
    updatedAt: Date;
    transactions: Transaction[];
}
