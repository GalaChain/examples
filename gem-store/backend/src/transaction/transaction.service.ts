import { Injectable, HttpException, HttpStatus } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository } from 'typeorm';
import { Transaction } from '../entities/transaction.entity';
import { User } from '../entities/user.entity';
import { WalletService } from '../wallet/wallet.service';

@Injectable()
export class TransactionService {
  constructor(
    @InjectRepository(Transaction)
    private transactionRepository: Repository<Transaction>,
    @InjectRepository(User)
    private userRepository: Repository<User>,
    private walletService: WalletService,
  ) {}

  async processBurnTransaction(
    signedTransaction: any,
    gemAmount: number,
    galaAmount: number,
    walletAddress: string
  ): Promise<{ transactionId: string }> {
    try {
      // Validate wallet address
      const isValid = await this.walletService.validateWalletAddress(walletAddress);
      if (!isValid) {
        throw new HttpException('Invalid wallet address', HttpStatus.BAD_REQUEST);
      }

      // Submit the signed transaction to GalaChain
      const galaChainResponse = await this.submitToGalaChain(signedTransaction);
      
      if (!galaChainResponse.success) {
        throw new HttpException('GalaChain transaction failed', HttpStatus.BAD_REQUEST);
      }

      // Create transaction record
      const transaction = this.transactionRepository.create({
        userWalletAddress: walletAddress,
        galaAmount,
        gemAmount,
        transactionId: galaChainResponse.transactionId,
        status: 'completed'
      });

      await this.transactionRepository.save(transaction);

      // Add gems to user's balance
      await this.walletService.addGems(walletAddress, gemAmount);

      return { transactionId: galaChainResponse.transactionId };

    } catch (error) {
      // Create failed transaction record
      const transaction = this.transactionRepository.create({
        userWalletAddress: walletAddress,
        galaAmount,
        gemAmount,
        transactionId: `failed-${Date.now()}`,
        status: 'failed'
      });

      await this.transactionRepository.save(transaction);

      throw new HttpException(
        error.message || 'Transaction processing failed',
        error.status || HttpStatus.INTERNAL_SERVER_ERROR
      );
    }
  }

  async getTransactionHistory(walletAddress: string): Promise<{
    transactions: Transaction[];
    totalGems: number;
  }> {
    try {
      const isValid = await this.walletService.validateWalletAddress(walletAddress);
      if (!isValid) {
        throw new HttpException('Invalid wallet address', HttpStatus.BAD_REQUEST);
      }

      const transactions = await this.transactionRepository.find({
        where: { userWalletAddress: walletAddress },
        order: { createdAt: 'DESC' }
      });

      const user = await this.walletService.findOrCreateUser(walletAddress);

      return {
        transactions,
        totalGems: user.gemBalance
      };

    } catch (error) {
      throw new HttpException(
        error.message || 'Failed to fetch transaction history',
        error.status || HttpStatus.INTERNAL_SERVER_ERROR
      );
    }
  }

  private async submitToGalaChain(signedTransaction: any): Promise<{
    success: boolean;
    transactionId: string;
  }> {
    try {
      // Submit to GalaChain API
      const response = await fetch(
        `${process.env.TOKEN_GATEWAY_API || 'https://gateway-mainnet.galachain.com/api/asset/token-contract'}/BurnTokens`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(signedTransaction)
        }
      );

      if (!response.ok) {
        throw new Error(`GalaChain API error: ${response.status}`);
      }

      const result = await response.json();
      
      return {
        success: true,
        transactionId: result.transactionId || `tx-${Date.now()}`
      };

    } catch (error) {
      console.error('GalaChain submission error:', error);
      return {
        success: false,
        transactionId: `failed-${Date.now()}`
      };
    }
  }
}
