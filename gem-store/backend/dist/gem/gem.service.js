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
Object.defineProperty(exports, "__esModule", { value: true });
exports.GemService = void 0;
const common_1 = require("@nestjs/common");
let GemService = class GemService {
    constructor() {
        this.exchangeRate = parseInt(process.env.GEM_EXCHANGE_RATE || '10');
    }
    getGemPackages() {
        return [
            { id: 1, gems: 10, gala: 1, description: 'Starter Pack' },
            { id: 2, gems: 50, gala: 5, description: 'Value Pack' },
            { id: 3, gems: 100, gala: 10, description: 'Popular Pack' },
            { id: 4, gems: 500, gala: 50, description: 'Premium Pack' }
        ];
    }
    getExchangeRate() {
        return this.exchangeRate;
    }
    calculateGemsFromGala(galaAmount) {
        return Math.floor(galaAmount * this.exchangeRate);
    }
    calculateGalaFromGems(gemAmount) {
        return Math.ceil(gemAmount / this.exchangeRate);
    }
    validateGemPackage(packageId, galaAmount, gemAmount) {
        const packages = this.getGemPackages();
        const pkg = packages.find(p => p.id === packageId);
        if (!pkg) {
            return false;
        }
        return pkg.gala === galaAmount && pkg.gems === gemAmount;
    }
};
exports.GemService = GemService;
exports.GemService = GemService = __decorate([
    (0, common_1.Injectable)(),
    __metadata("design:paramtypes", [])
], GemService);
//# sourceMappingURL=gem.service.js.map