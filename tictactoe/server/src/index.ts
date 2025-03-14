import 'dotenv/config';
import { Server } from 'boardgame.io/server';
import Koa from 'koa';
import cors from '@koa/cors';
import Router from '@koa/router';
import bodyParser from 'koa-bodyparser';
import { TicTacContract } from './game';
import { Chainstore } from './chainstore';
import { registerRandomEthUser, registerEthUser } from './identities';
import { proxy } from './proxy';

const server = Server({
  games: [TicTacContract],
  db: new Chainstore(process.env.CHAIN_BASE_URL, process.env.CHAIN_CONTRACT_PATH),
  origins: ['http://localhost:5173'] // Default Vite dev server port
});

const app = server.app;

// todo: set cors when NODE_ENV = localdev || development
app.use(cors());

const router = new Router();

router.use('/api', bodyParser());
router.use('/identities', bodyParser());

router.get('/api/status', async (ctx) => {
  ctx.body = { status: 'ok' };
});

router.get('/identities/new-random-user', registerRandomEthUser);
router.post('/identities/CreateHeadlessWallet', registerEthUser);
router.post('/api/:channel/:contract/:method', proxy);

app.use(router.routes())
app.use(router.allowedMethods());

server.run(8000);
