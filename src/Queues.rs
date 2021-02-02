// This file is part of message-dispatch. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT. No part of message-dispatch, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2019-2020 The developers of message-dispatch. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/message-dispatch/master/COPYRIGHT.


/// Pass cloned copies of this to each thread at initialization.
#[derive(Debug)]
pub struct Queues<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error>(Arc<PerBitSetAwareData<HyperThread, Queue<MessageHandlerArguments, DequeuedMessageProcessingError>>>);

unsafe impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Send for Queues<MessageHandlerArguments, DequeuedMessageProcessingError>
{
}

unsafe impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Sync for Queues<MessageHandlerArguments, DequeuedMessageProcessingError>
{
}

impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Clone for Queues<MessageHandlerArguments, DequeuedMessageProcessingError>
{
	#[inline(always)]
	fn clone(&self) -> Self
	{
		Self(self.0.clone())
	}
}

impl<MessageHandlerArguments, DequeuedMessageProcessingError: error::Error> Queues<MessageHandlerArguments, DequeuedMessageProcessingError>
{
	/// One way to estimate `queue_size_in_bytes` is to find the largest `size_of::<>()` of all types of `FixedSizeMessageBody` in `message_handlers`.
	/// `message_handlers()` is invoked from the calling thread, not the thread that will then invoke the message handlers.
	/// This means that they should be allocated from global memory and will *not* be NUMA aware (or will steal from the calling thread's NUMA memory).
	#[inline(always)]
	pub fn one_queue_for_each_hyper_thread(hyper_threads: &BitSet<HyperThread>, message_handlers_and_preferred_maximum_number_of_elements_of_largest_possible_fixed_size_message_body_in_queue_for_hyper_thread
	: &impl Fn(HyperThread) -> (MessageHandlers<MessageHandlerArguments, Result<(), DequeuedMessageProcessingError>>, NonZeroU64), defaults: &DefaultHugePageSizes, inclusive_maximum_bytes_wasted: u64) -> Self
	{
		Self
		(
			Arc::new
			(
				PerBitSetAwareData::new
				(
					hyper_threads,
					|hyper_thread|
					{
						let (message_handlers, preferred_maximum_number_of_elements_of_largest_possible_fixed_size_message_body) = message_handlers_and_preferred_maximum_number_of_elements_of_largest_possible_fixed_size_message_body_in_queue_for_hyper_thread(hyper_thread);
						let queue_size_in_bytes = message_handlers.queue_size_in_bytes(preferred_maximum_number_of_elements_of_largest_possible_fixed_size_message_body);
						Queue::new(message_handlers, defaults, queue_size_in_bytes, inclusive_maximum_bytes_wasted).unwrap()
					}
				)
			)
		)
	}
	
	/// New publisher.
	#[inline(always)]
	pub fn publisher<M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>>(&self, default_hyper_thread: HyperThread) -> Publisher<M, MessageHandlerArguments, DequeuedMessageProcessingError>
	{
		Publisher::new(self, default_hyper_thread)
	}
	
	/// New round-robin publisher.
	///
	/// Loops infinitely around a set (`hyper_threads_to_publish_to`) of `HyperThread`s to publish to.
	#[inline(always)]
	pub fn round_robin_publisher<M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>>(&self, hyper_threads_to_publish_to: Box<[HyperThread]>) -> RoundRobinPublisher<M, MessageHandlerArguments, DequeuedMessageProcessingError>
	{
		RoundRobinPublisher::new(self, hyper_threads_to_publish_to)
	}
	
	/// A publisher publishes to a specific hyper thread.
	///
	/// ***SLOW*** as it uses a hash map look up.
	///
	/// If there is no queue for the hyper thread, publishes to itself.
	/// This supports a scenario under Linux using the `SO_INCOMING_CPU` socket option, which can map to a CPU not assigned to the process.
	///
	/// Prefer `publisher().publish()` to this method.
	///
	/// Returns the actual hyper thread published to.
	pub fn publish_safe_but_slow<M: 'static + Message<MessageHandlerArguments=MessageHandlerArguments, DequeuedMessageProcessingError=DequeuedMessageProcessingError>>(&self, hyper_thread: HyperThread, default_hyper_thread: HyperThread, construct_message_arguments: M::ConstructMessageArguments) -> HyperThread
	{
		let (queue, actual_hyper_thread) = self.0.get_or(hyper_thread, default_hyper_thread);
		let fixed_sized_message_body_compressed_type_identifier = queue.fixed_sized_message_body_compressed_type_identifier::<M>();
		unsafe { queue.enqueue(fixed_sized_message_body_compressed_type_identifier, |uninitialized_memory| M::construct_message(uninitialized_memory, construct_message_arguments)) };
		actual_hyper_thread
	}
	
	/// Only works for the current hyper thread.
	#[inline(always)]
	pub fn subscriber(&self, for_hyper_thread: HyperThread) -> Subscriber<MessageHandlerArguments, DequeuedMessageProcessingError>
	{
		Subscriber::new(self, for_hyper_thread)
	}
}
