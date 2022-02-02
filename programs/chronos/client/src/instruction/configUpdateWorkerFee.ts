import { BN, Program } from "@project-serum/anchor";
import { TransactionInstruction } from "@solana/web3.js";
import { Chronos } from "../idl";
import { Account } from "../account";

export type ConfigUpdateWorkerFeeArgs = {
  newWorkerFee: BN;
};

export class ConfigUpdateWorkerFee {
  private account: Account;
  private chronos: Program<Chronos>;

  constructor(account: Account, chronos: Program<Chronos>) {
    this.account = account;
    this.chronos = chronos;
  }

  public async configUpdateWorkerFee({
    newWorkerFee,
  }: ConfigUpdateWorkerFeeArgs): Promise<TransactionInstruction> {
    const configPDA = await this.account.config.pda();
    const configData = await this.account.config.data(configPDA.address);
    return this.chronos.instruction.configUpdateWorkerFee(newWorkerFee, {
      accounts: {
        admin: configData.adminAuthority,
        config: configPDA.address,
      },
    });
  }
}
