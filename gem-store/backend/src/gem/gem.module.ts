import { Module } from '@nestjs/common';
import { GemController } from './gem.controller';
import { GemService } from './gem.service';

@Module({
  controllers: [GemController],
  providers: [GemService],
  exports: [GemService],
})
export class GemModule {}
