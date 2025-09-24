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
exports.WalletController = void 0;
const common_1 = require("@nestjs/common");
const wallet_service_1 = require("./wallet.service");
let WalletController = class WalletController {
    constructor(walletService) {
        this.walletService = walletService;
    }
    async getBalance(address) {
        try {
            const isValid = await this.walletService.validateWalletAddress(address);
            if (!isValid) {
                throw new common_1.HttpException('Invalid wallet address format', common_1.HttpStatus.BAD_REQUEST);
            }
            const balance = await this.walletService.getUserBalance(address);
            return balance;
        }
        catch (error) {
            throw new common_1.HttpException(error.message || 'Failed to get balance', error.status || common_1.HttpStatus.INTERNAL_SERVER_ERROR);
        }
    }
    async connectWallet(body) {
        try {
            const { walletAddress } = body;
            const isValid = await this.walletService.validateWalletAddress(walletAddress);
            if (!isValid) {
                throw new common_1.HttpException('Invalid wallet address format', common_1.HttpStatus.BAD_REQUEST);
            }
            await this.walletService.findOrCreateUser(walletAddress);
            return {
                success: true,
                message: 'Wallet connected successfully',
                walletAddress
            };
        }
        catch (error) {
            throw new common_1.HttpException(error.message || 'Failed to connect wallet', error.status || common_1.HttpStatus.INTERNAL_SERVER_ERROR);
        }
    }
};
exports.WalletController = WalletController;
__decorate([
    (0, common_1.Get)('balance/:address'),
    __param(0, (0, common_1.Param)('address')),
    __metadata("design:type", Function),
    __metadata("design:paramtypes", [String]),
    __metadata("design:returntype", Promise)
], WalletController.prototype, "getBalance", null);
__decorate([
    (0, common_1.Post)('connect'),
    __param(0, (0, common_1.Body)()),
    __metadata("design:type", Function),
    __metadata("design:paramtypes", [Object]),
    __metadata("design:returntype", Promise)
], WalletController.prototype, "connectWallet", null);
exports.WalletController = WalletController = __decorate([
    (0, common_1.Controller)('wallet'),
    __metadata("design:paramtypes", [wallet_service_1.WalletService])
], WalletController);
//# sourceMappingURL=wallet.controller.js.map