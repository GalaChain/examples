import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { WalletModule } from './wallet/wallet.module';
import { TransactionModule } from './transaction/transaction.module';
import { GemModule } from './gem/gem.module';
import { User } from './entities/user.entity';
import { Transaction } from './entities/transaction.entity';

@Module({
  imports: [
    TypeOrmModule.forRoot({
      type: 'sqlite',
      database: 'gem-store.db',
      entities: [User, Transaction],
      synchronize: true, // Only for development
      logging: true,
    }),
    WalletModule,
    TransactionModule,
    GemModule,
  ],
})
export class AppModule {}
