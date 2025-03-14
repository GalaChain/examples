import { Context } from 'koa';
import { createValidDTO, createValidSubmitDTO, RegisterEthUserDto } from '@gala-chain/api'

const apiBase = process.env.CHAIN_API ?? 'http://localhost:3000';
const channel = process.env.PRODUCT_CHANNEL ?? 'product';

export async function proxy(ctx: Context) {
  const channel = ctx.params.channel;
  const contract = ctx.params.contract;
  const method = ctx.params.method;

  const url = `${apiBase}/api/${channel}/${contract}/${method}`
  const dto = ctx.request.body as Record<string, unknown>;

  console.log(`proxy request dto: ${JSON.stringify(dto)}, to ${url}`);

  const chainRes = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(dto)
  });

  if (!chainRes.ok) {
    console.log(`proxy request failed: ${chainRes.status} from ${url}`);

    ctx.status = chainRes.status ?? 500;
    ctx.body = (await chainRes.json());
    return;
  }

  const chainResData = await chainRes.json();

  ctx.status = chainRes.status;
  ctx.body = chainResData;
}
