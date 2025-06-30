import { asValidUserRef, createValidDTO } from "@gala-chain/api";
import { GalaChainContext, authorize, deleteChainObject, getObjectByKey } from "@gala-chain/chaincode";

import { SubmissionCommitment, WithdrawSubmissionDto, WithdrawSubmissionResDto } from "./api";

export async function withdrawSubmission(
  ctx: GalaChainContext,
  dto: WithdrawSubmissionDto
): Promise<WithdrawSubmissionResDto> {
  const { collection, hash, nonce } = dto;
  const owner = asValidUserRef(ctx.callingUser);

  const commitmentKey = SubmissionCommitment.getCompositeKeyFromParts(SubmissionCommitment.INDEX_KEY, [
    collection,
    owner,
    hash,
    nonce
  ]);

  const entry = await getObjectByKey(ctx, SubmissionCommitment, commitmentKey);

  deleteChainObject(ctx, entry);

  const response = await createValidDTO(WithdrawSubmissionResDto, {
    collection,
    owner,
    hash,
    nonce
  });

  return response;
}
