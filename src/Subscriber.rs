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
///
///
/// `queue` holds a reference to `_queues_drop_reference`, making this a self-referential struct.
/// As `_queues_drop_reference` is internally an `Arc`, this is ok as the reference is to a stable location in memory, ie one that doesn't move.
/// This can not be expressed using lifetimes, hence the `*const Queue` below (otherwise the lifetime would be `'self', if such a thing existed).
#[derive(Debug)]
pub struct Subscriber<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error>
{
	_queues_drop_reference: Queues<MessageHandlerArguments, DequeuedMessageProcessingError>,
	queue: *const Queue<MessageHandlerArguments, DequeuedMessageProcessingError>,
	#[cfg(debug_assertions)] for_hyper_thread: HyperThread,
}

impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Subscriber<MessageHandlerArguments, DequeuedMessageProcessingError>
{
	#[inline(always)]
	fn new(queues: &Queues<MessageHandlerArguments, DequeuedMessageProcessingError>, for_hyper_thread: HyperThread) -> Self
	{
		Self
		{
			_queues_drop_reference: queues.clone(),
			queue: queues.0.get_unchecked_safe(for_hyper_thread),
			#[cfg(debug_assertions)] for_hyper_thread,
		}
	}
	
	/// Receives and handles messages; short-circuits if `self.terminate` becomes true or a message handler returns an error `DequeuedMessageProcessingError`.
	#[inline(always)]
	pub fn receive_and_handle_messages(&self, terminate: &Arc<impl Terminate>, message_handler_arguments: &MessageHandlerArguments) -> Result<(), DequeuedMessageProcessingError>
	{
		#[cfg(debug_assertions)]
		{
			debug_assert_eq!(self.for_hyper_thread, HyperThread::current().1, "Must only be accessed by one specific HyperThread")
		}
		
		let queue = unsafe { &*self.queue };
		queue.dequeue(terminate, message_handler_arguments)
	}
}
