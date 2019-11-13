import { BigInt } from "@graphprotocol/graph-ts"
import {
  Contract,
  BridgeStoppedMessage,
  BridgeStartedMessage,
  BridgePausedMessage,
  BridgeResumedMessage,
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
  HostAccountPausedMessage,
  HostAccountResumedMessage,
  GuestAccountPausedMessage,
  GuestAccountResumedMessage,
} from "../generated/Contract/Contract"
import { Message, Account, AccountMessage, BridgeMessage, ValidatorMessage } from "../generated/schema"

export function handleRelayMessage(event: RelayMessage): void {
  let message = new Message(event.params.messageID.toHex())
  message.ethAddress = event.params.sender.toHexString()
  message.subAddress = event.params.recipient.toHexString()
  message.amount = event.params.amount
  message.status = "PENDING"
  message.direction = "ETH2SUB"
  message.ethBlockNumber = event.block.number
  message.save()
}

export function handleRevertMessage(event: RevertMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CANCELED")
}

export function handleWithdrawMessage(event: WithdrawMessage): void {
  let message = new Message(event.params.messageID.toHex())
  message.ethAddress = event.params.substrateSender.toHexString()
  message.subAddress = event.params.recipient.toHexString()
  message.amount = event.params.amount
  message.status = "WITHDRAW"
  message.direction = "SUB2ETH"
  message.ethBlockNumber = event.block.number
  message.save()
}

export function handleApprovedRelayMessage(event: ApprovedRelayMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "APPROVED")
}

export function handleConfirmMessage(event: ConfirmMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CONFIRMED")
}

export function handleConfirmWithdrawMessage(event: ConfirmWithdrawMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CONFIRMED_WITHDRAW")
}

export function handleСancellationСonfirmedMessage(event: СancellationСonfirmedMessage): void {
  changeMessageStatus(event.params.messageID.toHex(), "CANCELED")
}

export function handleBridgeStoppedMessage(event: BridgeStoppedMessage): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex())
  bridge_message.action = "STOP"
  bridge_message.sender = event.params.sender.toHexString();
  bridge_message.status = "PENDING"
  bridge_message.ethBlockNumber = event.block.number
  bridge_message.save()
}

export function handleBridgeStartedMessage(event: BridgeStartedMessage): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex())
  bridge_message.action = "START"
  bridge_message.sender = event.params.sender.toHexString();
  bridge_message.status = "PENDING"
  bridge_message.ethBlockNumber = event.block.number
  bridge_message.save()
}

export function handleBridgePausedMessage(event: BridgePausedMessage): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex())
  bridge_message.action = "PAUSE"
  bridge_message.status = "PENDING"
  bridge_message.ethBlockNumber = event.block.number
  bridge_message.save()
}

export function handleBridgeResumedMessage(event: BridgeResumedMessage): void {
  let bridge_message = new BridgeMessage(event.params.messageID.toHex())
  bridge_message.action = "RESUME"
  bridge_message.status = "PENDING"
  bridge_message.ethBlockNumber = event.block.number
  bridge_message.save()
}

export function handleValidatorAddedMessage(event: ValidatorAddedMessage): void {
  let validator_message = new ValidatorMessage(event.params.messageID.toHex())
  validator_message.action = "ADD"
  validator_message.validator = event.params.validatorAddress.toHexString()
  validator_message.status = "PENDING"
  validator_message.ethBlockNumber = event.block.number
  validator_message.save()
}

export function handleValidatorRemovedMessage(event: ValidatorRemovedMessage): void {
  let validator_message = new ValidatorMessage(event.params.messageID.toHex())
  validator_message.action = "REMOVE"
  validator_message.validator = event.params.validatorAddress.toHexString()
  validator_message.status = "PENDING"
  validator_message.ethBlockNumber = event.block.number
  validator_message.save()
}

export function handleHostAccountPausedMessage(event: HostAccountPausedMessage): void {
  let id = event.params.messageID.toHex()
  let account_message = new AccountMessage(id)
  let ethAddress = event.params.sender.toHexString()
  account_message.action = "PAUSE"
  account_message.direction = "ETH2SUB"
  account_message.ethAddress = ethAddress
  account_message.timestamp = event.params.timestamp
  account_message.ethBlockNumber = event.block.number
  account_message.save()

  createOrUpdateAccount(ethAddress, id, "ETH", "BLOCKED", event.params.timestamp, event.block.number)
}

export function handleHostAccountResumedMessage(event: HostAccountResumedMessage): void {
  let id = event.params.messageID.toHex()
  let account_message = new AccountMessage(id)
  let ethAddress = event.params.sender.toHexString()
  account_message.action = "RESUME"
  account_message.direction = "ETH2SUB"
  account_message.ethAddress = ethAddress
  account_message.timestamp = event.params.timestamp
  account_message.ethBlockNumber = event.block.number
  account_message.save()

  createOrUpdateAccount(ethAddress, id, "ETH", "ACTIVE", event.params.timestamp, event.block.number)
}

export function handleGuestAccountPausedMessage(event: GuestAccountPausedMessage): void {
  let id = event.params.messageID.toHex()
  let account_message = new AccountMessage(id)
  let subAddress = event.params.sender.toHexString()
  account_message.action = "PAUSE"
  account_message.direction = "SUB2ETH"
  account_message.subAddress = subAddress
  account_message.timestamp = event.params.timestamp
  account_message.ethBlockNumber = event.block.number
  account_message.save()

  createOrUpdateAccount(subAddress, id, "SUB", "BLOCKED", event.params.timestamp, event.block.number)
}

export function handleGuestAccountResumedMessage(event: GuestAccountResumedMessage): void {
  let id = event.params.messageID.toHex()
  let account_message = new AccountMessage(id)
  let subAddress = event.params.sender.toHexString()
  account_message.action = "RESUME"
  account_message.direction = "SUB2ETH"
  account_message.subAddress = subAddress
  account_message.timestamp = event.params.timestamp
  account_message.ethBlockNumber = event.block.number
  account_message.save()

  createOrUpdateAccount(subAddress, id, "SUB", "ACTIVE", event.params.timestamp, event.block.number)
}

function changeMessageStatus(id: String, status: String): void {
  let message = Message.load(id)
  if (message != null) {
    message.status = status
    message.save()
  }
}

function createOrUpdateAccount(id: String, messageId: String, kind: String, status: String, timestamp: BigInt, ethBlockNumber: BigInt): void {
  let account = Account.load(id)
  if (account == null) {
    account = new Account(id)
  }
  account.messageId = messageId
  account.kind = kind
  account.status = status
  account.timestamp = timestamp
  account.ethBlockNumber = ethBlockNumber
  account.save()
}
