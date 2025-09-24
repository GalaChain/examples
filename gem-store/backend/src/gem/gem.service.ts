import { Injectable } from '@nestjs/common';

export interface GemPackage {
  id: number;
  gems: number;
  gala: number;
  description?: string;
}

@Injectable()
export class GemService {
  private readonly exchangeRate: number;

  constructor() {
    this.exchangeRate = parseInt(process.env.GEM_EXCHANGE_RATE || '10');
  }

  getGemPackages(): GemPackage[] {
    return [
      { id: 1, gems: 10, gala: 1, description: 'Starter Pack' },
      { id: 2, gems: 50, gala: 5, description: 'Value Pack' },
      { id: 3, gems: 100, gala: 10, description: 'Popular Pack' },
      { id: 4, gems: 500, gala: 50, description: 'Premium Pack' }
    ];
  }

  getExchangeRate(): number {
    return this.exchangeRate;
  }

  calculateGemsFromGala(galaAmount: number): number {
    return Math.floor(galaAmount * this.exchangeRate);
  }

  calculateGalaFromGems(gemAmount: number): number {
    return Math.ceil(gemAmount / this.exchangeRate);
  }

  validateGemPackage(packageId: number, galaAmount: number, gemAmount: number): boolean {
    const packages = this.getGemPackages();
    const pkg = packages.find(p => p.id === packageId);
    
    if (!pkg) {
      return false;
    }

    return pkg.gala === galaAmount && pkg.gems === gemAmount;
  }
}
