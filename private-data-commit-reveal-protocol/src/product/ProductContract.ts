import {
  EVALUATE,
  GalaChainContext,
  GalaContract,
  GalaTransaction,
  SUBMIT,
  Submit,
  UnsignedEvaluate
} from "@gala-chain/chaincode";
import { Info } from "fabric-contract-api";

import { version } from "../../package.json";
import {
  CommitSubmissionDto,
  CommitSubmissionResDto,
  RevealSubmissionDto,
  RevealSubmissionResDto,
  WithdrawSubmissionDto,
  WithdrawSubmissionResDto
} from "./api";
import { commitSubmission } from "./commitSubmission";
import { revealSubmission } from "./revealSubmission";
import { withdrawSubmission } from "./withdrawSubmission";

@Info({ title: "Product", description: "Product contract" })
export class ProductContract extends GalaContract {
  constructor() {
    super("Product", version);
  }

  @Submit({
    in: CommitSubmissionDto,
    out: CommitSubmissionResDto
  })
  public async CommitSubmission(
    ctx: GalaChainContext,
    dto: CommitSubmissionDto
  ): Promise<CommitSubmissionResDto> {
    return commitSubmission(ctx, dto);
  }

  @Submit({
    in: WithdrawSubmissionDto,
    out: WithdrawSubmissionResDto
  })
  public async WithdrawSubmission(
    ctx: GalaChainContext,
    dto: WithdrawSubmissionDto
  ): Promise<WithdrawSubmissionResDto> {
    return withdrawSubmission(ctx, dto);
  }

  @Submit({
    in: RevealSubmissionDto,
    out: RevealSubmissionResDto
  })
  public async RevealSubmission(
    ctx: GalaChainContext,
    dto: RevealSubmissionDto
  ): Promise<RevealSubmissionResDto> {
    return revealSubmission(ctx, dto);
  }
}
