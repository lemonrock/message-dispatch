// This file is part of linux-support. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/linux-support/master/COPYRIGHT. No part of linux-support, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2020 The developers of linux-support. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/linux-support/master/COPYRIGHT.


/// A round-robin publisher.
pub struct RoundRobinPublisher<'a, M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>, MessageHandlerArguments, DequeuedMessageProcessingError: error::Error>
{
	publisher: Publisher<'a, M, MessageHandlerArguments, DequeuedMessageProcessingError>,
	hyper_threads_to_publish_to: Box<[HyperThread]>,
	next_hyper_thread_to_publish_to_index: Cell<usize>,
}

impl<'a, M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>, MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> RoundRobinPublisher<'a, M, MessageHandlerArguments, DequeuedMessageProcessingError>
{
	#[inline(always)]
	fn new(queue_data: &'a PerBitSetAwareData<HyperThread, Queue<MessageHandlerArguments, DequeuedMessageProcessingError>>, hyper_threads_to_publish_to: Box<[HyperThread]>) -> Self
	{
		debug_assert_ne!(hyper_threads_to_publish_to.len(), 0);
		let default_hyper_thread = unsafe { * hyper_threads_to_publish_to.get_unchecked(0) };
		
		Self
		{
			publisher: Publisher::new(queue_data, default_hyper_thread),
			hyper_threads_to_publish_to,
			next_hyper_thread_to_publish_to_index: Cell::new(0),
		}
	}
	
	/// A publisher publishes to a specific hyper thread.
	///
	/// Returns the actual hyper thread published to.
	#[inline(always)]
	pub fn publish(&self, construct_message_arguments: M::ConstructMessageArguments)-> HyperThread
	{
		let next_hyper_thread_to_publish_to_index = self.next_hyper_thread_to_publish_to_index.get();
		let next_hyper_thread = unsafe { * self.hyper_threads_to_publish_to.get_unchecked(next_hyper_thread_to_publish_to_index) };
		if next_hyper_thread_to_publish_to_index == self.hyper_threads_to_publish_to.len()
		{
			self.next_hyper_thread_to_publish_to_index.set(0)
		}
		self.publisher.publish(next_hyper_thread, construct_message_arguments)
	}
}
