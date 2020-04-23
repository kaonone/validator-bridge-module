import { Address } from "@graphprotocol/graph-ts";
import { BigInt } from "@graphprotocol/graph-ts";
import { ByteArray } from "@graphprotocol/graph-ts";
import { log } from "@graphprotocol/graph-ts";
import { crypto } from "@graphprotocol/graph-ts";
import { SetNewLimits } from "../generated/Limits/Limits";
import { ProposalCreated, ProposalApproved } from "../generated/Dao/Dao";
import { ChangeValidatorsList } from "../generated/ValidatorsOperations/ValidatorsOperations";
import {
  AddCandidateValidator,
  RemoveCandidateValidator,
  ProposalCandidatesValidatorsCreated,
} from "../generated/Candidate/Candidate";
import {
  BridgeStopped,
  BridgeStarted,
  BridgePaused,
  BridgeResumed,
  BridgePausedByVolume,
  BridgeStartedByVolume,
  HostAccountPausedMessage,
  HostAccountResumedMessage,
  GuestAccountPausedMessage,
  GuestAccountResumedMessage,
} from "../generated/Status/Status";
import {
  RelayMessage,
  RevertMessage,
  WithdrawMessage,
  ApprovedRelayMessage,
  ConfirmMessage,
  ConfirmWithdrawMessage,
  ConfirmCancelMessage,
} from "../generated/Transfers/Transfers";
import {
  Account,
  AccountMessage,
  BridgeMessage,
  CandidateValidator,
  CandidateValidatorMessage,
  LimitMessage,
  Limit,
  Message,
  LimitProposal,
  CandidatesValidatorsProposal,
  ValidatorsListMessage,
} from "../generated/schema";

export function handleRelayMessage(event: RelayMessage): void {
  let message = new Message(event.params.messageID.toHex());
  message.ethAddress = event.params.sender.toHexString();
  message.subAddress = event.params.recipient.toHexString();
  message.amount = event.params.amount;
  message.token = event.params.token.toHexString();
  message.status = "PENDING";
  message.direction = "ETH2SUB";
  message.ethBlockNumber = event.block.number;
  message.save();
}

export function handleRevertMessage(event: RevertMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CANCELED");
}

export function handleWithdrawMessage(event: WithdrawMessage): void {
  let message = new Message(event.params.messageID.toHex());
  message.ethAddress = event.params.recepient.toHexString();
  message.subAddress = event.params.sender.toHexString();
  message.amount = event.params.amount;
  message.token = event.params.token.toHexString();
  message.status = "WITHDRAW";
  message.direction = "SUB2ETH";
  message.ethBlockNumber = event.block.number;
  message.save();
}

export function handleApprovedRelayMessage(event: ApprovedRelayMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "APPROVED");
}

export function handleConfirmMessage(event: ConfirmMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CONFIRMED");
}

export function handleConfirmWithdrawMessage(
  event: ConfirmWithdrawMessage
): void {
  changeMessageStatus(event.params.messageID.toHex(), "CONFIRMED_WITHDRAW");
}

export function handleConfirmCancelMessage(event: ConfirmCancelMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CANCELED");
}

export function handleBridgeStarted(event: BridgeStarted): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex());
  bridge_message.action = "START";
  bridge_message.status = "PENDING";
  bridge_message.ethBlockNumber = event.block.number;
  bridge_message.save();
}

export function handleBridgeStopped(event: BridgeStopped): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex());
  bridge_message.action = "STOP";
  bridge_message.status = "PENDING";
  bridge_message.ethBlockNumber = event.block.number;
  bridge_message.save();
}

export function handleBridgePaused(event: BridgePaused): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex());
  bridge_message.action = "PAUSE";
  bridge_message.status = "PENDING";
  bridge_message.ethBlockNumber = event.block.number;
  bridge_message.save();
}

export function handleBridgeResumed(event: BridgeResumed): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex());
  bridge_message.action = "RESUME";
  bridge_message.status = "PENDING";
  bridge_message.ethBlockNumber = event.block.number;
  bridge_message.save();
}

export function handleBridgePausedByVolume(event: BridgePausedByVolume): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex());
  bridge_message.action = "PAUSE";
  bridge_message.status = "PENDING";
  bridge_message.ethBlockNumber = event.block.number;
  bridge_message.save();
}

export function handleBridgeStartedByVolume(
  event: BridgeStartedByVolume
): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex());
  bridge_message.action = "RESUME";
  bridge_message.status = "PENDING";
  bridge_message.ethBlockNumber = event.block.number;
  bridge_message.save();
}

export function handleProposalCreated(event: ProposalCreated): void {
  let proposal = new LimitProposal(event.params.proposalID.toHex());
  proposal.ethAddress = event.params.sender.toHexString();
  proposal.status = "PENDING";
  proposal.minHostTransactionValue = event.params.minHostTransactionValue;
  proposal.maxHostTransactionValue = event.params.maxHostTransactionValue;
  proposal.dayHostMaxLimit = event.params.dayHostMaxLimit;
  proposal.dayHostMaxLimitForOneAddress =
    event.params.dayHostMaxLimitForOneAddress;
  proposal.maxHostPendingTransactionLimit =
    event.params.maxHostPendingTransactionLimit;
  proposal.minGuestTransactionValue = event.params.minGuestTransactionValue;
  proposal.maxGuestTransactionValue = event.params.maxGuestTransactionValue;
  proposal.dayGuestMaxLimit = event.params.dayGuestMaxLimit;
  proposal.dayGuestMaxLimitForOneAddress =
    event.params.dayGuestMaxLimitForOneAddress;
  proposal.maxGuestPendingTransactionLimit =
    event.params.maxGuestPendingTransactionLimit;
  proposal.ethBlockNumber = event.block.number;
  proposal.save();
}

export function handleProposalApproved(event: ProposalApproved): void {
  changeProposalStatus(event.params.proposalID.toHex(), "APPROVED");
}

export function handleHostAccountPausedMessage(
  event: HostAccountPausedMessage
): void {
  let id = event.params.messageID.toHex();
  let account_message = new AccountMessage(id);
  let ethAddress = event.params.sender.toHexString();
  account_message.action = "PAUSE";
  account_message.direction = "ETH2SUB";
  account_message.ethAddress = ethAddress;
  account_message.timestamp = event.params.timestamp;
  account_message.ethBlockNumber = event.block.number;
  account_message.save();

  createOrUpdateAccount(
    ethAddress,
    id,
    "ETH",
    "BLOCKED",
    event.params.timestamp,
    event.block.number
  );
}

export function handleHostAccountResumedMessage(
  event: HostAccountResumedMessage
): void {
  let id = event.params.messageID.toHex();
  let account_message = new AccountMessage(id);
  let ethAddress = event.params.sender.toHexString();
  account_message.action = "RESUME";
  account_message.direction = "ETH2SUB";
  account_message.ethAddress = ethAddress;
  account_message.timestamp = event.params.timestamp;
  account_message.ethBlockNumber = event.block.number;
  account_message.save();

  createOrUpdateAccount(
    ethAddress,
    id,
    "ETH",
    "ACTIVE",
    event.params.timestamp,
    event.block.number
  );
}

export function handleGuestAccountPausedMessage(
  event: GuestAccountPausedMessage
): void {
  let id = event.params.messageID.toHex();
  let account_message = new AccountMessage(id);
  let subAddress = event.params.recipient.toHexString();
  account_message.action = "PAUSE";
  account_message.direction = "SUB2ETH";
  account_message.subAddress = subAddress;
  account_message.timestamp = event.params.timestamp;
  account_message.ethBlockNumber = event.block.number;
  account_message.save();

  createOrUpdateAccount(
    subAddress,
    id,
    "SUB",
    "BLOCKED",
    event.params.timestamp,
    event.block.number
  );
}

export function handleGuestAccountResumedMessage(
  event: GuestAccountResumedMessage
): void {
  let id = event.params.messageID.toHex();
  let account_message = new AccountMessage(id);
  let subAddress = event.params.recipient.toHexString();
  account_message.action = "RESUME";
  account_message.direction = "SUB2ETH";
  account_message.subAddress = subAddress;
  account_message.timestamp = event.params.timestamp;
  account_message.ethBlockNumber = event.block.number;
  account_message.save();

  createOrUpdateAccount(
    subAddress,
    id,
    "SUB",
    "ACTIVE",
    event.params.timestamp,
    event.block.number
  );
}

export function handleSetNewLimits(event: SetNewLimits): void {
  let id = generateMessageID("0", event.block.number);
  let limitMessage = LimitMessage.load(id);
  if (limitMessage == null) {
    limitMessage = new LimitMessage(id);
  }
  limitMessage.minHostTransactionValue = event.params.minHostTransactionValue;
  limitMessage.maxHostTransactionValue = event.params.maxHostTransactionValue;
  limitMessage.dayHostMaxLimit = event.params.dayHostMaxLimit;
  limitMessage.dayHostMaxLimitForOneAddress =
    event.params.dayHostMaxLimitForOneAddress;
  limitMessage.maxHostPendingTransactionLimit =
    event.params.maxHostPendingTransactionLimit;
  limitMessage.minGuestTransactionValue = event.params.minGuestTransactionValue;
  limitMessage.maxGuestTransactionValue = event.params.maxGuestTransactionValue;
  limitMessage.dayGuestMaxLimit = event.params.dayGuestMaxLimit;
  limitMessage.dayGuestMaxLimitForOneAddress =
    event.params.dayGuestMaxLimitForOneAddress;
  limitMessage.maxGuestPendingTransactionLimit =
    event.params.maxGuestPendingTransactionLimit;
  limitMessage.ethBlockNumber = event.block.number;
  limitMessage.save();

  createOrUpdateLimit(
    "MIN_HOST_TRANSACTION_VALUE",
    event.params.minHostTransactionValue,
    id,
    event.block.number
  );
  createOrUpdateLimit(
    "MAX_HOST_TRANSACTION_VALUE",
    event.params.maxHostTransactionValue,
    id,
    event.block.number
  );
  createOrUpdateLimit(
    "DAY_HOST_MAX_LIMIT",
    event.params.dayHostMaxLimit,
    id,
    event.block.number
  );
  createOrUpdateLimit(
    "DAY_HOST_MAX_LIMIT_FOR_ONE_ADDRESS",
    event.params.dayHostMaxLimitForOneAddress,
    id,
    event.block.number
  );
  createOrUpdateLimit(
    "MAX_HOST_PENDING_TRANSACTION_LIMIT",
    event.params.maxHostPendingTransactionLimit,
    id,
    event.block.number
  );
  createOrUpdateLimit(
    "MIN_GUEST_TRANSACTION_VALUE",
    event.params.minGuestTransactionValue,
    id,
    event.block.number
  );
  createOrUpdateLimit(
    "MAX_GUEST_TRANSACTION_VALUE",
    event.params.maxGuestTransactionValue,
    id,
    event.block.number
  );
  createOrUpdateLimit(
    "DAY_GUEST_MAX_LIMIT",
    event.params.dayGuestMaxLimit,
    id,
    event.block.number
  );
  createOrUpdateLimit(
    "DAY_GUEST_MAX_LIMIT_FOR_ONE_ADDRESS",
    event.params.dayGuestMaxLimitForOneAddress,
    id,
    event.block.number
  );
  createOrUpdateLimit(
    "MAX_GUEST_PENDING_TRANSACTION_LIMIT",
    event.params.maxGuestPendingTransactionLimit,
    id,
    event.block.number
  );
}

export function handleAddCandidateValidator(
  event: AddCandidateValidator
): void {
  let candidateValidatorMessage = new CandidateValidatorMessage(
    event.params.messageID.toHex()
  );
  let ethAddress = event.params.host.toHexString();
  let subAddress = event.params.guest.toHexString();
  candidateValidatorMessage.ethAddress = ethAddress;
  candidateValidatorMessage.subAddress = subAddress;
  candidateValidatorMessage.action = "ADD";
  candidateValidatorMessage.ethBlockNumber = event.block.number;
  candidateValidatorMessage.save();

  createOrUpdateCandidateValidator(
    ethAddress,
    subAddress,
    true,
    event.block.number
  );
}

export function handleRemoveCandidateValidator(
  event: RemoveCandidateValidator
): void {
  let candidateValidatorMessage = new CandidateValidatorMessage(
    event.params.messageID.toHex()
  );
  let ethAddress = event.params.host.toHexString();
  let subAddress = event.params.guest.toHexString();
  candidateValidatorMessage.ethAddress = ethAddress;
  candidateValidatorMessage.subAddress = subAddress;
  candidateValidatorMessage.action = "REMOVE";
  candidateValidatorMessage.ethBlockNumber = event.block.number;
  candidateValidatorMessage.save();

  createOrUpdateCandidateValidator(
    ethAddress,
    subAddress,
    false,
    event.block.number
  );
}

export function handleProposalCandidatesValidatorsCreated(
  event: ProposalCandidatesValidatorsCreated
): void {
  let id = event.params.messageID.toHex();
  let candidatesValidatorsProposal = CandidatesValidatorsProposal.load(id);
  if (candidatesValidatorsProposal == null) {
    candidatesValidatorsProposal = new CandidatesValidatorsProposal(id);
  }
  candidatesValidatorsProposal.status = "PENDING";
  let hosts = event.params.hosts;
  var newHosts = new Array<string>(hosts.length);
  for (let i = 0; i < hosts.length; i++) newHosts[i] = hosts[i].toHexString();
  candidatesValidatorsProposal.hosts = newHosts;
  candidatesValidatorsProposal.ethBlockNumber = event.block.number;
  candidatesValidatorsProposal.save();
}

export function handleChangeValidatorsList(event: ChangeValidatorsList): void {
  let id = event.params.messageID.toHex();
  let validatorsListMessage = new ValidatorsListMessage(id);
  let validatorList = event.params.newvalidators;
  var newValidatorList = new Array<string>(0);
  for (let i = 0; i < validatorList.length; i++) {
    let ethAddress = validatorList[i].toHexString();
    let candidateValidator = CandidateValidator.load(ethAddress);
    if (candidateValidator != null && candidateValidator.active) {
      newValidatorList.push(candidateValidator.subAddress);
    } else {
      log.error(
        "can not found active CandidateValidator for {}, messageID: {}",
        [ethAddress, id]
      );
    }
  }
  validatorsListMessage.newValidators = newValidatorList;
  validatorsListMessage.newHowManyValidatorsDecide =
    event.params.newHowManyValidatorsDecide;
  validatorsListMessage.ethBlockNumber = event.block.number;
  if (newValidatorList.length == validatorList.length) {
    updateStatusOfCandidatesValidatorsProposal(id, "APPROVED");
    validatorsListMessage.save();
  } else {
    log.error("invalid ChangeValidatorsList event, messageID: {}", [id]);
  }
}

function changeMessageStatus(id: string, status: string): void {
  let message = Message.load(id);
  if (message != null) {
    message.status = status;
    message.save();
  }
}

function changeProposalStatus(id: string, status: string): void {
  let proposal = LimitProposal.load(id);
  if (proposal != null) {
    proposal.status = status;
    proposal.save();
  }
}

function createOrUpdateAccount(
  id: string,
  messageID: string,
  kind: string,
  status: string,
  timestamp: BigInt,
  ethBlockNumber: BigInt
): void {
  let account = Account.load(id);
  if (account == null) {
    account = new Account(id);
  }
  account.messageID = messageID;
  account.kind = kind;
  account.status = status;
  account.timestamp = timestamp;
  account.ethBlockNumber = ethBlockNumber;
  account.save();
}

function createOrUpdateLimit(
  id: string,
  value: BigInt,
  messageID: string,
  ethBlockNumber: BigInt
): void {
  let limit = Limit.load(id);
  if (limit == null) {
    limit = new Limit(id);
  }
  limit.value = value;
  limit.messageID = messageID;
  limit.kind = messageID;
  limit.ethBlockNumber = ethBlockNumber;
  limit.save();
}

function createOrUpdateCandidateValidator(
  ethAddress: string,
  subAddress: string,
  active: boolean,
  ethBlockNumber: BigInt
): void {
  let candidateValidator = CandidateValidator.load(ethAddress);
  if (candidateValidator == null) {
    candidateValidator = new CandidateValidator(ethAddress);
  }
  candidateValidator.subAddress = subAddress;
  candidateValidator.active = active;
  candidateValidator.ethBlockNumber = ethBlockNumber;
  candidateValidator.save();
}

function updateStatusOfCandidatesValidatorsProposal(
  id: string,
  status: string
): void {
  let candidatesValidatorsProposal = CandidatesValidatorsProposal.load(id);
  if (candidatesValidatorsProposal != null) {
    candidatesValidatorsProposal.status = status;
    candidatesValidatorsProposal.save();
  } else {
    log.error("can not found CandidatesValidatorsProposal, messageID: {}", [
      id,
    ]);
  }
}

function generateMessageID(salt: string, ethBlockNumber: BigInt): string {
  let hex = normalizeLength(salt.concat(ethBlockNumber.toHexString().slice(2)));
  return crypto.keccak256(ByteArray.fromHexString(hex)).toHexString();
}

function normalizeLength(str: string): string {
  if (str.length % 2 == 1) {
    return "0".concat(str);
  }
  return str;
}
