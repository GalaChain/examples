"use strict";
var __decorate = (this && this.__decorate) || function (decorators, target, key, desc) {
    var c = arguments.length, r = c < 3 ? target : desc === null ? desc = Object.getOwnPropertyDescriptor(target, key) : desc, d;
    if (typeof Reflect === "object" && typeof Reflect.decorate === "function") r = Reflect.decorate(decorators, target, key, desc);
    else for (var i = decorators.length - 1; i >= 0; i--) if (d = decorators[i]) r = (c < 3 ? d(r) : c > 3 ? d(target, key, r) : d(target, key)) || r;
    return c > 3 && r && Object.defineProperty(target, key, r), r;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.GemModule = void 0;
const common_1 = require("@nestjs/common");
const gem_controller_1 = require("./gem.controller");
const gem_service_1 = require("./gem.service");
let GemModule = class GemModule {
};
exports.GemModule = GemModule;
exports.GemModule = GemModule = __decorate([
    (0, common_1.Module)({
        controllers: [gem_controller_1.GemController],
        providers: [gem_service_1.GemService],
        exports: [gem_service_1.GemService],
    })
], GemModule);
//# sourceMappingURL=gem.module.js.map