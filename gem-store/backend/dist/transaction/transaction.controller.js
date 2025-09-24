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
exports.TransactionController = exports.BurnTransactionDto = void 0;
const common_1 = require("@nestjs/common");
const class_validator_1 = require("class-validator");
const transaction_service_1 = require("./transaction.service");
class BurnTransactionDto {
}
exports.BurnTransactionDto = BurnTransactionDto;
__decorate([
    (0, class_validator_1.IsObject)(),
    (0, class_validator_1.IsNotEmpty)(),
    __metadata("design:type", Object)
], BurnTransactionDto.prototype, "signedTransaction", void 0);
__decorate([
    (0, class_validator_1.IsNumber)(),
    (0, class_validator_1.IsNotEmpty)(),
    __metadata("design:type", Number)
], BurnTransactionDto.prototype, "gemAmount", void 0);
__decorate([
    (0, class_validator_1.IsNumber)(),
    (0, class_validator_1.IsNotEmpty)(),
    __metadata("design:type", Number)
], BurnTransactionDto.prototype, "galaAmount", void 0);
__decorate([
    (0, class_validator_1.IsString)(),
    (0, class_validator_1.IsNotEmpty)(),
    __metadata("design:type", String)
], BurnTransactionDto.prototype, "walletAddress", void 0);
let TransactionController = class TransactionController {
    constructor(transactionService) {
        this.transactionService = transactionService;
    }
    async burnTokens(body) {
        try {
            const { signedTransaction, gemAmount, galaAmount, walletAddress } = body;
            if (gemAmount <= 0 || galaAmount <= 0) {
                throw new common_1.HttpException('Amounts must be positive', common_1.HttpStatus.BAD_REQUEST);
            }
            const result = await this.transactionService.processBurnTransaction(signedTransaction, gemAmount, galaAmount, walletAddress);
            return {
                success: true,
                message: 'Transaction processed successfully',
                ...result
            };
        }
        catch (error) {
            throw new common_1.HttpException(error.message || 'Failed to process burn transaction', error.status || common_1.HttpStatus.INTERNAL_SERVER_ERROR);
        }
    }
    async getTransactionHistory(address) {
        try {
            const result = await this.transactionService.getTransactionHistory(address);
            return {
                success: true,
                ...result
            };
        }
        catch (error) {
            throw new common_1.HttpException(error.message || 'Failed to fetch transaction history', error.status || common_1.HttpStatus.INTERNAL_SERVER_ERROR);
        }
    }
};
exports.TransactionController = TransactionController;
__decorate([
    (0, common_1.Post)('burn'),
    __param(0, (0, common_1.Body)()),
    __metadata("design:type", Function),
    __metadata("design:paramtypes", [BurnTransactionDto]),
    __metadata("design:returntype", Promise)
], TransactionController.prototype, "burnTokens", null);
__decorate([
    (0, common_1.Get)('history/:address'),
    __param(0, (0, common_1.Param)('address')),
    __metadata("design:type", Function),
    __metadata("design:paramtypes", [String]),
    __metadata("design:returntype", Promise)
], TransactionController.prototype, "getTransactionHistory", null);
exports.TransactionController = TransactionController = __decorate([
    (0, common_1.Controller)('transactions'),
    __metadata("design:paramtypes", [transaction_service_1.TransactionService])
], TransactionController);
//# sourceMappingURL=transaction.controller.js.map