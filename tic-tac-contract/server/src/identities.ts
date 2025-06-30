import fs from 'node:fs';
import { Context } from 'koa';
import { createValidDTO, createValidSubmitDTO, ChainUser, RegisterEthUserDto } from '@gala-chain/api'

import {
  GalaChainResponse,
  GetMyProfileDto,
  RegisterUserDto,
  UserProfile,
  randomUniqueKey,
  signatures
} from "@gala-chain/api";

const adminPrivateKeyPath = process.env.CHAIN_ADMIN_SECRET_KEY_PATH ?? '';
let adminPrivateKeyString: string = "";

try {
  adminPrivateKeyString = fs.readFileSync(adminPrivateKeyPath).toString();
} catch (e) {
  console.log(
    `Failed to read admin key file for identity management: ${adminPrivateKeyPath ?? "undefined"}. ` +
    `Provide file path in env CHAIN_ADMIN_SECRET_KEY to use /identities/registerEthUser etc. ${e}`
  );
}

const apiBase = process.env.CHAIN_API ?? 'http://localhost:3000';
const channel = process.env.PRODUCT_CHANNEL ?? 'product';

export function adminSigningKey() {
  return adminPrivateKeyString;
}

export async function registerRandomEthUser(ctx: Context) {
  if (!adminPrivateKeyString) {
    ctx.status = 500;
    ctx.body = `No administrative chain credentials available for new user creation\n`;
    return;
  }

  const url = `${apiBase}/api/${channel}/PublicKeyContract/RegisterEthUser`

  const newUser = ChainUser.withRandomKeys();

  const dto = new RegisterEthUserDto();
  dto.publicKey = newUser.publicKey;
  dto.uniqueKey = randomUniqueKey();
  dto.sign(adminPrivateKeyString, false);

  const chainRes = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: dto.serialize()
  });

  if (!chainRes.ok) {
    ctx.status = 500;
    ctx.body = {
      url: url,
      status: chainRes.status,
      body: chainRes.body,
      dto: dto.serialize(),
      user: newUser
    };

    return;
  }

  ctx.status = 200;
  ctx.body = {
    response: chainRes.json(),
    user: newUser
  };
}

export interface IRegisterUser {
  publicKey: string;
}

export async function registerEthUser(ctx: Context) {
  if (!adminPrivateKeyString) {
    ctx.status = 500;
    ctx.body = `No administrative chain credentials available for new user creation\n`;
    return;
  }

  const url = `${apiBase}/api/${channel}/PublicKeyContract/RegisterEthUser`
  const requestBody: IRegisterUser = ctx.request.body as IRegisterUser;

  if (!requestBody || typeof requestBody.publicKey !== "string") {
    ctx.status = 400;
    ctx.body = `Bad Request\n`;
    return;
  }

  const dto = await createValidSubmitDTO(RegisterEthUserDto, {
    publicKey: requestBody.publicKey,
    uniqueKey: randomUniqueKey()
  });

  dto.sign(adminPrivateKeyString, false);

  const chainRes = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: dto.serialize()
  });

  if (!chainRes.ok) {
    ctx.status = 500;
    ctx.body = {
      url: url,
      status: chainRes.status,
      body: chainRes.body,
      dto: dto.serialize(),
    };

    return;
  }

  ctx.status = 201;
  ctx.body = dto.serialize();
}
