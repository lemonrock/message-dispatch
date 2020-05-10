// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// A subscriber to a queue.
///
/// `MessageHandlerArguments` must be common to all possible message types (all possible `MessageContents` and `CompressedTypeIdentifier`s).
/// `DequeuedMessageProcessingError` must be common to all possible message types (all possible `MessageContents` and `CompressedTypeIdentifier`s).
#[derive(Debug)]
pub struct PerThreadQueueSubscriber<T: Terminate, MessageHandlerArguments: Debug + Copy, DequeuedMessageProcessingError: Debug>
{
	queue: Arc<Queue<MessageHandlerArguments, DequeuedMessageProcessingError>>,
	terminate: Arc<T>,
}

impl<T: Terminate, MessageHandlerArguments: Debug + Copy, DequeuedMessageProcessingError: Debug> PerThreadQueueSubscriber<T, MessageHandlerArguments, DequeuedMessageProcessingError>
{
	/// Creates a new instance for the current logical core.
	///
	/// Thus must only be run on the thread that is doing subscribing.
	#[inline(always)]
	pub fn new<MHR: MessageHandlersRegistration<MessageHandlerArguments=MessageHandlerArguments, E=DequeuedMessageProcessingError>>(queue_per_threads_publisher: QueuePerThreadQueuesPublisher<MessageHandlerArguments, DequeuedMessageProcessingError>, terminate: Arc<T>, message_handlers_registration: &MHR, message_handlers_registration_arguments: &MHR::Arguments) -> Self
	{
		let hyper_thread_identifier = HyperThread::current().1;

		let queue = queue_per_threads_publisher.get_queue(hyper_thread_identifier);
		message_handlers_registration.register_all_message_handlers(queue.message_handlers(), message_handlers_registration_arguments);

		Self
		{
			queue,
			terminate,
		}
	}

	/// Receives and handles messages; short-circuits if `self.terminate` becomes true or a message handler returns an error `DequeuedMessageProcessingError`.
	#[inline(always)]
	pub fn receive_and_handle_messages(&self, message_handler_arguments: MessageHandlerArguments) -> Result<(), DequeuedMessageProcessingError>
	{
		self.queue.dequeue(self.terminate.deref(), message_handler_arguments)
	}
}
