import { Controller, Get } from '@nestjs/common';
import { GemService } from './gem.service';

@Controller('gems')
export class GemController {
  constructor(private readonly gemService: GemService) {}

  @Get('packages')
  getGemPackages() {
    return {
      success: true,
      packages: this.gemService.getGemPackages(),
      exchangeRate: this.gemService.getExchangeRate()
    };
  }

  @Get('exchange-rate')
  getExchangeRate() {
    return {
      success: true,
      exchangeRate: this.gemService.getExchangeRate(),
      description: 'Gems per GALA token'
    };
  }
}
