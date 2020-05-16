// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// A subscriber to a queue.
///
/// Create using `Queues::subscribe()`.
///
/// Not thread safe; one instance per thread is required.
///
/// `MessageHandlerArguments` must be common to all possible message types (all possible `FixedSizeMessageBody` and `CompressedTypeIdentifier`s).
/// `DequeuedMessageProcessingError` must be common to all possible message types (all possible `FixedSizeMessageBody` and `CompressedTypeIdentifier`s).
pub struct Subscriber<'a, MessageHandlerArguments, DequeuedMessageProcessingError: error::Error>(&'a Queue<MessageHandlerArguments, DequeuedMessageProcessingError>);

impl<'a, MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Subscriber<'a, MessageHandlerArguments, DequeuedMessageProcessingError>
{
	/// Receives and handles messages; short-circuits if `self.terminate` becomes true or a message handler returns an error `DequeuedMessageProcessingError`.
	#[inline(always)]
	pub fn receive_and_handle_messages(&self, terminate: &Arc<impl Terminate>, message_handler_arguments: &MessageHandlerArguments) -> Result<(), DequeuedMessageProcessingError>
	{
		self.0.dequeue(terminate, message_handler_arguments)
	}
}
