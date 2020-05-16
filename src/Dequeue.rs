// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// Access to the dequeue operations of a queue.
///
/// All implementations of `FixedSizeMessageBody` when specified in `Enqueue::enqueue()`. must share the same `DequeuedMessageProcessingError` when dequeued and processed.
trait Dequeue<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error>
{
	/// Dequeues messages.
	fn dequeue(&self, terminate: &Arc<impl Terminate>, message_handler_arguments: &MessageHandlerArguments) -> Result<(), DequeuedMessageProcessingError>;
}
