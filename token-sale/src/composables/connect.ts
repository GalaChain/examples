import { BrowserConnectClient, TokenApi } from "@gala-chain/connect"
import { TokenBalance, FetchBalancesDto, FetchTokenClassesDto, TokenClass } from "@gala-chain/api"
import { plainToInstance } from "class-transformer";
import BigNumber from "bignumber.js";

export const useConnect = () => {
  const galaChainBaseUri = import.meta.env.VITE_GALA_CHAIN_BASE_URI as string;
  const connectClient = new BrowserConnectClient();
  const tokenApi = new TokenApi(`${galaChainBaseUri}/api/asset/token-contract`, connectClient); 

  const fetchTokenBalances = async (dto: FetchBalancesDto) => {
    const response = await tokenApi.FetchBalances(dto);
    if(response.status === 1) {
      return plainToInstance(Array<TokenBalance>, response.data);
    } else {
      throw new Error(response.message);
    }
  }

  const fetchTokenClasses = async (dto: FetchTokenClassesDto) => {
    const response = await tokenApi.FetchTokenClasses(dto);
    if(response.status === 1) {
      return plainToInstance(Array<TokenClass>, response.data);
    } else {
      throw new Error(response.message);
    }
  }

  const fetchTokenSaleById = async (tokenSaleId: string) => {
    // Todo
    return  {
      tokenSaleId,
      created: Date.now(),
      txId: '',
      selling: [{
        tokenClassKey: {
          collection: 'DragonStrike',
          category: 'Hero',
          type: 'Marian',
          additionalKey: 'Rare'
        },
        quantity: BigNumber(1000)
      }],
      cost: [{
        tokenClassKey: {
          collection: 'GALA',
          category: 'Unit',
          type: 'none',
          additionalKey: 'none'
        },
        quantity: BigNumber(1000)
      }],
      owner: 'client|mock',
      start: 0,
      end: Date.now() + 3600 * 1000,
      quantity: BigNumber(100),
      quantityFulfilled: BigNumber(100)
    }
  }

  return {
    connectClient,
    fetchTokenBalances,
    fetchTokenClasses,
    fetchTokenSaleById,
  }
}