"use strict";
var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
var __metadata = (this && this.__metadata) || function (k, v) {
    if (typeof Reflect === "object" && typeof Reflect.metadata === "function") return Reflect.metadata(k, v);
};
var __param = (this && this.__param) || function (paramIndex, decorator) {
    return function (target, key) { decorator(target, key, paramIndex); }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.TransactionService = void 0;
const common_1 = require("@nestjs/common");
const typeorm_1 = require("@nestjs/typeorm");
const typeorm_2 = require("typeorm");
const transaction_entity_1 = require("../entities/transaction.entity");
const user_entity_1 = require("../entities/user.entity");
const wallet_service_1 = require("../wallet/wallet.service");
let TransactionService = class TransactionService {
    constructor(transactionRepository, userRepository, walletService) {
        this.transactionRepository = transactionRepository;
        this.userRepository = userRepository;
        this.walletService = walletService;
    }
    async processBurnTransaction(signedTransaction, gemAmount, galaAmount, walletAddress) {
        try {
            const isValid = await this.walletService.validateWalletAddress(walletAddress);
            if (!isValid) {
                throw new common_1.HttpException('Invalid wallet address', common_1.HttpStatus.BAD_REQUEST);
            }
            const galaChainResponse = await this.submitToGalaChain(signedTransaction);
            if (!galaChainResponse.success) {
                throw new common_1.HttpException('GalaChain transaction failed', common_1.HttpStatus.BAD_REQUEST);
            }
            const transaction = this.transactionRepository.create({
                userWalletAddress: walletAddress,
                galaAmount,
                gemAmount,
                transactionId: galaChainResponse.transactionId,
                status: 'completed'
            });
            await this.transactionRepository.save(transaction);
            await this.walletService.addGems(walletAddress, gemAmount);
            return { transactionId: galaChainResponse.transactionId };
        }
        catch (error) {
            const transaction = this.transactionRepository.create({
                userWalletAddress: walletAddress,
                galaAmount,
                gemAmount,
                transactionId: `failed-${Date.now()}`,
                status: 'failed'
            });
            await this.transactionRepository.save(transaction);
            throw new common_1.HttpException(error.message || 'Transaction processing failed', error.status || common_1.HttpStatus.INTERNAL_SERVER_ERROR);
        }
    }
    async getTransactionHistory(walletAddress) {
        try {
            const isValid = await this.walletService.validateWalletAddress(walletAddress);
            if (!isValid) {
                throw new common_1.HttpException('Invalid wallet address', common_1.HttpStatus.BAD_REQUEST);
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
        }
        catch (error) {
            throw new common_1.HttpException(error.message || 'Failed to fetch transaction history', error.status || common_1.HttpStatus.INTERNAL_SERVER_ERROR);
        }
    }
    async submitToGalaChain(signedTransaction) {
        try {
            const response = await fetch(`${process.env.TOKEN_GATEWAY_API || 'https://gateway-mainnet.galachain.com/api/asset/token-contract'}/BurnTokens`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(signedTransaction)
            });
            if (!response.ok) {
                throw new Error(`GalaChain API error: ${response.status}`);
            }
            const result = await response.json();
            return {
                success: true,
                transactionId: result.transactionId || `tx-${Date.now()}`
            };
        }
        catch (error) {
            console.error('GalaChain submission error:', error);
            return {
                success: false,
                transactionId: `failed-${Date.now()}`
            };
        }
    }
};
exports.TransactionService = TransactionService;
exports.TransactionService = TransactionService = __decorate([
    (0, common_1.Injectable)(),
    __param(0, (0, typeorm_1.InjectRepository)(transaction_entity_1.Transaction)),
    __param(1, (0, typeorm_1.InjectRepository)(user_entity_1.User)),
    __metadata("design:paramtypes", [typeorm_2.Repository,
        typeorm_2.Repository,
        wallet_service_1.WalletService])
], TransactionService);
//# sourceMappingURL=transaction.service.js.map