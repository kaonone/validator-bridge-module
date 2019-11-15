import { BigInt } from "@graphprotocol/graph-ts";
import {
  Contract,
  BridgeStopped,
  BridgeStarted,
  BridgePaused,
  BridgeResumed,
  RelayMessage,
  RevertMessage,
  WithdrawMessage,
  ApprovedRelayMessage,
  ConfirmMessage,
  ConfirmWithdrawMessage,
  СancellationСonfirmedMessage,
  WithdrawTransferCall,
  ValidatorAddedMessage,
  ValidatorRemovedMessage,
  ProposalCreatedMessage,
  ProposalApprovedMessage
} from "../generated/Contract/Contract";
import {
  Message,
  BridgeMessage,
  ValidatorMessage,
  LimitMessage,
  Proposal,
  BridgeLimits
} from "../generated/schema";

export function handleRelayMessage(event: RelayMessage): void {
  let message = new Message(event.params.messageID.toHex());
  message.ethAddress = event.params.sender.toHexString();
  message.subAddress = event.params.recipient.toHexString();
  message.amount = event.params.amount;
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
  message.ethAddress = event.params.substrateSender.toHexString();
  message.subAddress = event.params.recipient.toHexString();
  message.amount = event.params.amount;
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

export function handleConfirmWithdrawMessage(event: ConfirmWithdrawMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CONFIRMED_WITHDRAW");
}

export function handleСancellationСonfirmedMessage(event: СancellationСonfirmedMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CANCELED");
}

export function handleBridgeStopped(event: BridgeStopped): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex());
  bridge_message.action = "STOP";
  bridge_message.sender = event.params.sender.toHexString();
  bridge_message.status = "PENDING";
  bridge_message.ethBlockNumber = event.block.number;
  bridge_message.save();
}

export function handleBridgeStarted(event: BridgeStarted): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex());
  bridge_message.action = "START";
  bridge_message.sender = event.params.sender.toHexString();
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

export function handleValidatorAddedMessage(event: ValidatorAddedMessage): void {
  let validator_message = new ValidatorMessage(event.params.messageID.toHex());
  validator_message.action = "ADD";
  validator_message.validator = event.params.validatorAddress.toHexString();
  validator_message.status = "PENDING";
  validator_message.ethBlockNumber = event.block.number;
  validator_message.save();
}

export function handleProposalCreatedMessage(event: ProposalCreatedMessage): void {
  let proposal = new Proposal(event.params.proposalID.toHex());
  let limits = new BridgeLimits(); 
  
  //ETH Limits
  limits.inHostTransactionValue = event.params.inHostTransactionValue;
  limits.axHostTransactionValue = event.params.axHostTransactionValue;
  limits.ayHostMaxLimit = event.params.ayHostMaxLimit;
  limits.ayHostMaxLimitForOneAddress = event.params.ayHostMaxLimitForOneAddress;
  limits.axHostPendingTransactionLimit = event.params.axHostPendingTransactionLimit;
  //guest chain Limits
  limits.inGuestTransactionValue = event.params.inGuestTransactionValue;
  limits.axGuestTransactionValue = event.params.axGuestTransactionValue;
  limits.ayGuestMaxLimit = event.params.ayGuestMaxLimit;
  limits.ayGuestMaxLimitForOneAddress = event.params.ayGuestMaxLimitForOneAddress;
  limits.axGuestPendingTransactionLimit = event.params.axGuestPendingTransactionLimit;

  proposal.limits = limits;
  proposal.status = "PENDING";
  proposal.ethBlockNumber = event.block.number;
  proposal.save();
}

export function handleProposalApprovedMessage(event: ProposalApprovedMessage): void {
  changeProposalStatus(event.params.proposalID.toHex(), "APPROVED");
}

function changeMessageStatus(id: String, status: String): void {
  let message = Message.load(id);
  if (message != null) {
    message.status = status;
    message.save();
  }
}

function changeProposalStatus(id: String, status: String): void {
  let proposal = Proposal.load(id);
  if (proposal != null) {
    proposal.status = status;
    proposal.save();
  }
}
