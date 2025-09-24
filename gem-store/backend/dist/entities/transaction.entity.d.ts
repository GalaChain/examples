import { User } from './user.entity';
export declare class Transaction {
    id: number;
    userWalletAddress: string;
    galaAmount: number;
    gemAmount: number;
    transactionId: string;
    status: string;
    createdAt: Date;
    user: User;
}
