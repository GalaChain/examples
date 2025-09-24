import { Entity, PrimaryGeneratedColumn, Column, CreateDateColumn, ManyToOne, JoinColumn } from 'typeorm';
import { User } from './user.entity';

// Transaction status values: 'pending', 'completed', 'failed'

@Entity('transactions')
export class Transaction {
  @PrimaryGeneratedColumn()
  id: number;

  @Column()
  userWalletAddress: string;

  @Column('decimal', { precision: 10, scale: 6 })
  galaAmount: number;

  @Column('int')
  gemAmount: number;

  @Column()
  transactionId: string;

  @Column({
    type: 'varchar',
    default: 'pending'
  })
  status: string;

  @CreateDateColumn()
  createdAt: Date;

  @ManyToOne(() => User, user => user.transactions)
  @JoinColumn({ name: 'userWalletAddress', referencedColumnName: 'walletAddress' })
  user: User;
}
