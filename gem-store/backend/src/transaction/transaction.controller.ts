import { Controller, Post, Get, Body, Param, HttpException, HttpStatus } from '@nestjs/common';
import { IsNotEmpty, IsNumber, IsString, IsObject } from 'class-validator';
import { TransactionService } from './transaction.service';

export class BurnTransactionDto {
  @IsObject()
  @IsNotEmpty()
  signedTransaction: any;

  @IsNumber()
  @IsNotEmpty()
  gemAmount: number;

  @IsNumber()
  @IsNotEmpty()
  galaAmount: number;

  @IsString()
  @IsNotEmpty()
  walletAddress: string;
}

@Controller('transactions')
export class TransactionController {
  constructor(private readonly transactionService: TransactionService) {}

  @Post('burn')
  async burnTokens(@Body() body: BurnTransactionDto) {
    try {
      const { signedTransaction, gemAmount, galaAmount, walletAddress } = body;

      if (gemAmount <= 0 || galaAmount <= 0) {
        throw new HttpException('Amounts must be positive', HttpStatus.BAD_REQUEST);
      }

      const result = await this.transactionService.processBurnTransaction(
        signedTransaction,
        gemAmount,
        galaAmount,
        walletAddress
      );

      return {
        success: true,
        message: 'Transaction processed successfully',
        ...result
      };

    } catch (error) {
      throw new HttpException(
        error.message || 'Failed to process burn transaction',
        error.status || HttpStatus.INTERNAL_SERVER_ERROR
      );
    }
  }

  @Get('history/:address')
  async getTransactionHistory(@Param('address') address: string) {
    try {
      const result = await this.transactionService.getTransactionHistory(address);
      return {
        success: true,
        ...result
      };
    } catch (error) {
      throw new HttpException(
        error.message || 'Failed to fetch transaction history',
        error.status || HttpStatus.INTERNAL_SERVER_ERROR
      );
    }
  }
}
